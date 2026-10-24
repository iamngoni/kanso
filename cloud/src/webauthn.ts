// WebAuthn checks for passkeys and security keys (docs/ACCOUNTS.md › Passkeys).
//
// The Worker is the relying party. It reads only what it needs: the
// attestation object's authenticator data (never its statement or a
// certificate chain: attestation is "none"), the COSE public key inside it,
// and an assertion's authenticator data, client data and signature. The same
// checks serve a platform passkey (macOS, Windows) and a Linux security key,
// whose app builds the identical WebAuthn messages from CTAP2.
//
// Nothing here touches D1; the routes in passkeys.ts store and look up.

import { sha256 } from "./crypto";
import { base64url, fromBase64url } from "./protocol";

/** ES256, EdDSA (Ed25519) and RS256, in the order the registration options list them. */
export const COSE_ALGORITHMS = [-7, -8, -257] as const;
export type CoseAlgorithm = (typeof COSE_ALGORITHMS)[number];

/** Raw credential IDs are 16 to 1,023 bytes (the column holds 22 to 1,364 base64url characters). */
export const MIN_CREDENTIAL_ID = 16;
export const MAX_CREDENTIAL_ID = 1023;

const FLAG_UP = 0x01;
const FLAG_UV = 0x04;
const FLAG_AT = 0x40;
const FLAG_ED = 0x80;

// ---------------------------------------------------------------------- CBOR

export type Cbor = number | bigint | Uint8Array | string | boolean | null | undefined | Cbor[] | CborMap;
export type CborMap = Map<number | string, Cbor>;

class CborError extends Error {}

const MAX_DEPTH = 16;

/**
 * Decodes one CBOR item starting at `at`, as CTAP2 and WebAuthn send it:
 * definite lengths only, map keys that are integers or text, no tags and no
 * floats. Returns the item and the offset just past it.
 */
export function decodeCbor(bytes: Uint8Array, at = 0, depth = 0): { value: Cbor; end: number } {
	if (depth > MAX_DEPTH) throw new CborError("too deep");
	if (at >= bytes.length) throw new CborError("truncated");
	const initial = bytes[at];
	const major = initial >> 5;
	const info = initial & 0x1f;
	let offset = at + 1;
	const need = (n: number) => {
		if (offset + n > bytes.length) throw new CborError("truncated");
	};
	let argument: number;
	if (info < 24) {
		argument = info;
	} else if (info === 24) {
		need(1);
		argument = bytes[offset];
		offset += 1;
	} else if (info === 25) {
		need(2);
		argument = (bytes[offset] << 8) | bytes[offset + 1];
		offset += 2;
	} else if (info === 26) {
		need(4);
		argument = ((bytes[offset] << 24) >>> 0) + (bytes[offset + 1] << 16) + (bytes[offset + 2] << 8) + bytes[offset + 3];
		offset += 4;
	} else if (info === 27) {
		need(8);
		const view = new DataView(bytes.buffer, bytes.byteOffset + offset, 8);
		const big = view.getBigUint64(0);
		if (big > BigInt(Number.MAX_SAFE_INTEGER)) throw new CborError("integer too large");
		argument = Number(big);
		offset += 8;
	} else {
		throw new CborError("indefinite lengths are not accepted");
	}
	switch (major) {
		case 0:
			return { value: argument, end: offset };
		case 1:
			return { value: -1 - argument, end: offset };
		case 2:
			need(argument);
			return { value: bytes.slice(offset, offset + argument), end: offset + argument };
		case 3: {
			need(argument);
			const text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: false }).decode(bytes.subarray(offset, offset + argument));
			return { value: text, end: offset + argument };
		}
		case 4: {
			const items: Cbor[] = [];
			for (let i = 0; i < argument; i++) {
				const item = decodeCbor(bytes, offset, depth + 1);
				items.push(item.value);
				offset = item.end;
			}
			return { value: items, end: offset };
		}
		case 5: {
			const map: CborMap = new Map();
			for (let i = 0; i < argument; i++) {
				const key = decodeCbor(bytes, offset, depth + 1);
				if (typeof key.value !== "number" && typeof key.value !== "string") throw new CborError("map key");
				if (map.has(key.value)) throw new CborError("duplicate map key");
				const value = decodeCbor(bytes, key.end, depth + 1);
				map.set(key.value, value.value);
				offset = value.end;
			}
			return { value: map, end: offset };
		}
		case 7:
			if (info === 20) return { value: false, end: offset };
			if (info === 21) return { value: true, end: offset };
			if (info === 22) return { value: null, end: offset };
			throw new CborError("simple value or float");
		default:
			throw new CborError("tags are not accepted");
	}
}

/** The whole of `bytes` as one CBOR item, or null when it isn't exactly one. */
export function decodeCborExactly(bytes: Uint8Array): Cbor | null {
	try {
		const { value, end } = decodeCbor(bytes);
		return end === bytes.length ? value : null;
	} catch {
		return null;
	}
}

// -------------------------------------------------------- authenticator data

export interface AuthenticatorData {
	rpIdHash: Uint8Array;
	flags: number;
	signCount: number;
	/** Present when the AT flag is set (registration). */
	attested: { aaguid: Uint8Array; credentialId: Uint8Array; publicKey: Uint8Array } | null;
}

/**
 * Parses authenticator data (WebAuthn §6.1): the RP ID hash, flags, the
 * signature counter, then the attested credential data when AT is set and the
 * extensions map when ED is set. Every byte must be accounted for.
 */
export function parseAuthenticatorData(bytes: Uint8Array): AuthenticatorData | null {
	if (bytes.length < 37) return null;
	const flags = bytes[32];
	const signCount = new DataView(bytes.buffer, bytes.byteOffset + 33, 4).getUint32(0);
	let at = 37;
	let attested: AuthenticatorData["attested"] = null;
	try {
		if (flags & FLAG_AT) {
			if (bytes.length < at + 18) return null;
			const aaguid = bytes.slice(at, at + 16);
			const length = (bytes[at + 16] << 8) | bytes[at + 17];
			at += 18;
			if (length < MIN_CREDENTIAL_ID || length > MAX_CREDENTIAL_ID || bytes.length < at + length) return null;
			const credentialId = bytes.slice(at, at + length);
			at += length;
			const key = decodeCbor(bytes, at);
			if (!(key.value instanceof Map)) return null;
			attested = { aaguid, credentialId, publicKey: bytes.slice(at, key.end) };
			at = key.end;
		}
		if (flags & FLAG_ED) {
			const extensions = decodeCbor(bytes, at);
			if (!(extensions.value instanceof Map)) return null;
			at = extensions.end;
		}
	} catch {
		return null;
	}
	if (at !== bytes.length) return null;
	return { rpIdHash: bytes.slice(0, 32), flags, signCount, attested };
}

// ---------------------------------------------------------------- COSE keys

/** The algorithm a COSE public key declares, when it is one Kanso accepts and well formed. */
export function coseAlgorithm(cose: Uint8Array): CoseAlgorithm | null {
	const key = decodeCborExactly(cose);
	if (!(key instanceof Map)) return null;
	const alg = key.get(3);
	const kty = key.get(1);
	const bytesOf = (label: number, length?: number): boolean => {
		const value = key.get(label);
		return value instanceof Uint8Array && (length === undefined || value.length === length);
	};
	if (alg === -7) return kty === 2 && key.get(-1) === 1 && bytesOf(-2, 32) && bytesOf(-3, 32) ? -7 : null;
	if (alg === -8) return kty === 1 && key.get(-1) === 6 && bytesOf(-2, 32) ? -8 : null;
	if (alg === -257) {
		const n = key.get(-1);
		// At least 2048 bits, as WebCrypto and every current authenticator use.
		return kty === 3 && n instanceof Uint8Array && n.length >= 256 && n.length <= 512 && bytesOf(-2) ? -257 : null;
	}
	return null;
}

async function importCose(cose: Uint8Array, alg: CoseAlgorithm): Promise<CryptoKey | null> {
	const key = decodeCborExactly(cose);
	if (!(key instanceof Map) || coseAlgorithm(cose) !== alg) return null;
	const b = (label: number) => base64url(key.get(label) as Uint8Array);
	try {
		if (alg === -7) {
			return await crypto.subtle.importKey("jwk", { kty: "EC", crv: "P-256", x: b(-2), y: b(-3) }, { name: "ECDSA", namedCurve: "P-256" }, false, [
				"verify",
			]);
		}
		if (alg === -8) {
			return await crypto.subtle.importKey("raw", key.get(-2) as Uint8Array, { name: "Ed25519" }, false, ["verify"]);
		}
		return await crypto.subtle.importKey(
			"jwk",
			{ kty: "RSA", n: b(-1), e: b(-2), alg: "RS256" },
			{ name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" },
			false,
			["verify"],
		);
	} catch {
		return null;
	}
}

/** An ASN.1 DER ECDSA signature as the 64-byte `r || s` WebCrypto verifies, or null. */
export function derToRaw(der: Uint8Array): Uint8Array | null {
	// SEQUENCE { INTEGER r, INTEGER s }, short lengths only (P-256 fits).
	if (der.length < 8 || der[0] !== 0x30 || der[1] !== der.length - 2) return null;
	const out = new Uint8Array(64);
	let at = 2;
	for (let part = 0; part < 2; part++) {
		if (der[at] !== 0x02) return null;
		const length = der[at + 1];
		if (length < 1 || length > 33 || at + 2 + length > der.length) return null;
		let value = der.subarray(at + 2, at + 2 + length);
		// A leading zero only when the next byte's top bit is set (DER's minimal form).
		if (value.length > 1 && value[0] === 0 && (value[1] & 0x80) === 0) return null;
		if (value[0] & 0x80) return null;
		if (value.length === 33) {
			if (value[0] !== 0) return null;
			value = value.subarray(1);
		}
		out.set(value, part * 32 + (32 - value.length));
		at += 2 + length;
	}
	return at === der.length ? out : null;
}

/** Whether `signature` by the COSE key verifies over `data`. */
export async function verifySignature(cose: Uint8Array, alg: CoseAlgorithm, data: Uint8Array, signature: Uint8Array): Promise<boolean> {
	const key = await importCose(cose, alg);
	if (!key) return false;
	try {
		if (alg === -7) {
			const raw = derToRaw(signature);
			return raw !== null && (await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, key, raw, data));
		}
		if (alg === -8) return await crypto.subtle.verify({ name: "Ed25519" }, key, signature, data);
		return await crypto.subtle.verify({ name: "RSASSA-PKCS1-v1_5" }, key, signature, data);
	} catch {
		return false;
	}
}

// ---------------------------------------------------------------- ceremonies

/** What a ceremony must match: the relying party, the origins it accepts and the flow's challenge. */
export interface Expected {
	rpId: string;
	origins: readonly string[];
	/** 43 base64url characters, exactly as the start answered it. */
	challenge: string;
}

/**
 * Checks `clientDataJSON` (WebAuthn §7.1 steps 5–10, §7.2 steps 10–14): its
 * type, the challenge, an accepted origin, and no cross-origin use.
 */
function checkClientData(clientDataJson: Uint8Array, type: "webauthn.create" | "webauthn.get", expected: Expected): boolean {
	let data: unknown;
	try {
		data = JSON.parse(new TextDecoder("utf-8", { fatal: true, ignoreBOM: false }).decode(clientDataJson));
	} catch {
		return false;
	}
	if (typeof data !== "object" || data === null || Array.isArray(data)) return false;
	const d = data as Record<string, unknown>;
	return (
		d.type === type &&
		d.challenge === expected.challenge &&
		typeof d.origin === "string" &&
		expected.origins.includes(d.origin) &&
		(d.crossOrigin === undefined || d.crossOrigin === false)
	);
}

function equalBytes(a: Uint8Array, b: Uint8Array): boolean {
	if (a.length !== b.length) return false;
	let difference = 0;
	for (let i = 0; i < a.length; i++) difference |= a[i] ^ b[i];
	return difference === 0;
}

/** The RP ID hash, user presence and user verification every ceremony needs. */
async function checkAuthenticator(data: AuthenticatorData, rpId: string): Promise<boolean> {
	const required = FLAG_UP | FLAG_UV;
	return equalBytes(data.rpIdHash, await sha256(rpId)) && (data.flags & required) === required;
}

export interface Registration {
	credentialId: Uint8Array;
	publicKey: Uint8Array;
	alg: CoseAlgorithm;
	signCount: number;
	aaguid: Uint8Array;
}

/**
 * Verifies a registration (WebAuthn §7.1) with attestation "none": the client
 * data, then the attestation object's authenticator data: RP ID hash, UP and
 * UV, attested credential data whose ID is the one the client named, and a
 * public key in an accepted algorithm. The attestation statement is not read.
 */
export async function verifyRegistration(
	input: { credentialId: Uint8Array; attestationObject: Uint8Array; clientDataJson: Uint8Array },
	expected: Expected,
): Promise<Registration | null> {
	if (!checkClientData(input.clientDataJson, "webauthn.create", expected)) return null;
	const object = decodeCborExactly(input.attestationObject);
	// Attestation is "none" (the start asks for it); the statement is never
	// read, so no certificate chain is validated.
	if (!(object instanceof Map) || object.get("fmt") !== "none") return null;
	const attStmt = object.get("attStmt");
	if (!(attStmt instanceof Map) || attStmt.size !== 0) return null;
	const authData = object.get("authData");
	if (!(authData instanceof Uint8Array)) return null;
	const data = parseAuthenticatorData(authData);
	if (!data?.attested || !(await checkAuthenticator(data, expected.rpId))) return null;
	if (!equalBytes(data.attested.credentialId, input.credentialId)) return null;
	const alg = coseAlgorithm(data.attested.publicKey);
	if (alg === null || (await importCose(data.attested.publicKey, alg)) === null) return null;
	return { credentialId: data.attested.credentialId, publicKey: data.attested.publicKey, alg, signCount: data.signCount, aaguid: data.attested.aaguid };
}

/**
 * Verifies an assertion (WebAuthn §7.2) against a stored credential: client
 * data, RP ID hash, UP and UV, then the signature over
 * `authenticatorData || SHA-256(clientDataJSON)`. The caller checks the sign
 * count against the stored one. Returns the new count, or null.
 */
export async function verifyAssertion(
	input: { authenticatorData: Uint8Array; clientDataJson: Uint8Array; signature: Uint8Array },
	credential: { publicKey: Uint8Array; alg: CoseAlgorithm },
	expected: Expected,
): Promise<{ signCount: number } | null> {
	if (!checkClientData(input.clientDataJson, "webauthn.get", expected)) return null;
	const data = parseAuthenticatorData(input.authenticatorData);
	if (!data || data.attested || !(await checkAuthenticator(data, expected.rpId))) return null;
	const signed = new Uint8Array(input.authenticatorData.length + 32);
	signed.set(input.authenticatorData, 0);
	signed.set(await sha256(input.clientDataJson), input.authenticatorData.length);
	if (!(await verifySignature(credential.publicKey, credential.alg, signed, input.signature))) return null;
	return { signCount: data.signCount };
}

/**
 * The sign-count rule (docs/ACCOUNTS.md › Passkeys › Sign count): a stored
 * count of 0 means the authenticator doesn't count and is never enforced;
 * when both are above 0, a new count at or below the stored one is a possible
 * clone and is refused.
 */
export const signCountAccepted = (stored: number, next: number): boolean => stored === 0 || next === 0 || next > stored;

/** The WebAuthn user handle: `SHA-256("kanso/v1/webauthn-user" || lp(account_id))[0..16]`, base64url. */
export async function userHandle(accountId: string): Promise<string> {
	const encoder = new TextEncoder();
	const label = encoder.encode("kanso/v1/webauthn-user");
	const id = encoder.encode(accountId);
	const input = new Uint8Array(label.length + 2 + id.length);
	input.set(label, 0);
	input[label.length] = id.length >> 8;
	input[label.length + 1] = id.length & 0xff;
	input.set(id, label.length + 2);
	return base64url((await sha256(input)).slice(0, 16));
}

/** A base64url field of at least `min` and at most `max` decoded bytes, canonical, or null. */
export function bytesField(value: unknown, min: number, max: number): Uint8Array | null {
	if (typeof value !== "string" || value.length === 0 || value.length > Math.ceil((max * 4) / 3)) return null;
	const bytes = fromBase64url(value);
	return bytes && bytes.length >= min && bytes.length <= max ? bytes : null;
}
