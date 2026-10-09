// Types for bolt-identity.js, the bundled AuthBOLT identity service (ChainBrowsers packages/bolt,
// `npm run bundle:identity`). Hodos's own identity prompt runs it; pages never do.

export interface IdentityApp {
  domain: string;
  appPubKey: string;
  keepSignedIn: boolean;
  linkedAt: number;
  /** The key this identity signs with for this app (absent: the issuer key) and its rotation count. */
  signKeyId?: string;
  signSeq?: number;
}

export interface Identity {
  id: string;
  type: 'AuthBOLT';
  issuer: string;
  keyId: string;
  holderKeyId: string;
  apps: IdentityApp[];
}

export interface Presentation {
  package: string[];
  id: string;
}

/** What the page's /bolt/sign asks: a holder-key signature, or a move of the app's signing key. */
export type SignKind = 'signin' | 'refresh' | 'write' | 'rotate' | 'confirm' | 'recover';

export interface SignAnswer {
  identity: string;
  holder?: string;
  signature?: string;
  newHolder?: string;
  seq?: number;
}

export interface IdentityWallet {
  /** Answer a page's /bolt/sign request (silently only under the keep-signed-in grant). */
  answer(o: { id?: string; domain: string; appPubKey: string; kind: SignKind; payload: string; silent: boolean }): Promise<SignAnswer>;
  identities(): Promise<Identity[]>;
  forApp(o: { domain: string; appPubKey: string }): Promise<Identity[]>;
  create(): Promise<Identity>;
  present(o: { id: string; domain: string; appPubKey: string; data: string; keepSignedIn?: boolean }): Promise<Presentation>;
  refresh(o: { domain: string; appPubKey: string; data: string }): Promise<Presentation>;
  setKeepSignedIn(o: { id: string; domain: string; appPubKey: string; keep: boolean }): Promise<void>;
  forget(o: { id: string; domain: string; appPubKey: string }): Promise<void>;
}

/** `call(endpoint, body)` resolves with the wallet's JSON reply and throws on an error reply. */
export function identityService(call: (endpoint: string, body?: unknown) => Promise<any>): IdentityWallet;

/** Each write kind's tier, from the app's published actions; any other kind is prompted. */
export const WRITE_TIERS: Readonly<Record<string, 'prompted' | 'silent'>>;

export function decodeAuthData(data: string): { purpose: 'register' | 'signin' | 'refresh'; appPubKey: string; challengeHash: string };
