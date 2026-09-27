# beta.6 release — intake only

**Opened:** 2026-09-25, during beta.5 planning. **Status:** 📥 **INTAKE ONLY** — no planning has run.
This folder exists so work that belongs to the *next* release has a home the day it is found, instead
of being lost or squeezed into beta.5.

⛔ Nothing here is scheduled. beta.6 planning starts at its own gate `G0` (`../RELEASE_CYCLE.md`),
after beta.5 ships and its AAR is written.

| File | What |
|---|---|
| `tickets/` | Tickets filed for beta.6. Same conventions as `../0.4.0-beta.6/tickets/README.md` |

## Deferred here by the planned release's G2 decisions (2026-09-27)

Pointers, so nothing deferred is lost. Each item's evidence stays where it was researched.

| Item | Re-check condition | Source |
|---|---|---|
| **x402 adapter** (B5-T4-P3) | x402 PR #2890 merges, or a live BRC-29 `exact` server exists, or the owner wants bsv.cx | decision 9 · `../0.4.0-beta.6/track-4-402-payments/SCOPE.md` |
| **BSV-21 transfer** (B5-T2-P5) | a named user need; BRC PR #273 (binary BSV-21) settled | decision 8 · `../0.4.0-beta.6/track-2-1sat-ordinals/SCOPE.md` |
| **Split view** | after session restore (B5-T6-P2) lands — its own scoping run | decision 14 · ticket in `../0.4.0-beta.6/tickets/` (❄️ deferred) |
| **Chrome password import** — CSV only, never Chrome's keys | — | decision 14 |
| **Site-scoped identity keys** (derived-keys part 3) | the ecosystem adopts it — carried in the BRC draft `originator-scoped-authentication-keys` | decision 11 |
| **BRC-140 key shares** — build no earlier than here | root-key convention settled (decision 6) | T3 SCOPE Q5 |

⚠️ **Naming:** once the planned release's folder is renamed to `0.4.0-beta.7/` (decision 1), this intake folder becomes `0.4.0-beta.7/`.

