// Test helpers: requests through handle(request, env, now) with a test clock,
// a typed v2 client, fresh credentials, a server reset, and the admin
// commands run against the test database.

import { env } from "cloudflare:workers";
import { type AdminIO, type Report, type RestoreTarget, type StatementResult, admin } from "../src/accounts";
import { hashToken, sha256Hex } from "../src/crypto";
import { type Env, handle } from "../src/index";
import { base64url, fromBase64url } from "../src/protocol";

/** The fixtures' start: a UTC midnight. */
export const START = 1_800_057_600_000;
export const ORIGIN = "https://kanso-sync.test";

/** The settings 0002_accounts.sql seeds. */
export const SEEDED_SETTINGS: Record<string, string> = {
	signup: "closed",
	max_accounts: "50",
	signups_per_day: "5",
	quota_docs: "50000",
	quota_doc_bytes: "104857600",
	quota_files: "10000",
	quota_file_bytes: "524288000",
	writes_per_day: "20000",
	upload_bytes_per_day: "1073741824",
	global_doc_bytes: "6000000000",
	global_file_bytes: "50000000000",
	global_writes_per_day: "300000",
	global_upload_bytes_per_day: "5000000000",
};

/** The test clock every helper uses unless told otherwise. */
export const clock = { now: START };

/** The Worker's bindings without AUTH_LIMITER and SESSION_LIMITER, as the fixtures run. */
export function testEnv(overrides: Partial<Env> = {}): Env {
	return { DB: env.DB, FILES: env.FILES, NOTE_ROOM: env.NOTE_ROOM, ACCOUNT_ROOM: env.ACCOUNT_ROOM, AUTH_PEPPER: env.AUTH_PEPPER, ...overrides };
}

export interface Call {
	method?: string;
	body?: BodyInit | null;
	/** JSON-encoded with Content-Type application/json. */
	json?: unknown;
	headers?: Record<string, string>;
	/** Sent as `Authorization: Bearer <token>`. */
	token?: string;
	now?: number;
	env?: Env;
	/** CF-Connecting-IP. */
	ip?: string;
}

export function request(path: string, init: Call = {}): Request {
	const headers = new Headers(init.headers);
	if (init.token !== undefined) headers.set("authorization", `Bearer ${init.token}`);
	if (init.ip !== undefined) headers.set("cf-connecting-ip", init.ip);
	let body = init.body ?? null;
	if (init.json !== undefined) {
		body = JSON.stringify(init.json);
		if (!headers.has("content-type")) headers.set("content-type", "application/json");
	}
	return new Request(`${ORIGIN}${path}`, { method: init.method ?? (body === null ? "GET" : "POST"), body, headers });
}

export function call(path: string, init: Call = {}): Promise<Response> {
	return handle(request(path, init), init.env ?? testEnv(), init.now ?? clock.now);
}

export async function callJson(path: string, init: Call = {}): Promise<{ status: number; body: any; headers: Headers }> {
	const response = await call(path, init);
	const text = await response.text();
	return { status: response.status, body: text === "" ? null : JSON.parse(text), headers: response.headers };
}

// --------------------------------------------------------------- credentials

const random = (n: number): Uint8Array => crypto.getRandomValues(new Uint8Array(n));
const randomHex = (n: number): string => [...random(n)].map((b) => b.toString(16).padStart(2, "0")).join("");

export const KDF = { alg: "argon2id", version: 19, m_kib: 1_048_576, t: 3, p: 4 };

/** Everything a Mac would derive for one account, at random. */
export interface Creds {
	username: string;
	accountId: string;
	salt: string;
	authKey: string;
	wrappedKey: string;
	recoveryAuthKey: string;
	recoveryWrappedKey: string;
	keyId: string;
}

export interface Device {
	id: string;
	proof: string;
	sealedName: string | null;
}

export function newCreds(username: string): Creds {
	return {
		username,
		accountId: `a_${randomHex(16)}`,
		salt: base64url(random(16)),
		authKey: base64url(random(32)),
		wrappedKey: base64url(random(73)),
		recoveryAuthKey: base64url(random(32)),
		recoveryWrappedKey: base64url(random(73)),
		keyId: randomHex(8),
	};
}

export function newDevice(named = true): Device {
	return { id: `d_${randomHex(16)}`, proof: base64url(random(32)), sealedName: named ? base64url(random(60)) : null };
}

export function signupBody(c: Creds, d: Device) {
	return {
		username: c.username,
		account_id: c.accountId,
		kdf: KDF,
		salt: c.salt,
		auth_key: c.authKey,
		wrapped_key: c.wrappedKey,
		recovery_auth_key: c.recoveryAuthKey,
		recovery_wrapped_key: c.recoveryWrappedKey,
		key_id: c.keyId,
		device: { id: d.id, sealed_name: d.sealedName, proof: d.proof },
	};
}

/**
 * A password change from `currentAuthKey` to `next`'s salt, auth key and
 * wrap, with `next`'s recovery key as the new one (a password change always
 * replaces the recovery key).
 */
export function passwordBody(currentAuthKey: string, next: Creds) {
	return {
		current_auth_key: currentAuthKey,
		kdf: KDF,
		salt: next.salt,
		auth_key: next.authKey,
		wrapped_key: next.wrappedKey,
		recovery_auth_key: next.recoveryAuthKey,
		recovery_wrapped_key: next.recoveryWrappedKey,
	};
}

export interface Account {
	creds: Creds;
	device: Device;
	token: string;
	epoch: string;
}

/** Signs up `username` with fresh credentials; fails the test unless it gets 201. */
export async function signUp(username: string, init: Call = {}): Promise<Account> {
	const creds = newCreds(username);
	const device = newDevice();
	const r = await callJson("/v2/auth/signup", { ...init, json: signupBody(creds, device) });
	if (r.status !== 201) throw new Error(`sign-up of ${username}: ${r.status} ${JSON.stringify(r.body)}`);
	return { creds, device, token: r.body.token, epoch: r.body.epoch };
}

export function login(creds: Creds, device: Device, init: Call = {}) {
	return callJson("/v2/auth/login", {
		...init,
		json: { username: creds.username, auth_key: creds.authKey, device: { id: device.id, proof: device.proof } },
	});
}

// -------------------------------------------------------------------- client

/** Random base64url text of length n; past 64 KiB of randomness, a random block repeated. */
const sealedOf = (n: number): string => {
	const block = base64url(random(Math.min(Math.ceil((n * 3) / 4), 48 * 1024)));
	return block.repeat(Math.ceil(n / block.length)).slice(0, n);
};

/** A random sealed string of `length` base64url characters. */
export const sealed = (length = 64): string => sealedOf(length);
export const wireId = (kind = "note"): string => `${kind}:${base64url(random(16))}`;
export const wireRev = (): string => randomHex(16);

/** The sync protocol as the holder of `token`. */
export function client(token: string, init: Call = {}) {
	const as = (path: string, more: Call = {}) => callJson(path, { ...init, ...more, token });
	return {
		token,
		push: (docs: unknown) => as("/v2/push", { json: { docs } }),
		changes: (since?: number, limit?: number) => {
			const query = new URLSearchParams();
			if (since !== undefined) query.set("since", String(since));
			if (limit !== undefined) query.set("limit", String(limit));
			return as(`/v2/changes?${query}`);
		},
		async putFile(id: string, bytes: Uint8Array, sha?: string) {
			const response = await call(`/v2/files/${id}`, {
				...init,
				token,
				method: "PUT",
				body: bytes,
				// As the native client sends it; the Worker counts upload bytes from it before reading the body.
				headers: { "content-type": "application/octet-stream", "content-length": String(bytes.length), "x-kanso-sha256": sha ?? (await sha256Hex(bytes)) },
			});
			return { status: response.status, body: await response.json<any>() };
		},
		async getFile(id: string) {
			const response = await call(`/v2/files/${id}`, { ...init, token });
			return { status: response.status, bytes: new Uint8Array(await response.arrayBuffer()) };
		},
		async putShare(id: string, bytes: Uint8Array, options: { sha?: string; expiresAt?: number } = {}) {
			const headers: Record<string, string> = {
				"content-type": "application/octet-stream",
				"content-length": String(bytes.length),
				"x-kanso-sha256": options.sha ?? (await sha256Hex(bytes)),
			};
			if (options.expiresAt !== undefined) headers["x-kanso-expires-at"] = String(options.expiresAt);
			const response = await call(`/v2/shares/${id}`, { ...init, token, method: "PUT", body: bytes, headers });
			return { status: response.status, body: await response.json<any>() };
		},
		async deleteShare(id: string) {
			const response = await call(`/v2/shares/${id}`, { ...init, token, method: "DELETE" });
			return { status: response.status, body: response.status === 204 ? null : await response.json<any>() };
		},
		account: () => as("/v2/account"),
		devices: () => as("/v2/account/devices"),
		logout: () => as("/v2/auth/logout", { method: "POST" }),
		revoke: (deviceId: string) => as(`/v2/account/devices/${deviceId}`, { method: "DELETE" }),
	};
}

export type Client = ReturnType<typeof client>;

// --------------------------------------------------------------------- state

/** Empties every account table and the bucket, and puts the settings back to the seed (sign-ups open unless told). */
export async function resetServer(settings: Record<string, string> = { signup: "open" }): Promise<void> {
	const values = { ...SEEDED_SETTINGS, ...settings };
	await env.DB.batch([
		// Shared notes first: their rows point at accounts. Rooms keyed by old share IDs are left
		// alone; tests use fresh IDs.
		env.DB.prepare("DELETE FROM collab_notifications"),
		env.DB.prepare("DELETE FROM collab_wraps"),
		env.DB.prepare("DELETE FROM collab_lists"),
		env.DB.prepare("DELETE FROM collab_requests"),
		env.DB.prepare("DELETE FROM collab_links"),
		env.DB.prepare("DELETE FROM collab_members"),
		env.DB.prepare("DELETE FROM collab_shares"),
		env.DB.prepare("DELETE FROM collab_retired"),
		env.DB.prepare("DELETE FROM collab_purges"),
		env.DB.prepare("DELETE FROM device_approvals"),
		env.DB.prepare("DELETE FROM account_passkeys"),
		env.DB.prepare("DELETE FROM account_identities"),
		env.DB.prepare("DELETE FROM auth_flows"),
		env.DB.prepare("DELETE FROM identities"),
		env.DB.prepare("DELETE FROM account_docs"),
		env.DB.prepare("DELETE FROM account_files"),
		env.DB.prepare("DELETE FROM shares"),
		env.DB.prepare("DELETE FROM share_purges"),
		env.DB.prepare("DELETE FROM retired_shares"),
		env.DB.prepare("DELETE FROM short_link_purges"),
		env.DB.prepare("DELETE FROM sessions"),
		env.DB.prepare("DELETE FROM accounts"),
		env.DB.prepare("DELETE FROM limits"),
		env.DB.prepare("DELETE FROM purges"),
		env.DB.prepare("UPDATE daily SET day = 0, count = 0"),
		...Object.entries(values).map(([name, value]) => env.DB.prepare("UPDATE settings SET value = ?2 WHERE name = ?1").bind(name, value)),
	]);
	for (;;) {
		const listed = await env.FILES.list({ limit: 1000 });
		if (listed.objects.length === 0) break;
		await env.FILES.delete(listed.objects.map((o) => o.key));
	}
	clock.now = START;
}

export async function setSetting(name: string, value: string): Promise<void> {
	await env.DB.prepare("UPDATE settings SET value = ?2 WHERE name = ?1").bind(name, value).run();
}

/** Every R2 key, sorted. */
export async function r2Keys(): Promise<string[]> {
	const keys: string[] = [];
	let cursor: string | undefined;
	do {
		const listed = await env.FILES.list({ limit: 1000, cursor });
		keys.push(...listed.objects.map((o) => o.key));
		cursor = listed.truncated ? listed.cursor : undefined;
	} while (cursor);
	return keys.sort();
}

export async function accountRow(accountId: string): Promise<Record<string, any> | null> {
	return env.DB.prepare("SELECT * FROM accounts WHERE id = ?1").bind(accountId).first();
}

/** The account's counters recomputed from its rows, as `recount` would set them. */
export async function recomputed(accountId: string): Promise<{ docs: number; doc_bytes: number; files: number; file_bytes: number }> {
	const row = await env.DB.prepare(
		`SELECT (SELECT count(*) FROM account_docs WHERE account_id = ?1 AND deleted = 0) AS docs,
  (SELECT coalesce(sum(length(sealed) + 512), 0) FROM account_docs WHERE account_id = ?1) AS doc_bytes,
  (SELECT count(*) FROM account_files WHERE account_id = ?1) + (SELECT count(*) FROM shares WHERE account_id = ?1) AS files,
  (SELECT coalesce(sum(size + 1024), 0) FROM account_files WHERE account_id = ?1)
    + (SELECT coalesce(sum(size + 1024), 0) FROM shares WHERE account_id = ?1) AS file_bytes`,
	)
		.bind(accountId)
		.first<{ docs: number; doc_bytes: number; files: number; file_bytes: number }>();
	return row!;
}

export { hashToken, base64url, fromBase64url };

// --------------------------------------------------------------------- admin

/**
 * An AdminIO over a test database, with files and deleted secrets kept in
 * memory. A Time Travel restore is only recorded, never run: it answers
 * `restoreAnswer`, or throws it if it is an Error.
 */
export function testIO(options: { db?: D1Database; local?: boolean; restoreAnswer?: { bookmark: string; previousBookmark: string } | Error } = {}) {
	const db = options.db ?? env.DB;
	const files = new Map<string, string | Uint8Array>();
	const dirs: string[] = [];
	const secretsDeleted: string[] = [];
	const exports: { tables: string[]; file: string }[] = [];
	const restores: RestoreTarget[] = [];
	/** Every call that reaches the outside world, in order. */
	const calls: string[] = [];
	const io: AdminIO = {
		local: options.local ?? true,
		async sql(statements: string[]): Promise<StatementResult[]> {
			calls.push(`sql ${statements.join("; ")}`);
			const results = await db.batch(statements.map((sql) => db.prepare(sql)));
			return results.map((r) => ({ results: r.results as Record<string, unknown>[] }));
		},
		async exportTables(tables, file) {
			exports.push({ tables, file });
			files.set(file, `-- export of ${tables.join(", ")}\n`);
		},
		async r2Get(key, file) {
			const object = await env.FILES.get(key);
			if (!object) return false;
			files.set(file, new Uint8Array(await object.arrayBuffer()));
			return true;
		},
		async makeDir(dir) {
			dirs.push(dir);
		},
		async writeFile(file, content) {
			if (files.has(file)) throw new Error(`EEXIST: ${file}`);
			files.set(file, content);
		},
		async readFile(file) {
			const content = files.get(file);
			if (typeof content !== "string") throw new Error(`ENOENT: ${file}`);
			return content;
		},
		async deleteSecret(name) {
			secretsDeleted.push(name);
		},
		async timeTravelRestore(target) {
			calls.push(`restore ${target.kind}=${target.value}`);
			restores.push(target);
			const answer = options.restoreAnswer ?? { bookmark: "00000001-00000002-00000003-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", previousBookmark: "00000001-00000002-00000003-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" };
			if (answer instanceof Error) throw answer;
			return answer;
		},
	};
	return { io, files, dirs, secretsDeleted, exports, restores, calls };
}

/** An admin command against the test database. */
export async function runAdmin(args: string[], now = clock.now, db: D1Database = env.DB): Promise<Report> {
	return admin(args, testIO({ db }).io, now);
}
