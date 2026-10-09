// AuthBOLT identity prompt (notification type "bolt_request").
//
// A site asked to see an identity (register, sign in). The site never chooses and never signs:
// the person does, here. The identity already linked to this site is offered first; otherwise a new
// identity for this site (its own key, so sites cannot link identities the person keeps apart);
// other identities are behind "Use another identity", each showing the sites it is linked to.
// The answer goes back to the page that asked as `bolt_result` (C++ holds its request meanwhile).
//
// Keep-alives never reach this screen: `registerSilentPresenter` answers them without a window,
// only for an app the person chose to stay signed in to.
//
// After registering, the app knows the identity's holder key, and the page asks for signatures with
// POST /bolt/sign (`BoltSignPrompt` below; silently through `window.boltSign`). The wallet builds
// each digest itself (ChainBrowsers packages/bolt `signDigest`).
import React, { useEffect, useState } from 'react';
import { HodosButton } from './HodosButton';
import { walletFetch } from '../services/walletApi';
import type { Identity, IdentityWallet, SignKind } from '../vendor/bolt-identity.js';

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
 * Answer keep-alives and writes with no window (C++ calls `window.boltSilent(json)` in the preloaded
 * notification browser). Presents only when the person chose to stay signed in to that app on that
 * site; otherwise answers NEEDS_PROMPT: a keep-alive's session lapses at its expiry, and a write is
 * not made. A write is one change the person makes in the app (tag 04), signed in passing.
 */
export function registerSilentPresenter() {
  (window as any).boltSilent = async (argsJson: string) => {
    let args: { key: string; domain: string; appPubKey: string; data: string; purpose: string };
    try {
      args = JSON.parse(argsJson);
    } catch {
      return;
    }
    try {
      if (args.purpose !== 'refresh' && args.purpose !== 'write') {
        throw Object.assign(new Error('only a keep-alive or a write is answered without asking'), { code: 'NEEDS_PROMPT' });
      }
      const shown = await (await identities()).refresh({ domain: args.domain, appPubKey: args.appPubKey, data: args.data });
      answer(args.key, true, { package: shown.package });
    } catch (e: any) {
      refuse(args.key, e);
    }
  };
  // POST /bolt/sign, silent: signs only under the grant and only silent-tier writes (the wallet
  // decides, from the app's published tiers); anything else answers NEEDS_PROMPT.
  const page = window as unknown as Record<string, unknown>;
  page.boltSign = async (argsJson: string) => {
    let args: SignArgs;
    try {
      args = JSON.parse(argsJson);
    } catch {
      return;
    }
    try {
      const got = await (await identities()).answer({ domain: args.domain, appPubKey: args.appPubKey, kind: args.kind, payload: args.payload, silent: true });
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
        d = got.appPubKey === appPubKey && got.purpose === purpose ? got : null;
      } catch { /* not auth data: refused below */ }
      if (live) setDecoded(d);
    });
    return () => { live = false; };
  }, [data, appPubKey, purpose]);

  useEffect(() => {
    let live = true;
    (async () => {
      try {
        const all = await (await identities()).identities();
        const mine = all.filter((t) => t.apps.some((a) => a.domain === domain && a.appPubKey === appPubKey));
        if (!live) return;
        setLinked(mine);
        setOthers(all.filter((t) => !mine.includes(t)));
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
      setBusy(purpose === 'register' ? 'Preparing your registration…' : 'Signing you in…');
      const shown = await (await identities()).present({ id, domain, appPubKey, data, keepSignedIn });
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

  const title = purpose === 'register' ? 'Create an account' : 'Sign in';
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
    const sites = [...new Set(t.apps.map((a) => a.domain))];
    return sites.length ? `Linked to ${sites.join(', ')}` : 'Not linked to any site';
  };

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px', margin: '0 0 8px' }}>
        {/* The BOLT Association's mark (boltassociation.com): this is an AuthBOLT request. */}
        <img src="/authbolt.png" alt="" width={36} height={36} style={{ borderRadius: '8px', flex: 'none' }} />
        <h2 style={{ fontSize: '18px', fontWeight: 600, color: COLORS.text, margin: 0 }}>
          {title} on {domain}
          <span style={{ display: 'block', fontSize: '12px', fontWeight: 500, color: COLORS.muted, marginTop: '2px' }}>with an AuthBOLT identity</span>
        </h2>
      </div>
      <p style={{ margin: '0 0 16px', fontSize: '14px', color: COLORS.text, lineHeight: 1.6 }}>
        {domain} asks to see an identity token from this wallet. It learns only that identity, and
        that you chose to show it. No password, and nothing is spent{choice === NEW ? ' except a small network fee to create the token' : ''}.
      </p>

      {linked === null && !error && <p style={{ color: COLORS.muted }}>Reading your identities…</p>}

      {linked !== null && (
        <div style={{ marginBottom: '12px' }}>
          {linked.map((t) => row(t.id, `Your identity for this site`, `${short(t.issuer)} · ${linkedTo(t)}`))}
          {linked.length === 0 && row(NEW, 'A new identity for this site', 'Recommended: its own key, so other sites cannot tell it is you')}
          {(showOthers || linked.length === 0) && others.map((t) => row(t.id, `Identity ${short(t.issuer)}`, linkedTo(t)))}
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
        Keep me signed in on {domain} (it can renew your session without asking)
      </label>

      {busy && <p style={{ color: COLORS.muted, margin: '0 0 12px' }}>{busy}</p>}
      {error && <p role="alert" style={{ color: COLORS.error, margin: '0 0 12px' }}>{error}</p>}

      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px' }}>
        <HodosButton variant="secondary" onClick={decline} disabled={!!busy}>Decline</HodosButton>
        <HodosButton variant="primary" onClick={approve} disabled={!!busy || linked === null || !choice}>
          {purpose === 'register' ? 'Create account' : 'Sign in'}
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
  confirm: 'Change your signing key',
  recover: 'Recover your account',
};

/**
 * A prompted /bolt/sign request ("bolt_sign"): a sign-in without the keep-signed-in grant, a change
 * the wallet always asks about (roles, approvals, blocks, a channel's settings), or a recovery with
 * the identity's issuer key. The person sees what is signed, as text, and for which identity.
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
    setBusy(args.kind === 'recover' ? 'Signing with your identity key…' : 'Signing…');
    try {
      const ids = await identities();
      const got = await ids.answer({ id: choice, domain: args.domain, appPubKey: args.appPubKey, kind: args.kind, payload: args.payload, silent: false });
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
      {args.kind === 'recover' && (
        <p style={{ margin: '0 0 12px', fontSize: '14px', color: COLORS.text, lineHeight: 1.6 }}>
          {domain} asks you to recover your account: your identity key signs a new signing key for this
          site, and every session there ends. Do this only if you asked to recover your account.
        </p>
      )}
      {(args.kind === 'signin' || args.kind === 'refresh' || args.kind === 'rotate' || args.kind === 'confirm') && (
        <p style={{ margin: '0 0 12px', fontSize: '14px', color: COLORS.text, lineHeight: 1.6 }}>
          {args.kind === 'rotate' || args.kind === 'confirm'
            ? `${domain} asks to move your account to a new signing key held by this wallet.`
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
          Identity {short(t.issuer)}
        </label>
      ))}
      {(args.kind === 'signin' || args.kind === 'refresh') && (
        <label style={{ display: 'flex', gap: '8px', alignItems: 'center', color: COLORS.text, fontSize: '14px', margin: '4px 0 16px' }}>
          <input type="checkbox" checked={keepSignedIn} onChange={(e) => setKeepSignedIn(e.target.checked)} />
          Keep me signed in on {domain} (it can renew your session and sign everyday changes without asking)
        </label>
      )}

      {busy && <p style={{ color: COLORS.muted, margin: '0 0 12px' }}>{busy}</p>}
      {error && <p role="alert" style={{ color: COLORS.error, margin: '0 0 12px' }}>{error}</p>}

      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px' }}>
        <HodosButton variant="secondary" onClick={decline} disabled={!!busy}>Decline</HodosButton>
        <HodosButton variant="primary" onClick={approve} disabled={!!busy || !choice}>
          {args.kind === 'write' ? 'Sign this change' : args.kind === 'recover' ? 'Recover' : 'Sign'}
        </HodosButton>
      </div>
    </div>
  );
};
