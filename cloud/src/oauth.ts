// Provider token verification (design: Kanso accounts spec, revision 3).

import { sha256, sha256Hex } from "./crypto";
import type { Env } from "./env";
import { base64url, fromBase64url } from "./protocol";

export const APPLE_ISSUER = "https://appleid.apple.com";
export const APPLE_JWKS_URL = "https://appleid.apple.com/auth/keys";
export const APPLE_TOKEN_URL = "https://appleid.apple.com/auth/token";
export const GOOGLE_ISSUERS: readonly string[] = ["https://accounts.google.com", "accounts.google.com"];
export const GOOGLE_JWKS_URL = "https://www.googleapis.com/oauth2/v3/certs";
export const GOOGLE_TOKEN_URL = "https://oauth2.googleapis.com/token";
export const GITHUB_DEVICE_CODE_URL = "https://github.com/login/device/code";
export const GITHUB_TOKEN_URL = "https://github.com/login/oauth/access_token";
export const GITHUB_USER_URL = "https://api.github.com/user";

/** The algorithms each provider's ID token may use (docs/ACCOUNTS.md › Common rules). ES256 is reserved for Apple's client secret. */
export const APPLE_ID_TOKEN_ALGORITHMS = ["RS256"] as const;
export const GOOGLE_ID_TOKEN_ALGORITHMS = ["RS256"] as const;

const REQUEST_TIMEOUT_MS = 10_000;
const JWKS_TTL_MS = 60 * 60 * 1000;
/** The shortest gap between two refetches for an unknown `kid`. */
const JWKS_MISS_REFETCH_MS = 5 * 60 * 1000;
const APPLE_SECRET_MAX_AGE_MS = 23 * 60 * 60 * 1000;
const DAY_SECONDS = 24 * 60 * 60;
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: false });

export type TokenFailure = "invalid_credentials" | "nonce_reused" | "provider_unavailable";
export type TokenResult = { ok: true; claims: Record<string, unknown> } | { ok: false; error: TokenFailure };

export interface VerifyOptions {
	jwks_url: string;
	algorithms: readonly ("RS256" | "ES256")[];
	issuers: readonly string[];
	audience: string;
	/**
	 * SHA-256 hex of the nonce the flow sent the provider, as `auth_flows.nonce_hash`
	 * stores it. Required: Apple and Google tokens are bound to their flow, and a
	 * token without the nonce, or with another one, is refused.
	 */
	nonce_hash: string;
	now: number;
}

/** A configuration or provider failure with a message safe for route responses. */
export class ProviderUnavailable extends Error {
	constructor() {
		super("Provider unavailable.");
		this.name = "ProviderUnavailable";
	}
}

type JsonObject = Record<string, unknown>;
type Jwk = JsonObject;

interface JwksCacheEntry {
	expires_at: number;
	keys: Jwk[];
	/**
	 * When the provider was last asked again for an unknown `kid`, or null. Another
	 * miss within JWKS_MISS_REFETCH_MS is refused without a request; after it, a key
	 * the provider has published since is found.
	 */
	forced_at: number | null;
}

const jwksCache = new Map<string, JwksCacheEntry>();

interface AppleSecretCacheEntry {
	config_hash: string;
	minted_at: number;
	secret: string;
}

let appleSecretCache: AppleSecretCacheEntry | null = null;

const isObject = (value: unknown): value is JsonObject => typeof value === "object" && value !== null && !Array.isArray(value);

const unavailable = (): { ok: false; error: "provider_unavailable" } => ({ ok: false, error: "provider_unavailable" });
const invalidCredentials = (): { ok: false; error: "invalid_credentials" } => ({ ok: false, error: "invalid_credentials" });

/** Test seam: forget every cached JWKS and Apple client secret. */
export function resetProviderCache(): void {
	jwksCache.clear();
	resetAppleSecretCache();
}

/** Test seam for the separately exported Apple-secret cache reset. */
export function resetAppleSecretCache(): void {
	appleSecretCache = null;
}

function decodeCanonicalBase64url(encoded: string): Uint8Array | null {
	if (encoded.length === 0) return null;
	const decoded = fromBase64url(encoded);
	if (!decoded || decoded.length === 0 || base64url(decoded) !== encoded) return null;
	return decoded;
}

function decodeJsonSegment(encoded: string): JsonObject | null {
	const bytes = decodeCanonicalBase64url(encoded);
	if (!bytes) return null;
	try {
		const parsed: unknown = JSON.parse(textDecoder.decode(bytes));
		return isObject(parsed) ? parsed : null;
	} catch {
		return null;
	}
}

async function fetchJwks(url: string, force = false): Promise<JwksCacheEntry> {
	const now = Date.now();
	const cached = jwksCache.get(url);
	const fresh = cached !== undefined && cached.expires_at > now;
	if (fresh) {
		// A fresh entry answers normally. A forced read is the unknown-`kid`
		// refetch, rate-limited so a stream of bogus `kid`s can't hammer the provider.
		if (!force) return cached;
		if (cached.forced_at !== null && now - cached.forced_at < JWKS_MISS_REFETCH_MS) return cached;
	}

	let response: Response;
	try {
		response = await fetch(url, {
			method: "GET",
			headers: { accept: "application/json" },
			redirect: "error",
			signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
		});
	} catch {
		throw new ProviderUnavailable();
	}
	if (!response.ok) throw new ProviderUnavailable();

	let body: unknown;
	try {
		body = await response.json();
	} catch {
		throw new ProviderUnavailable();
	}
	if (!isObject(body) || !Array.isArray(body.keys)) throw new ProviderUnavailable();
	const keys = body.keys.filter(isObject);
	if (keys.length === 0) throw new ProviderUnavailable();
	// A forced refetch keeps the window it replaced, so a miss can't stretch the cache.
	const entry: JwksCacheEntry = {
		expires_at: force && fresh ? cached.expires_at : now + JWKS_TTL_MS,
		keys,
		forced_at: force ? now : null,
	};
	jwksCache.set(url, entry);
	return entry;
}

async function keyForKid(url: string, kid: string): Promise<Jwk | null> {
	const first = await fetchJwks(url);
	const firstMatch = first.keys.find((key) => key.kid === kid);
	if (firstMatch) return firstMatch;
	// A rotated provider key gets a fresh JWKS read, at most one every
	// JWKS_MISS_REFETCH_MS; within that, fetchJwks answers from the cache.
	const refreshed = await fetchJwks(url, true);
	return refreshed.keys.find((key) => key.kid === kid) ?? null;
}

function derInteger(value: Uint8Array): Uint8Array {
	let first = 0;
	while (first < value.length - 1 && value[first] === 0) first++;
	const needsPadding = (value[first] & 0x80) !== 0;
	const length = value.length - first + (needsPadding ? 1 : 0);
	const encoded = new Uint8Array(length + 2);
	encoded[0] = 0x02;
	encoded[1] = length;
	if (needsPadding) encoded[2] = 0;
	encoded.set(value.subarray(first), needsPadding ? 3 : 2);
	return encoded;
}

/** Convert JOSE's fixed-width r||s ES256 signature to ASN.1 DER. */
function joseToDer(signature: Uint8Array): Uint8Array | null {
	if (signature.length !== 64) return null;
	const r = derInteger(signature.subarray(0, 32));
	const s = derInteger(signature.subarray(32));
	const sequenceLength = r.length + s.length;
	const der = new Uint8Array(sequenceLength + 2);
	der[0] = 0x30;
	der[1] = sequenceLength;
	der.set(r, 2);
	der.set(s, 2 + r.length);
	return der;
}

async function verifySignature(alg: "RS256" | "ES256", jwk: Jwk, data: Uint8Array, signature: Uint8Array): Promise<boolean> {
	if (alg === "RS256") {
		if (jwk.kty !== "RSA" || typeof jwk.n !== "string" || typeof jwk.e !== "string") throw new ProviderUnavailable();
		const keyData: JsonWebKey = { kty: "RSA", n: jwk.n, e: jwk.e, alg: "RS256", ext: true };
		let key: CryptoKey;
		try {
			key = await crypto.subtle.importKey(
				"jwk",
				keyData,
				{ name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" },
				false,
				["verify"],
			);
		} catch {
			throw new ProviderUnavailable();
		}
		try {
			return await crypto.subtle.verify("RSASSA-PKCS1-v1_5", key, signature, data);
		} catch {
			return false;
		}
	}

	if (jwk.kty !== "EC" || jwk.crv !== "P-256" || typeof jwk.x !== "string" || typeof jwk.y !== "string") {
		throw new ProviderUnavailable();
	}
	const keyData: JsonWebKey = { kty: "EC", crv: "P-256", x: jwk.x, y: jwk.y, ext: true };
	let key: CryptoKey;
	try {
		key = await crypto.subtle.importKey("jwk", keyData, { name: "ECDSA", namedCurve: "P-256" }, false, ["verify"]);
	} catch {
		throw new ProviderUnavailable();
	}
	const der = joseToDer(signature);
	if (!der) return false;
	try {
		// Convert the JWT signature as required by DER-based Web Crypto implementations.
		if (await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, key, der, data)) return true;
	} catch {
		// Standard Web Crypto and Workers accept the JOSE fixed-width form directly.
	}
	try {
		return await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, key, signature, data);
	} catch {
		return false;
	}
}

function validClaims(claims: JsonObject, options: VerifyOptions): boolean {
	if (typeof claims.iss !== "string" || !options.issuers.includes(claims.iss)) return false;
	const audience = claims.aud;
	if (typeof audience === "string") {
		if (audience !== options.audience) return false;
	} else if (!Array.isArray(audience) || !audience.includes(options.audience) || claims.azp !== options.audience) {
		// A token for several audiences counts only when this client is its authorized party.
		return false;
	}
	if (typeof claims.exp !== "number" || !Number.isFinite(claims.exp)) return false;
	if (typeof claims.iat !== "number" || !Number.isFinite(claims.iat)) return false;
	const nowSeconds = options.now / 1000;
	if (claims.exp < nowSeconds - 120 || claims.iat > nowSeconds + 120) return false;
	if (claims.nbf !== undefined && (typeof claims.nbf !== "number" || claims.nbf > nowSeconds + 120)) return false;
	return typeof claims.sub === "string" && /^[\x20-\x7e]{1,255}$/.test(claims.sub);
}

async function constantTimeTextEqual(left: string, right: string): Promise<boolean> {
	const leftDigest = await sha256(left);
	const rightDigest = await sha256(right);
	let difference = 0;
	for (let index = 0; index < leftDigest.length; index++) difference |= leftDigest[index] ^ rightDigest[index];
	return difference === 0;
}

/**
 * Verifies a provider's ID token. This is pure and stateless: it reports the
 * same answer for the same token twice, on purpose. One-time use belongs to
 * the flow — the route consumes the stored flow atomically before it issues a
 * session, so a replayed token meets a consumed flow and is refused
 * (`nonce_reused`). Never reuse this result across requests.
 */
export async function verifyIdToken(token: string, options: VerifyOptions): Promise<TokenResult> {
	const parts = token.split(".");
	if (parts.length !== 3) return invalidCredentials();
	const [encodedHeader, encodedClaims, encodedSignature] = parts;
	const header = decodeJsonSegment(encodedHeader);
	const claims = decodeJsonSegment(encodedClaims);
	const signature = decodeCanonicalBase64url(encodedSignature);
	if (!header || !claims || !signature) return invalidCredentials();

	const alg = header.alg;
	// Algorithm allowlisting precedes key fetching or any cryptographic operation.
	if (alg !== "RS256" && alg !== "ES256") return invalidCredentials();
	if (!options.algorithms.includes(alg)) return invalidCredentials();
	// No extension this verifier understands may be marked critical.
	if (header.crit !== undefined) return invalidCredentials();
	// `typ` is optional; when supplied, require the conventional JWT value.
	if (header.typ !== undefined && header.typ !== "JWT") return invalidCredentials();
	if (typeof header.kid !== "string" || header.kid.length === 0) return invalidCredentials();
	if (!Number.isFinite(options.now) || typeof options.audience !== "string" || options.audience.length === 0) return invalidCredentials();
	if (typeof options.nonce_hash !== "string" || !/^[0-9a-f]{64}$/.test(options.nonce_hash)) return invalidCredentials();
	if (!validClaims(claims, options)) return invalidCredentials();

	let jwk: Jwk | null;
	try {
		jwk = await keyForKid(options.jwks_url, header.kid);
	} catch (error) {
		if (error instanceof ProviderUnavailable) return unavailable();
		return unavailable();
	}
	if (!jwk) return invalidCredentials();

	let verified = false;
	try {
		verified = await verifySignature(alg, jwk, textEncoder.encode(`${encodedHeader}.${encodedClaims}`), signature);
	} catch (error) {
		if (error instanceof ProviderUnavailable) return unavailable();
		return invalidCredentials();
	}
	if (!verified) return invalidCredentials();
	// The token must carry the flow's nonce. A missing or different one is a
	// token from another flow: invalid, not "reused" (that is a consumed flow).
	if (typeof claims.nonce !== "string" || !(await constantTimeTextEqual(await sha256Hex(claims.nonce), options.nonce_hash))) {
		return invalidCredentials();
	}
	return { ok: true, claims };
}

/** `base64url(SHA-256(verifier))`, the S256 PKCE challenge. */
export async function pkceChallenge(verifier: string): Promise<string> {
	return base64url(await sha256(verifier));
}

/**
 * `auth_flows.pkce_hash` for an S256 challenge: the same 32 bytes, as lowercase hex.
 * Null for anything that isn't a canonical base64url SHA-256.
 */
export function pkceHash(challenge: string): string | null {
	const decoded = decodeCanonicalBase64url(challenge);
	if (!decoded || decoded.length !== 32) return null;
	return Array.from(decoded, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** Whether `verifier` belongs to the stored `pkce_hash`, compared in constant time. */
export async function pkceMatches(verifier: string, storedHash: string): Promise<boolean> {
	if (!/^[0-9a-f]{64}$/.test(storedHash)) return false;
	return constantTimeTextEqual(await sha256Hex(verifier), storedHash);
}

function required(value: string | undefined): value is string {
	return typeof value === "string" && value.trim().length > 0;
}

function pemPkcs8Bytes(pem: string): Uint8Array {
	const normalized = pem.replace(/\\n/g, "\n").trim();
	const match = /^-----BEGIN PRIVATE KEY-----([\s\S]+)-----END PRIVATE KEY-----$/.exec(normalized);
	if (!match) throw new ProviderUnavailable();
	const encoded = match[1].replace(/\s/g, "");
	if (!/^[A-Za-z0-9+/]+={0,2}$/.test(encoded)) throw new ProviderUnavailable();
	let binary: string;
	try {
		binary = atob(encoded);
	} catch {
		throw new ProviderUnavailable();
	}
	const bytes = new Uint8Array(binary.length);
	for (let index = 0; index < binary.length; index++) bytes[index] = binary.charCodeAt(index);
	if (bytes.length === 0) throw new ProviderUnavailable();
	return bytes;
}

/** Apple's ES256 client secret, cached for no longer than 23 hours per configuration. */
export async function appleClientSecret(env: Env, now: number): Promise<string> {
	if (
		!required(env.APPLE_CLIENT_ID) ||
		!required(env.APPLE_TEAM_ID) ||
		!required(env.APPLE_KEY_ID) ||
		!required(env.APPLE_REDIRECT_URI) ||
		!required(env.APPLE_PRIVATE_KEY) ||
		!Number.isFinite(now)
	) {
		throw new ProviderUnavailable();
	}
	const config = JSON.stringify([
		env.APPLE_CLIENT_ID,
		env.APPLE_TEAM_ID,
		env.APPLE_KEY_ID,
		env.APPLE_REDIRECT_URI,
		env.APPLE_PRIVATE_KEY,
	]);
	const configHash = base64url(await sha256(config));
	if (
		appleSecretCache?.config_hash === configHash &&
		now >= appleSecretCache.minted_at &&
		now - appleSecretCache.minted_at <= APPLE_SECRET_MAX_AGE_MS
	) {
		return appleSecretCache.secret;
	}

	const issuedAt = Math.floor(now / 1000);
	const encodedHeader = base64url(textEncoder.encode(JSON.stringify({ alg: "ES256", kid: env.APPLE_KEY_ID })));
	const encodedClaims = base64url(
		textEncoder.encode(
			JSON.stringify({
				iss: env.APPLE_TEAM_ID,
				iat: issuedAt,
				exp: issuedAt + DAY_SECONDS,
				aud: APPLE_ISSUER,
				sub: env.APPLE_CLIENT_ID,
			}),
		),
	);
	let key: CryptoKey;
	try {
		key = await crypto.subtle.importKey("pkcs8", pemPkcs8Bytes(env.APPLE_PRIVATE_KEY), { name: "ECDSA", namedCurve: "P-256" }, false, ["sign"]);
	} catch {
		throw new ProviderUnavailable();
	}
	let signature: ArrayBuffer;
	try {
		signature = await crypto.subtle.sign(
			{ name: "ECDSA", hash: "SHA-256" },
			key,
			textEncoder.encode(`${encodedHeader}.${encodedClaims}`),
		);
	} catch {
		throw new ProviderUnavailable();
	}
	const secret = `${encodedHeader}.${encodedClaims}.${base64url(new Uint8Array(signature))}`;
	appleSecretCache = { config_hash: configHash, minted_at: now, secret };
	return secret;
}

async function exchangeToken(url: string, form: URLSearchParams): Promise<{ ok: true; id_token: string } | { ok: false; error: TokenFailure }> {
	let response: Response;
	try {
		response = await fetch(url, {
			method: "POST",
			headers: { accept: "application/json", "content-type": "application/x-www-form-urlencoded" },
			body: form.toString(),
			redirect: "error",
			signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
		});
	} catch {
		return unavailable();
	}
	let body: unknown;
	try {
		body = await response.json();
	} catch {
		return unavailable();
	}
	if (!response.ok) {
		return isObject(body) && body.error === "invalid_grant" ? invalidCredentials() : unavailable();
	}
	if (!isObject(body) || typeof body.id_token !== "string" || body.id_token.length === 0) return unavailable();
	return { ok: true, id_token: body.id_token };
}

/** Exchange an Apple authorization code and return its ID token. */
export async function exchangeAppleCode(
	env: Env,
	code: string,
	now: number,
	redirect_uri: string,
): Promise<{ ok: true; id_token: string } | { ok: false; error: TokenFailure }> {
	if (!required(env.APPLE_CLIENT_ID) || !required(redirect_uri)) return unavailable();
	let clientSecret: string;
	try {
		clientSecret = await appleClientSecret(env, now);
	} catch {
		return unavailable();
	}
	const form = new URLSearchParams({
		client_id: env.APPLE_CLIENT_ID,
		client_secret: clientSecret,
		code,
		grant_type: "authorization_code",
		redirect_uri,
	});
	return exchangeToken(APPLE_TOKEN_URL, form);
}

/** Exchange a Google authorization code. Loopback callers supply their redirect URI. */
export async function exchangeGoogleCode(
	env: Env,
	code: string,
	codeVerifier: string,
	redirect_uri?: string,
): Promise<{ ok: true; id_token: string } | { ok: false; error: TokenFailure }> {
	if (!required(env.GOOGLE_CLIENT_ID) || !required(env.GOOGLE_CLIENT_SECRET)) return unavailable();
	const form = new URLSearchParams({
		code,
		client_id: env.GOOGLE_CLIENT_ID,
		client_secret: env.GOOGLE_CLIENT_SECRET,
		grant_type: "authorization_code",
		code_verifier: codeVerifier,
	});
	if (redirect_uri !== undefined) form.set("redirect_uri", redirect_uri);
	return exchangeToken(GOOGLE_TOKEN_URL, form);
}

type GithubTokenResult =
	| { ok: true; token: string }
	| { ok: false; error: "pending" | "slow_down" | "flow_expired" | "provider_denied" | "provider_unavailable" };

/** One GitHub token poll per call; the flow caller enforces the provider's stored interval. */
export async function githubDeviceToken(env: Env, deviceCode: string): Promise<GithubTokenResult> {
	if (!required(env.GITHUB_CLIENT_ID)) return unavailable();
	let response: Response;
	try {
		response = await fetch(GITHUB_TOKEN_URL, {
			method: "POST",
			headers: { accept: "application/json", "content-type": "application/x-www-form-urlencoded" },
			body: new URLSearchParams({
				client_id: env.GITHUB_CLIENT_ID,
				device_code: deviceCode,
				grant_type: "urn:ietf:params:oauth:grant-type:device_code",
			}).toString(),
			redirect: "error",
			signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
		});
	} catch {
		return unavailable();
	}
	let body: unknown;
	try {
		body = await response.json();
	} catch {
		return unavailable();
	}
	if (!isObject(body)) return unavailable();
	if (typeof body.error === "string") {
		switch (body.error) {
			case "authorization_pending":
				return { ok: false, error: "pending" };
			case "slow_down":
				return { ok: false, error: "slow_down" };
			case "expired_token":
				return { ok: false, error: "flow_expired" };
			case "access_denied":
				return { ok: false, error: "provider_denied" };
			default:
				return unavailable();
		}
	}
	if (!response.ok || typeof body.access_token !== "string" || body.access_token.length === 0) return unavailable();
	return { ok: true, token: body.access_token };
}

/** Read only GitHub's stable numeric user ID; the supplied token is never retained. */
export async function githubSubject(token: string): Promise<{ ok: true; subject: string } | { ok: false; error: "provider_unavailable" }> {
	let response: Response;
	try {
		response = await fetch(GITHUB_USER_URL, {
			method: "GET",
			headers: {
				authorization: `Bearer ${token}`,
				"user-agent": "kanso-sync/0.1.0",
				accept: "application/vnd.github+json",
			},
			redirect: "error",
			signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
		});
	} catch {
		return unavailable();
	}
	if (!response.ok) return unavailable();
	let body: unknown;
	try {
		body = await response.json();
	} catch {
		return unavailable();
	}
	if (!isObject(body)) return unavailable();
	if (typeof body.id === "number" && Number.isSafeInteger(body.id) && body.id >= 0) {
		return { ok: true, subject: String(body.id) };
	}
	if (typeof body.id === "string" && /^[0-9]{1,255}$/.test(body.id)) return { ok: true, subject: body.id };
	return unavailable();
}

export interface GithubDeviceCode {
	device_code: string;
	user_code: string;
	verification_uri: string;
	expires_in: number;
	interval: number;
}

/**
 * Starts GitHub's device flow with `client_id` and no scope, so the token it
 * later mints reads only the public profile, of which the Worker keeps the
 * numeric ID. The device code comes back for sealing; nothing logs it.
 */
export async function githubDeviceCode(
	env: Env,
): Promise<{ ok: true; value: GithubDeviceCode } | { ok: false; error: "provider_unavailable" }> {
	if (!required(env.GITHUB_CLIENT_ID)) return unavailable();
	let response: Response;
	try {
		response = await fetch(GITHUB_DEVICE_CODE_URL, {
			method: "POST",
			headers: { accept: "application/json", "content-type": "application/x-www-form-urlencoded" },
			body: new URLSearchParams({ client_id: env.GITHUB_CLIENT_ID }).toString(),
			redirect: "error",
			signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
		});
	} catch {
		return unavailable();
	}
	if (!response.ok) return unavailable();
	let body: unknown;
	try {
		body = await response.json();
	} catch {
		return unavailable();
	}
	if (!isObject(body)) return unavailable();
	const { device_code, user_code, verification_uri, expires_in, interval } = body;
	if (
		typeof device_code !== "string" ||
		!/^[\x21-\x7e]{1,256}$/.test(device_code) ||
		typeof user_code !== "string" ||
		!/^[A-Z0-9-]{4,16}$/.test(user_code) ||
		verification_uri !== GITHUB_VERIFICATION_URI ||
		typeof expires_in !== "number" ||
		!Number.isSafeInteger(expires_in) ||
		expires_in <= 0 ||
		(interval !== undefined && (typeof interval !== "number" || !Number.isSafeInteger(interval) || interval < 0))
	) {
		return unavailable();
	}
	const seconds = typeof interval === "number" ? interval : GITHUB_MIN_INTERVAL;
	return { ok: true, value: { device_code, user_code, verification_uri, expires_in, interval: Math.max(GITHUB_MIN_INTERVAL, seconds) } };
}

/** Where GitHub's device flow sends the person; the only value the app is told to open. */
export const GITHUB_VERIFICATION_URI = "https://github.com/login/device";
/** GitHub's default device-flow interval, which `slow_down` raises by the same again. */
export const GITHUB_MIN_INTERVAL = 5;
