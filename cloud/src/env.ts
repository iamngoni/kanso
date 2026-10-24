import type { AccountRoom } from "./account_room";
import type { NoteRoom } from "./room";

/** The Worker's bindings (docs/ACCOUNTS.md › Worker configuration). */
export interface Env {
	DB: D1Database;
	FILES: R2Bucket;
	/** One NoteRoom per shared note, keyed by share ID (docs/ACCOUNTS.md › Collaboration). */
	NOTE_ROOM: DurableObjectNamespace<NoteRoom>;
	/** One AccountRoom per account, keyed by account ID: the live approval hints (design: Kanso accounts spec, revision 3). */
	ACCOUNT_ROOM: DurableObjectNamespace<AccountRoom>;
	/** 32 bytes of base64url; `wrangler secret put AUTH_PEPPER`. Missing or malformed answers 503 where it is needed. */
	AUTH_PEPPER?: string;
	/** The write-free front filter of the auth routes, keyed by IP group; absent in unit tests. */
	AUTH_LIMITER?: RateLimit;
	/** The write-free filter of the session routes, keyed by account ID; absent in unit tests. */
	SESSION_LIMITER?: RateLimit;
	/** The write-free filter of published notes and their viewer page, keyed by IP group; absent in unit tests. */
	PUBLIC_LIMITER?: RateLimit;
	/** Rerout's API key for published notes' short links; `wrangler secret put REROUT_API_KEY`. Without it there are none. */
	REROUT_API_KEY?: string;
	/** The Rerout project the key belongs to (a plain var); a link Rerout made elsewhere is deleted and not used. */
	REROUT_PROJECT_ID?: string;
	/** The custom domain short links are made on (a plain var, `go.kansoapp.app`), added and verified in that project; without it, Rerout's own. */
	REROUT_DOMAIN?: string;
	/** Where published notes are shown (a plain var, `https://share.kansoapp.app`): short links lead there. Without it, wherever the note was published. */
	SHARE_ORIGIN?: string;
	/**
	 * Passkeys (design: Kanso accounts spec, revision 3 › Sign-in methods): the relying party's host,
	 * and a JSON array of the origins accepted in `clientDataJSON.origin`.
	 * Without both, health advertises no passkeys.
	 */
	PASSKEY_RP_ID?: string;
	PASSKEY_ORIGINS?: string;
	/** Sign in with Apple: the Services ID, team, key ID, exact callback and the `.p8` secret. */
	APPLE_CLIENT_ID?: string;
	APPLE_TEAM_ID?: string;
	APPLE_KEY_ID?: string;
	APPLE_REDIRECT_URI?: string;
	APPLE_PRIVATE_KEY?: string;
	/** Google's OAuth client (type Desktop app). Both values are needed to advertise it. */
	GOOGLE_CLIENT_ID?: string;
	GOOGLE_CLIENT_SECRET?: string;
	/** GitHub's OAuth App. The device flow needs only the client ID; the secret is for the web fallback. */
	GITHUB_CLIENT_ID?: string;
	GITHUB_CLIENT_SECRET?: string;
}
