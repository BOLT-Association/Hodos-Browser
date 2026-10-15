<#
.SYNOPSIS
  Start a throwaway Hodos wallet in spv mode against the public Teranode testnet.

.DESCRIPTION
  Runs rust-wallet\target\debug\hodos-wallet.exe (build it first: `cargo build` in rust-wallet)
  on 127.0.0.1:31401 with HODOS_DEV=1 and a scratch HODOS_DATA_DIR, so it never opens the
  real HodosBrowser or HodosBrowserDev wallet. Stop it with Ctrl+C; never stop a wallet by
  image name (scripts\stop-dev.ps1 explains why).

.PARAMETER DataDir  Scratch data root (default %TEMP%\hodos-ttn-wallet). Reused across runs.
.PARAMETER Arcade   Arcade base URL; chaintracks is <Arcade>/chaintracks/v2.
.PARAMETER Network  Header-chain rules (teratestnet; `regtest` is the negative control).
.PARAMETER Headers  Header depth: full (default; every header from the genesis) or lite (a verified
                    window from the 18000 checkpoint; the header trackers answer below it).
.PARAMETER Trackers  Extra chaintracks base URLs for the header trackers, comma-separated (lite only;
                    <Arcade>/chaintracks/v2 is always the first source).
.PARAMETER Push     Also subscribe to Arcade's SSE status stream at <Arcade>/events.
#>
param(
  [string]$DataDir = (Join-Path $env:TEMP 'hodos-ttn-wallet'),
  [string]$Arcade = 'https://arcade-ttn-us-1.bsvblockchain.tech',
  [string]$Network = 'teratestnet',
  [ValidateSet('full', 'lite')][string]$Headers = 'full',
  [string]$Trackers = '',
  [switch]$Push
)
$ErrorActionPreference = 'Stop'

$full = [IO.Path]::GetFullPath($DataDir).TrimEnd('\')
foreach ($real in 'HodosBrowser', 'HodosBrowserDev') {
  $realDir = (Join-Path $env:APPDATA $real).TrimEnd('\')
  if ($full -ieq $realDir -or $full.StartsWith("$realDir\", [StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to run a test wallet in the real data directory $realDir"
  }
}

$exe = Join-Path $PSScriptRoot '..\..\target\debug\hodos-wallet.exe'
if (-not (Test-Path $exe)) { throw "Not built: run 'cargo build' in rust-wallet first ($exe)" }
New-Item -ItemType Directory -Force (Join-Path $full 'wallet') | Out-Null

$Arcade = $Arcade.TrimEnd('/')
$env:HODOS_DEV = '1'
$env:HODOS_DATA_DIR = $full
$env:HODOS_CHAIN_MODE = 'spv'
$env:HODOS_CHAIN_NETWORK = $Network
$env:HODOS_ARCADE_URL = $Arcade
$env:HODOS_CHAINTRACKS_URL = "$Arcade/chaintracks/v2"
$env:HODOS_HEADERS = $Headers
if ($Trackers) { $env:HODOS_HEADER_TRACKERS = $Trackers } else { Remove-Item Env:HODOS_HEADER_TRACKERS -ErrorAction SilentlyContinue }
if ($Push) { $env:HODOS_ARCADE_SSE_URL = $Arcade } else { Remove-Item Env:HODOS_ARCADE_SSE_URL -ErrorAction SilentlyContinue }

Write-Host "Hodos wallet: spv / $Network / headers $Headers / $Arcade / data $full / push $([bool]$Push)"
# The wallet logs to stderr; under 'Stop', Windows PowerShell 5.1 turns the first stderr line
# of a native command into a terminating error and the wallet dies at startup.
$ErrorActionPreference = 'Continue'
& $exe
exit $LASTEXITCODE
