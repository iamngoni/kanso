// Protocol v2's limits, patterns and body checks (docs/ACCOUNTS.md › HTTP
// protocol). Mirrors crates/kanso-sync/src/wire.rs; the shared fixtures in
// test/fixtures/ hold the cases both sides run.

export const PROTOCOL_VERSION = 2;

/** Auth and account request bodies. */
export const MAX_AUTH_BODY = 8 * 1024;
/** Three D1 statements per document in one batch. */
export const MAX_PUSH_DOCS = 200;
export const MAX_PUSH_BODY = 4 * 1024 * 1024;
export const MIN_SEALED = 55;
/** Keeps a D1 row under 2 MB. */
export const MAX_SEALED = 1_900_000;
export const MIN_SEALED_FILE = 41;
export const MAX_SEALED_FILE = 10_485_801 + 4_096;
export const DEFAULT_CHANGES_LIMIT = 100;
export const MAX_CHANGES_LIMIT = 500;
/** Sealed text per changes page, and per push response. */
export const PAGE_SEALED_BUDGET = 8 * 1024 * 1024;

/** Charged per document row on top of its sealed text, and per stored file on top of its size. */
export const DOC_ROW_COST = 512;
export const FILE_ROW_COST = 1024;

export const MAX_ACTIVE_DEVICES = 10;
export const HOUR = 3_600_000;
export const DAY = 86_400_000;
/** Active sessions unseen this long expire. */
export const SESSION_IDLE_MS = 180 * DAY;
/** Revoked session rows are kept this long. */
export const REVOKED_KEEP_MS = 90 * DAY;
/** At most this many revoked rows per account survive a sign-in. */
export const MAX_REVOKED_ROWS = 90;

export const KDF_FLOOR = { m_kib: 1_048_576, t: 3, p: 1 } as const;
export const KDF_CEILING = { m_kib: 2_097_152, t: 8, p: 8 } as const;
export const KDF_DEFAULT = { alg: "argon2id", version: 19, m_kib: 1_048_576, t: 3, p: 4 } as const;

const WIRE_ID = /^(note|book|tag|file):[A-Za-z0-9_-]{22}$/;
const WIRE_FILE_ID = /^file:[A-Za-z0-9_-]{22}$/;
const WIRE_REV = /^[0-9a-f]{32}$/;
const ACCOUNT_ID = /^a_[0-9a-f]{32}$/;
const DEVICE_ID = /^d_[0-9a-f]{32}$/;
const HEX16 = /^[0-9a-f]{16}$/;
const SHA256_HEX = /^[0-9a-f]{64}$/;
const SALT = /^[A-Za-z0-9_-]{22}$/;
const WRAPPED = /^[A-Za-z0-9_-]{98}$/;
const SEALED_NAME = /^[A-Za-z0-9_-]{55,400}$/;
const TOKEN = /^kanso_[A-Za-z0-9_-]{43}$/;

export const isWireId = (id: string): boolean => WIRE_ID.test(id);
export const isWireFileId = (id: string): boolean => WIRE_FILE_ID.test(id);
export const isWireRev = (rev: string): boolean => WIRE_REV.test(rev);
export const isAccountId = (id: string): boolean => ACCOUNT_ID.test(id);
export const isDeviceId = (id: string): boolean => DEVICE_ID.test(id);
export const isKeyId = (id: string): boolean => HEX16.test(id);
export const isSha256Hex = (value: string): boolean => SHA256_HEX.test(value);
export const isSalt = (salt: string): boolean => SALT.test(salt);
export const isWrappedKey = (wrapped: string): boolean => WRAPPED.test(wrapped);
export const isSealedName = (name: string): boolean => SEALED_NAME.test(name);
/** `kanso_` and 32 random bytes in unpadded base64url. Anything else is never looked up. */
export const isTokenShape = (token: string): boolean => TOKEN.test(token);

// ------------------------------------------------------------------ base64url

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
const LOOKUP = new Map([...ALPHABET].map((c, i) => [c, i]));

export function base64url(bytes: Uint8Array): string {
	let out = "";
	for (let i = 0; i < bytes.length; i += 3) {
		const n = (bytes[i] << 16) | ((bytes[i + 1] ?? 0) << 8) | (bytes[i + 2] ?? 0);
		const chars = Math.min(4, Math.ceil(((bytes.length - i) * 8) / 6));
		for (let c = 0; c < chars; c++) out += ALPHABET[(n >> (18 - 6 * c)) & 63];
	}
	return out;
}

/**
 * Decodes unpadded base64url, refusing anything non-canonical: a character
 * outside the alphabet, an impossible length, or unused bits that aren't zero
 * (so re-encoding wouldn't give the input back).
 */
export function fromBase64url(text: string): Uint8Array | null {
	if (text.length % 4 === 1) return null;
	const bytes = new Uint8Array(Math.floor((text.length * 6) / 8));
	let bits = 0;
	let value = 0;
	let o = 0;
	for (const c of text) {
		const v = LOOKUP.get(c);
		if (v === undefined) return null;
		// Fewer than 8 bits are pending before each character.
		value = ((value & 0xff) << 6) | v;
		bits += 6;
		if (bits >= 8) {
			bits -= 8;
			bytes[o++] = (value >> bits) & 255;
		}
	}
	if ((value & ((1 << bits) - 1)) !== 0) return null;
	return bytes;
}

/** A 32-byte proof (auth key, recovery auth key, device proof): exactly 43 canonical characters. */
export function proof32(value: unknown): Uint8Array | null {
	if (typeof value !== "string" || value.length !== 43) return null;
	const bytes = fromBase64url(value);
	return bytes && bytes.length === 32 ? bytes : null;
}

// ---------------------------------------------------------------- body checks

export type Checked<T> = { ok: true; value: T } | { ok: false; status: number; code: string; message: string };

export const fail = (status: number, code: string, message: string): Checked<never> => ({ ok: false, status, code, message });
export const badRequest = (message: string): Checked<never> => fail(400, "bad_request", message);

export function isObject(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

export interface Kdf {
	m_kib: number;
	t: number;
	p: number;
}

/** `{"alg": "argon2id", "version": 19, "m_kib", "t", "p"}` within the production floor and ceilings. */
export function checkKdf(value: unknown): Checked<Kdf> {
	if (!isObject(value)) return badRequest("kdf must be an object.");
	const { alg, version, m_kib, t, p } = value;
	if (typeof alg !== "string" || !Number.isSafeInteger(version)) return badRequest("kdf.alg and kdf.version are required.");
	for (const n of [m_kib, t, p]) if (!Number.isSafeInteger(n)) return badRequest("kdf.m_kib, kdf.t and kdf.p must be integers.");
	if (alg !== "argon2id" || version !== 19) return fail(400, "unsupported_kdf", "Only Argon2id version 19 is supported.");
	const kdf = { m_kib: m_kib as number, t: t as number, p: p as number };
	if (
		kdf.m_kib < KDF_FLOOR.m_kib ||
		kdf.t < KDF_FLOOR.t ||
		kdf.p < KDF_FLOOR.p ||
		kdf.m_kib > KDF_CEILING.m_kib ||
		kdf.t > KDF_CEILING.t ||
		kdf.p > KDF_CEILING.p
	) {
		return fail(400, "weak_kdf", "The KDF parameters are outside the floor or ceilings.");
	}
	return { ok: true, value: kdf };
}

export interface NewDevice {
	id: string;
	sealedName: string | null;
	proof: Uint8Array;
}

/** `{"id", "sealed_name"?, "proof"}`; with `named` false, any name is ignored (sign-in sends none). */
export function checkDevice(value: unknown, named: boolean): Checked<NewDevice> {
	if (!isObject(value)) return badRequest("device must be an object.");
	const { id, sealed_name } = value;
	if (typeof id !== "string" || !isDeviceId(id)) return badRequest("device.id is not a device ID.");
	const proof = proof32(value.proof);
	if (!proof) return badRequest("device.proof must be 32 bytes of canonical base64url.");
	let sealedName: string | null = null;
	if (named && sealed_name !== undefined && sealed_name !== null) {
		if (typeof sealed_name !== "string" || !isSealedName(sealed_name)) {
			return badRequest("device.sealed_name must be 55–400 base64url characters.");
		}
		sealedName = sealed_name;
	}
	return { ok: true, value: { id, sealedName, proof } };
}

/** A validated pushed document. */
export interface PushDoc {
	id: string;
	rev: string;
	deleted: boolean;
	sealed: string;
	baseRev: string | null;
}

/**
 * Validates a `POST /v2/push` body: every field present with its type, every
 * pattern, and the sealed length. The sealed alphabet is left to D1's CHECK.
 * Any invalid document rejects the whole push.
 */
export function checkPush(body: unknown): Checked<PushDoc[]> {
	if (!isObject(body) || !Array.isArray(body.docs)) return badRequest('The body must be {"docs": [...]}.');
	if (body.docs.length > MAX_PUSH_DOCS) return fail(413, "too_large", `A push carries at most ${MAX_PUSH_DOCS} documents.`);
	const docs: PushDoc[] = [];
	for (const [i, doc] of body.docs.entries()) {
		if (!isObject(doc)) return badRequest(`docs[${i}] is not an object.`);
		const { id, rev, deleted, sealed, base_rev } = doc;
		if (typeof id !== "string" || !isWireId(id)) return badRequest(`docs[${i}].id is not a wire ID.`);
		if (typeof rev !== "string" || !isWireRev(rev)) return badRequest(`docs[${i}].rev is not a wire revision.`);
		if (typeof deleted !== "boolean") return badRequest(`docs[${i}].deleted must be a boolean.`);
		if (typeof sealed !== "string") return badRequest(`docs[${i}].sealed must be a string.`);
		if (sealed.length < MIN_SEALED) return badRequest(`docs[${i}].sealed is shorter than ${MIN_SEALED} characters.`);
		if (sealed.length > MAX_SEALED) return fail(413, "too_large", `docs[${i}].sealed is longer than ${MAX_SEALED} characters.`);
		if (base_rev === undefined) return badRequest(`docs[${i}].base_rev is required (a revision or null).`);
		if (base_rev !== null && (typeof base_rev !== "string" || !isWireRev(base_rev))) {
			return badRequest(`docs[${i}].base_rev is not a wire revision.`);
		}
		docs.push({ id, rev, deleted, sealed, baseRev: base_rev });
	}
	return { ok: true, value: docs };
}

/** A non-negative integer query parameter; `fallback` when absent, null when malformed. */
export function natParam(value: string | null, fallback: number): number | null {
	if (value === null) return fallback;
	if (!/^[0-9]{1,15}$/.test(value)) return null;
	return Number(value);
}
