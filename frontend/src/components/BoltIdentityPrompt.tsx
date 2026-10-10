// AuthBOLT identity prompt (notification type "bolt_request").
//
// A site asked to register an identity. The site never chooses and never signs: the person does,
// here. Registering moves the identity token on chain from its mint to the identity's first holder
// key, so only an identity whose token has not moved yet can register: a new identity for this
// site (its own key, so sites cannot link identities the person keeps apart) is offered, or an
// unused one the wallet already holds. The answer goes back to the page that asked as `bolt_result`
// (C++ holds its request meanwhile).
//
// The app pays for every move of the token: the wallet asks the page for a coin per transaction
// (`bolt_fund`; C++ runs the page's fund callback) and the page's answer comes back through
// `window.boltFundResult`. Nothing the page sends is trusted: the wallet checks each coin's amount,
// and a coin that is not valid makes the transaction fail on the network.
//
// After registering, the page asks for signatures and the moves the app asks for with POST
// /bolt/sign (`BoltSignPrompt` below; silently through `window.boltSign`). The wallet builds each
// digest and transaction itself (ChainBrowsers packages/bolt `signDigest`, `IdentityWallet`).
import React, { useEffect, useState } from 'react';
import { HodosButton } from './HodosButton';
import { walletFetch } from '../services/walletApi';
import type { Coin, CoinRequest, Funder, Identity, IdentityWallet, SignKind } from '../vendor/bolt-identity.js';

// The identity code (b017 + the SDK, ~400 KB) is loaded the first time an AuthBOLT request needs
// it, not with the overlay: every other prompt type shares this overlay and must stay light.
type IdentityModule = typeof import('../vendor/bolt-identity.js');
let moduleLoad: Promise<IdentityModule> | null = null;
const loadIdentityModule = () => (moduleLoad ??= import('../vendor/bolt-identity.js'));

const COLORS = {
  text: '#f0f0f0',
  muted: '#9ca3af',
  border: '#2a2d35',
  gold: '#a67c00',
  error: '#ef5350',
};

/** The wallet's rails, called as Hodos itself (no site domain: the wallet treats it as internal). */
async function walletCall(endpoint: string, body?: unknown): Promise<any> {
  const res = await walletFetch(endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body ?? {}),
  });
  const json = await res.json().catch(() => null);
  if (!res.ok || (json && json.error)) {
    throw new Error(typeof json?.error === 'string' ? json.error : `wallet ${endpoint} failed (${res.status})`);
  }
  return json;
}

let service: Promise<IdentityWallet> | null = null;
const identities = (): Promise<IdentityWallet> =>
  (service ??= loadIdentityModule().then((m) => m.identityService(walletCall)));

/** Send the answer for request `key` to the page that asked. */
function answer(key: string, ok: boolean, payload: unknown) {
  window.cefMessage?.send('bolt_result', [key, ok, JSON.stringify(payload)]);
}

const short = (hex: string) => `${hex.slice(0, 8)}…${hex.slice(-6)}`;

/**
 * The name an app accepted for an identity (BOLT.label, after a successful sign-in): for that app on
 * that site when given, else the first the identity has. The app's text, rendered as text only.
 */
const nameOf = (t: Identity, app?: { domain: string; appPubKey: string }) =>
  (app ? t.apps.find((a) => a.domain === app.domain && a.appPubKey === app.appPubKey) : t.apps.find((a) => a.label))?.label;

/** A coin request waiting for the page's answer, by request key (a move asks one at a time). */
const coinAsks = new Map<string, { resolve: (coin: Coin) => void; reject: (e: Error) => void }>();
const COIN_TIMEOUT_MS = 60_000;

function askPage(key: string, request: CoinRequest): Promise<Coin> {
  return new Promise((resolve, reject) => {
    coinAsks.get(key)?.reject(new Error('a newer coin request replaced this one'));
    const timer = window.setTimeout(() => {
      coinAsks.delete(key);
      reject(new Error('the site gave no coin in time to pay for moving your identity token'));
    }, COIN_TIMEOUT_MS);
    coinAsks.set(key, {
      resolve: (coin) => { window.clearTimeout(timer); coinAsks.delete(key); resolve(coin); },
      reject: (e) => { window.clearTimeout(timer); coinAsks.delete(key); reject(e); },
    });
    window.cefMessage?.send('bolt_fund', [key, JSON.stringify(request)]);
  });
}

/** The app's coins for request `key`, asked of the page that holds it. */
const funderFor = async (key: string): Promise<Funder> =>
  (await loadIdentityModule()).textFunder((request) => askPage(key, request));

/**
 * Install what C++ calls in the preloaded notification browser: `boltSign` (a silent /bolt/sign),
 * `boltSignPrompt` (a prompted one's arguments, before its screen shows), `boltLabel` (the name an
 * app accepted for an identity) and `boltFundResult` (the page's answer to a coin request).
 */
export function registerBoltHandlers() {
  const page = window as unknown as Record<string, unknown>;
  // POST /bolt/sign, silent: answers only under the grant, only silent-tier writes and rotations
  // (the wallet decides, from the app's published tiers); anything else answers NEEDS_PROMPT.
  page.boltSign = async (argsJson: string) => {
    let args: SignArgs;
    try {
      args = JSON.parse(argsJson);
    } catch {
      return;
    }
    try {
      const got = await (await identities()).answer({
        domain: args.domain, appPubKey: args.appPubKey, kind: args.kind, payload: args.payload, silent: true,
        count: args.count, holder: args.holder, funder: await funderFor(args.key),
      });
      answer(args.key, true, got);
    } catch (e) {
      refuse(args.key, e);
    }
  };
  // POST /bolt/sign, prompted: C++ hands the arguments over here first, then shows "bolt_sign",
  // which reads them by key (a write can be 64 KiB: it does not travel in the overlay's URL).
  page.boltSignPrompt = (argsJson: string) => {
    try {
      const args: SignArgs = JSON.parse(argsJson);
      signRequests.set(args.key, args);
      window.dispatchEvent(new Event('bolt-sign-args'));
    } catch { /* malformed: the request times out as declined */ }
  };
  // POST /bolt/label, never shown: keep the name the app accepted for an identity linked to it.
  page.boltLabel = async (argsJson: string) => {
    let args: { key: string; domain: string; appPubKey: string; identity: string; name: string };
    try {
      args = JSON.parse(argsJson);
    } catch {
      return;
    }
    try {
      await (await identities()).label({ domain: args.domain, appPubKey: args.appPubKey, identity: args.identity, name: args.name });
      answer(args.key, true, { ok: true });
    } catch (e) {
      refuse(args.key, e);
    }
  };
  page.boltFundResult = (json: string) => {
    let got: { key?: string; coin?: Coin; error?: string };
    try {
      got = JSON.parse(json);
    } catch {
      return;
    }
    const ask = got.key ? coinAsks.get(got.key) : undefined;
    if (!ask) return; // nothing asked (or it timed out): ignored
    if (got.coin) ask.resolve(got.coin);
    else ask.reject(new Error(`the site gave no coin: ${got.error ?? 'no reason given'}`));
  };
}

/** A wallet error as the page is told it: NEEDS_PROMPT when the wallet would have to ask. */
type WalletError = { message?: string; code?: string };
const messageOf = (e: unknown) => (e as WalletError)?.message ?? String(e);

function refuse(key: string, e: unknown) {
  const needs = (e as WalletError)?.code === 'NEEDS_PROMPT';
  answer(key, false, { error: `BOLT: ${needs ? 'NEEDS_PROMPT ' : ''}${messageOf(e)}`, code: needs ? 'NEEDS_PROMPT' : 'WALLET' });
}

interface SignArgs {
  key: string;
  domain: string;
  appPubKey: string;
  kind: SignKind;
  payload: string;
  silent: boolean;
  /** The holder the app expects (a refused sign-in named it): its count and hash. Checked by C++. */
  count?: number;
  holder?: string;
}

/** Prompted /bolt/sign requests waiting for their screen, by key. */
const signRequests = new Map<string, SignArgs>();

interface Props {
  requestKey: string;
  domain: string;
  appPubKey: string;
  data: string;
  purpose: string;
  onDone: () => void;
}

const NEW = 'new';

export const BoltIdentityPrompt: React.FC<Props> = ({ requestKey, domain, appPubKey, data, purpose, onDone }) => {
  const [linked, setLinked] = useState<Identity[] | null>(null);
  const [others, setOthers] = useState<Identity[]>([]);
  const [choice, setChoice] = useState<string>('');
  const [showOthers, setShowOthers] = useState(false);
  const [keepSignedIn, setKeepSignedIn] = useState(true);
  const [busy, setBusy] = useState('');
  const [error, setError] = useState('');

  // The request was checked by C++ already; decoding again here is the prompt's own check that the
  // data names the app it says it is for, before anything is shown or signed.
  const [decoded, setDecoded] = useState<ReturnType<IdentityModule['decodeAuthData']> | null | undefined>(undefined);
  useEffect(() => {
    let live = true;
    loadIdentityModule().then((m) => {
      let d: ReturnType<IdentityModule['decodeAuthData']> | null = null;
      try {
        const got = m.decodeAuthData(data);
        d = got.appPubKey === appPubKey && got.purpose === 'register' && purpose === 'register' ? got : null;
      } catch { /* not auth data: refused below */ }
      if (live) setDecoded(d);
    });
    return () => { live = false; };
  }, [data, appPubKey, purpose]);

  useEffect(() => {
    let live = true;
    (async () => {
      try {
        // Only a token still at its mint can register (its move to holder 1 is the registration).
        const unused = (await (await identities()).identities()).filter((t) => t.holderCount === 0);
        const mine = unused.filter((t) => t.apps.some((a) => a.domain === domain && a.appPubKey === appPubKey));
        if (!live) return;
        setLinked(mine);
        setOthers(unused.filter((t) => !mine.includes(t)));
        setChoice(mine[0]?.id ?? NEW);
        const app = mine[0]?.apps.find((a) => a.domain === domain && a.appPubKey === appPubKey);
        if (app) setKeepSignedIn(app.keepSignedIn);
      } catch (e: any) {
        if (live) setError(`The wallet's identities could not be read: ${e?.message ?? e}`);
      }
    })();
    return () => { live = false; };
  }, [domain, appPubKey]);

  const decline = () => {
    answer(requestKey, false, { error: 'BOLT: the user declined', code: 'DECLINED' });
    onDone();
  };

  const approve = async () => {
    if (!decoded) return;
    setError('');
    try {
      let id = choice;
      if (choice === NEW) {
        setBusy('Creating your identity for this site…');
        id = (await (await identities()).create()).id;
      }
      setBusy('Registering your identity (the site pays the network fees)…');
      const shown = await (await identities()).present({ id, domain, appPubKey, data, keepSignedIn, funder: await funderFor(requestKey) });
      answer(requestKey, true, { package: shown.package });
      onDone();
    } catch (e: any) {
      setBusy('');
      setError(e?.message ?? String(e));
    }
  };

  if (decoded === undefined) {
    return <p style={{ color: COLORS.muted }}>Reading the request…</p>;
  }
  if (!decoded) {
    // Should not happen (C++ checks it); refuse rather than show a prompt about the wrong app.
    return (
      <div>
        <p style={{ color: COLORS.error }}>This request does not match the site that sent it, so it was refused.</p>
        <HodosButton variant="secondary" onClick={decline}>Close</HodosButton>
      </div>
    );
  }

  const row = (id: string, label: React.ReactNode, detail: string) => (
    <label key={id} style={{ display: 'flex', gap: '10px', alignItems: 'flex-start', padding: '10px 12px',
      border: `1px solid ${choice === id ? COLORS.gold : COLORS.border}`, borderRadius: '8px', cursor: 'pointer', marginBottom: '8px' }}>
      <input type="radio" name="bolt-identity" checked={choice === id} onChange={() => setChoice(id)} style={{ marginTop: '3px' }} />
      <span>
        <span style={{ color: COLORS.text, fontWeight: 600 }}>{label}</span>
        <span style={{ display: 'block', color: COLORS.muted, fontSize: '12px', marginTop: '2px' }}>{detail}</span>
      </span>
    </label>
  );
  const linkedTo = (t: Identity) => {
    const sites = [...new Set(t.apps.map((a) => (a.label ? `${a.label} on ${a.domain}` : a.domain)))];
    return sites.length ? `Linked to ${sites.join(', ')}` : 'Not linked to any site';
  };

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px', margin: '0 0 8px' }}>
        {/* The BOLT Association's mark (boltassociation.com): this is an AuthBOLT request. */}
        <img src="/authbolt.png" alt="" width={36} height={36} style={{ borderRadius: '8px', flex: 'none' }} />
        <h2 style={{ fontSize: '18px', fontWeight: 600, color: COLORS.text, margin: 0 }}>
          Create an account on {domain}
          <span style={{ display: 'block', fontSize: '12px', fontWeight: 500, color: COLORS.muted, marginTop: '2px' }}>with an AuthBOLT identity</span>
        </h2>
      </div>
      <p style={{ margin: '0 0 16px', fontSize: '14px', color: COLORS.text, lineHeight: 1.6 }}>
        {domain} asks to register an identity token from this wallet. It learns only that identity, and
        that you chose to show it. No password. {domain} pays the network fees for registering
        it{choice === NEW ? '; creating the new token costs a small network fee from this wallet' : ''}.
      </p>

      {linked === null && !error && <p style={{ color: COLORS.muted }}>Reading your identities…</p>}

      {linked !== null && (
        <div style={{ marginBottom: '12px' }}>
          {linked.map((t) => row(t.id, `Your identity for this site`, `${short(t.issuer)} · ${linkedTo(t)}`))}
          {linked.length === 0 && row(NEW, 'A new identity for this site', 'Recommended: its own key, so other sites cannot tell it is you')}
          {(showOthers || linked.length === 0) && others.map((t) => row(t.id, nameOf(t) ? `${nameOf(t)} · ${short(t.issuer)}` : `Identity ${short(t.issuer)}`, linkedTo(t)))}
          {linked.length > 0 && showOthers && row(NEW, 'A new identity for this site', 'Its own key, unlinked from your others')}
          {linked.length > 0 && !showOthers && (
            <button type="button" onClick={() => setShowOthers(true)}
              style={{ background: 'none', border: 0, color: COLORS.gold, cursor: 'pointer', padding: '4px 0', textDecoration: 'underline' }}>
              Use another identity
            </button>
          )}
        </div>
      )}

      <label style={{ display: 'flex', gap: '8px', alignItems: 'center', color: COLORS.text, fontSize: '14px', margin: '4px 0 16px' }}>
        <input type="checkbox" checked={keepSignedIn} onChange={(e) => setKeepSignedIn(e.target.checked)} />
        Keep me signed in on {domain} (it can renew your session and change your signing key without asking)
      </label>

      {busy && <p style={{ color: COLORS.muted, margin: '0 0 12px' }}>{busy}</p>}
      {error && <p role="alert" style={{ color: COLORS.error, margin: '0 0 12px' }}>{error}</p>}

      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px' }}>
        <HodosButton variant="secondary" onClick={decline} disabled={!!busy}>Decline</HodosButton>
        <HodosButton variant="primary" onClick={approve} disabled={!!busy || linked === null || !choice}>
          Create account
        </HodosButton>
      </div>
    </div>
  );
};

/** How a write reads in the prompt: its kind, where it goes, and what it changes, as text. */
function describeWrite(payload: string): { kind: string; target: string; body: string } {
  try {
    const w = JSON.parse(payload);
    return {
      kind: String(w?.kind ?? ''),
      target: String(w?.target ?? ''),
      body: JSON.stringify(w?.body ?? {}, null, 2).slice(0, 4000),
    };
  } catch {
    return { kind: '', target: '', body: payload.slice(0, 4000) };
  }
}

const SIGN_TITLES: Record<SignKind, string> = {
  signin: 'Sign in',
  refresh: 'Stay signed in',
  write: 'Approve a change',
  rotate: 'Change your signing key',
  reissue: 'Recover your account',
};

const moves = (kind: SignKind) => kind === 'rotate' || kind === 'reissue';

/**
 * A prompted /bolt/sign request ("bolt_sign"): a sign-in without the keep-signed-in grant, a change
 * the wallet always asks about (roles, approvals, blocks, a channel's settings), a key change the
 * app asked for, or a recovery (a new token from the identity's issuer key). The person sees what
 * is signed, as text, and for which identity.
 */
export const BoltSignPrompt: React.FC<{ requestKey: string; domain: string; onDone: () => void }> = ({ requestKey, domain, onDone }) => {
  const [args, setArgs] = useState<SignArgs | undefined>(() => signRequests.get(requestKey));
  const [linked, setLinked] = useState<Identity[] | null>(null);
  const [choice, setChoice] = useState('');
  const [keepSignedIn, setKeepSignedIn] = useState(true);
  const [busy, setBusy] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    const read = () => setArgs(signRequests.get(requestKey));
    read();
    window.addEventListener('bolt-sign-args', read);
    return () => window.removeEventListener('bolt-sign-args', read);
  }, [requestKey]);

  useEffect(() => {
    if (!args) return;
    let live = true;
    (async () => {
      try {
        const mine = await (await identities()).forApp({ domain: args.domain, appPubKey: args.appPubKey });
        if (!live) return;
        setLinked(mine);
        setChoice(mine[0]?.id ?? '');
        const app = mine[0]?.apps.find((a) => a.domain === args.domain && a.appPubKey === args.appPubKey);
        if (app) setKeepSignedIn(app.keepSignedIn);
      } catch (e) {
        if (live) setError(`The wallet's identities could not be read: ${messageOf(e)}`);
      }
    })();
    return () => { live = false; };
  }, [args]);

  const finish = (ok: boolean, payload: unknown) => {
    signRequests.delete(requestKey);
    answer(requestKey, ok, payload);
    onDone();
  };
  const decline = () => finish(false, { error: 'BOLT: the user declined', code: 'DECLINED' });

  const approve = async () => {
    if (!args || !choice) return;
    setError('');
    setBusy(moves(args.kind) ? 'Moving your identity token (the site pays the network fees)…' : 'Signing…');
    try {
      const ids = await identities();
      const got = await ids.answer({
        id: choice, domain: args.domain, appPubKey: args.appPubKey, kind: args.kind, payload: args.payload, silent: false,
        count: args.count, holder: args.holder, funder: moves(args.kind) ? await funderFor(requestKey) : undefined,
      });
      if (args.kind === 'signin' || args.kind === 'refresh') {
        await ids.setKeepSignedIn({ id: choice, domain: args.domain, appPubKey: args.appPubKey, keep: keepSignedIn });
      }
      finish(true, got);
    } catch (e) {
      setBusy('');
      setError(messageOf(e));
    }
  };

  if (!args) {
    return <p style={{ color: COLORS.muted }}>Reading the request…</p>;
  }
  const write = args.kind === 'write' ? describeWrite(args.payload) : null;
  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px', margin: '0 0 8px' }}>
        <img src="/authbolt.png" alt="" width={36} height={36} style={{ borderRadius: '8px', flex: 'none' }} />
        <h2 style={{ fontSize: '18px', fontWeight: 600, color: COLORS.text, margin: 0 }}>
          {SIGN_TITLES[args.kind] ?? 'Sign'} on {domain}
          <span style={{ display: 'block', fontSize: '12px', fontWeight: 500, color: COLORS.muted, marginTop: '2px' }}>with your AuthBOLT identity</span>
        </h2>
      </div>

      {write && (
        <div style={{ margin: '0 0 12px' }}>
          <p style={{ margin: '0 0 6px', fontSize: '14px', color: COLORS.text }}>
            {domain} asks you to sign this change: <strong>{write.kind}</strong>
            <span style={{ display: 'block', color: COLORS.muted, fontSize: '12px' }}>{write.target}</span>
          </p>
          <pre style={{ maxHeight: '180px', overflow: 'auto', background: '#14161a', border: `1px solid ${COLORS.border}`,
            borderRadius: '6px', padding: '8px', color: COLORS.text, fontSize: '12px', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
            {write.body}
          </pre>
        </div>
      )}
      {args.kind === 'reissue' && (
        <p style={{ margin: '0 0 12px', fontSize: '14px', color: COLORS.text, lineHeight: 1.6 }}>
          {domain} asks you to recover your account: your identity key makes a new identity token and
          moves it to a new signing key held by this wallet, and the old token can no longer be used.
          {' '}{domain} pays the network fees. Do this only if you asked to recover your account.
        </p>
      )}
      {(args.kind === 'signin' || args.kind === 'refresh' || args.kind === 'rotate') && (
        <p style={{ margin: '0 0 12px', fontSize: '14px', color: COLORS.text, lineHeight: 1.6 }}>
          {args.kind === 'rotate'
            ? `${domain} asks to move your account to a new signing key held by this wallet. ${domain} pays the network fees.`
            : `${domain} asks you to sign in with the identity you registered there. Nothing is spent.`}
        </p>
      )}

      {linked === null && !error && <p style={{ color: COLORS.muted }}>Reading your identities…</p>}
      {linked !== null && linked.length === 0 && (
        <p style={{ color: COLORS.error }}>None of your identities is registered with {domain}.</p>
      )}
      {linked !== null && linked.length > 1 && linked.map((t) => (
        <label key={t.id} style={{ display: 'flex', gap: '8px', alignItems: 'center', color: COLORS.text, fontSize: '14px', marginBottom: '6px' }}>
          <input type="radio" name="bolt-sign-identity" checked={choice === t.id} onChange={() => setChoice(t.id)} />
          {nameOf(t, args) ? `${nameOf(t, args)} · ${short(t.issuer)}` : `Identity ${short(t.issuer)}`}
        </label>
      ))}
      {linked !== null && linked.length === 1 && nameOf(linked[0], args) && (
        <p style={{ margin: '0 0 12px', fontSize: '14px', color: COLORS.text }}>
          As <strong>{nameOf(linked[0], args)}</strong>
          <span style={{ color: COLORS.muted, fontSize: '12px' }}> · {short(linked[0].issuer)}</span>
        </p>
      )}
      {(args.kind === 'signin' || args.kind === 'refresh') && (
        <label style={{ display: 'flex', gap: '8px', alignItems: 'center', color: COLORS.text, fontSize: '14px', margin: '4px 0 16px' }}>
          <input type="checkbox" checked={keepSignedIn} onChange={(e) => setKeepSignedIn(e.target.checked)} />
          Keep me signed in on {domain} (it can renew your session, sign everyday changes and change your signing key without asking)
        </label>
      )}

      {busy && <p style={{ color: COLORS.muted, margin: '0 0 12px' }}>{busy}</p>}
      {error && <p role="alert" style={{ color: COLORS.error, margin: '0 0 12px' }}>{error}</p>}

      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px' }}>
        <HodosButton variant="secondary" onClick={decline} disabled={!!busy}>Decline</HodosButton>
        <HodosButton variant="primary" onClick={approve} disabled={!!busy || !choice}>
          {args.kind === 'write' ? 'Sign this change' : args.kind === 'reissue' ? 'Recover' : args.kind === 'rotate' ? 'Change key' : 'Sign'}
        </HodosButton>
      </div>
    </div>
  );
};
