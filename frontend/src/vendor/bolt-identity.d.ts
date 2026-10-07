// Types for bolt-identity.js, the bundled AuthBOLT identity service (ChainBrowsers packages/bolt,
// `npm run bundle:identity`). Hodos's own identity prompt runs it; pages never do.

export interface IdentityApp {
  domain: string;
  appPubKey: string;
  keepSignedIn: boolean;
  linkedAt: number;
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

export interface IdentityWallet {
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

export function decodeAuthData(data: string): { purpose: 'register' | 'signin' | 'refresh'; appPubKey: string; challengeHash: string };
