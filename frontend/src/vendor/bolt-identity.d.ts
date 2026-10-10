// Types for bolt-identity.js, the bundled AuthBOLT identity service (ChainBrowsers packages/bolt,
// `npm run bundle:identity`). Hodos's own identity prompt runs it; pages never do.

export interface IdentityApp {
  domain: string;
  appPubKey: string;
  keepSignedIn: boolean;
  /** The name the app accepted for this identity (the app's text, reported after a sign-in). */
  label?: string;
  /** When this identity was last used with this app (ms); `forApp` lists the last used first. */
  lastUsedAt?: number;
  linkedAt: number;
}

export interface Identity {
  id: string;
  type: 'AuthBOLT';
  issuer: string;
  keyId: string;
  holderKeyId: string;
  /** How many times the token has moved to a new holder key: 0 while it sits at its mint. */
  holderCount: number;
  apps: IdentityApp[];
}

export interface Presentation {
  package: string[];
  id: string;
}

/** What the page's /bolt/sign asks: a holder-key signature, or a move of the identity token. */
export type SignKind = 'signin' | 'refresh' | 'write' | 'rotate' | 'reissue';

export interface SignAnswer {
  identity?: string;
  holder?: string;
  signature?: string;
  package?: string[];
  id?: string;
}

/** The app's coin for one step of a move, as the wallet receives it. */
export type Funder = (o: { step: string; amount: number; tx: unknown; index: number }) => Promise<unknown>;

/** A coin request as text, as it travels to the page: the draft as raw hex. */
export interface CoinRequest {
  step: 'commit' | 'settle';
  amount: number;
  tx: string;
  index: number;
}

/** The page's coin as text: its transaction as atomic BEEF hex, fundd's unlocking script in hex. */
export interface Coin {
  tx: string;
  vout: number;
  unlockingScript: string;
}

export interface IdentityWallet {
  /**
   * Answer a page's /bolt/sign request (silently only under the keep-signed-in grant). `count` and
   * `holder` name the holder the app expects: the wallet signs with that key only if it holds it.
   */
  answer(o: { id?: string; domain: string; appPubKey: string; kind: SignKind; payload: string; silent: boolean; funder?: Funder; count?: number; holder?: string }): Promise<SignAnswer>;
  identities(): Promise<Identity[]>;
  forApp(o: { domain: string; appPubKey: string }): Promise<Identity[]>;
  create(): Promise<Identity>;
  /** Register: move the token from its mint to holder 1, paid by the app (`funder`), broadcast. */
  present(o: { id: string; domain: string; appPubKey: string; data: string; keepSignedIn?: boolean; funder: Funder }): Promise<Presentation>;
  setKeepSignedIn(o: { id: string; domain: string; appPubKey: string; keep: boolean }): Promise<void>;
  forget(o: { id: string; domain: string; appPubKey: string }): Promise<void>;
  /** Keep the name an app accepted for `identity` (its issuer key), only if it is linked to that app here. */
  label(o: { domain: string; appPubKey: string; identity: string; name: string }): Promise<void>;
}

/** `call(endpoint, body)` resolves with the wallet's JSON reply and throws on an error reply. */
export function identityService(call: (endpoint: string, body?: unknown) => Promise<any>): IdentityWallet;

/** A funder that asks over text (the browser asks the page, the page asks the app). */
export function textFunder(ask: (request: CoinRequest) => Promise<Coin>): Funder;

/** Each write kind's tier, from the app's published actions; any other kind is prompted. */
export const WRITE_TIERS: Readonly<Record<string, 'prompted' | 'silent'>>;

export function decodeAuthData(data: string): {
  purpose: 'register' | 'signin' | 'refresh' | 'write' | 'rotate' | 'reissue';
  appPubKey: string;
  challengeHash: string;
  /** The holder count, for register, rotate and reissue data. */
  count?: number;
};
