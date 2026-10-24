// Passkey registration and assertion (docs/ACCOUNTS.md › Passkeys).
//
// A software authenticator built on node:crypto produces the exact WebAuthn
// messages a platform passkey or a Linux security key would: a P-256 COSE key,
// attestation "none", and ECDSA signatures over
// `authenticatorData || SHA-256(clientDataJSON)`. No page is loaded and no
// network is reached.

/// <reference types="node" />
import { Buffer } from "node:buffer";
import { createHash, generateKeyPairSync, randomBytes, sign as nodeSign, type KeyObject } from "node:crypto";
import { beforeAll, beforeEach, describe, expect, it } from "vitest";
import { env } from "cloudflare:workers";
import { base64url } from "../src/protocol";
import { callJson, clock, newDevice, resetServer, signUp, testEnv, type Device } from "./helpers";

const ORIGIN = "https://kanso-sync.test";
const RP_ID = "kanso-sync.test";
const NOW = Date.UTC(2026, 9, 24, 12);
const encoder = new TextEncoder();
const sha256 = (data: Uint8Array | string): Uint8Array =>
	new Uint8Array(createHash("sha256").update(typeof data === "string" ? Buffer.from(data) : data).digest());

const passportEnv = () => testEnv({ PASSKEY_RP_ID: RP_ID, PASSKEY_ORIGINS: JSON.stringify([ORIGIN]) });

// --------------------------------------------------------------------- CBOR

type Cbor = number | string | Uint8Array | Cbor[] | Map<number | string, Cbor>;

function head(major: number, length: number): number[] {
	if (length < 24) return [(major << 5) | length];
	if (length <= 0xff) return [(major << 5) | 24, length];
	if (length <= 0xffff) return [(major << 5) | 25, length >> 8, length & 0xff];
	return [(major << 5) | 26, (length >>> 24) & 0xff, (length >>> 16) & 0xff, (length >>> 8) & 0xff, length & 0xff];
}

function cbor(value: Cbor): number[] {
	if (typeof value === "number") {
		return value >= 0 ? head(0, value) : head(1, -1 - value);
	}
	if (typeof value === "string") {
		const bytes = encoder.encode(value);
		return [...head(3, bytes.length), ...bytes];
	}
	if (value instanceof Uint8Array) return [...head(2, value.length), ...value];
	if (Array.isArray(value)) return [...head(4, value.length), ...value.flatMap(cbor)];
	const entries = [...value.entries()];
	return [...head(5, entries.length), ...entries.flatMap(([k, v]) => [...cbor(k), ...cbor(v)])];
}

// ---------------------------------------------------------- authenticator

interface Credential {
	privateKey: KeyObject;
	credentialId: Uint8Array;
	cose: Map<number | string, Cbor>;
	alg: -7 | -8 | -257;
}

function newCredential(): Credential {
	const { privateKey, publicKey } = generateKeyPairSync("ec", { namedCurve: "prime256v1" });
	const jwk = publicKey.export({ format: "jwk" }) as { x: string; y: string };
	const cose = new Map<number | string, Cbor>([
		[1, 2],
		[3, -7],
		[-1, 1],
		[-2, new Uint8Array(Buffer.from(jwk.x, "base64url"))],
		[-3, new Uint8Array(Buffer.from(jwk.y, "base64url"))],
	]);
	return { privateKey, credentialId: new Uint8Array(randomBytes(32)), cose, alg: -7 };
}

function newEd25519Credential(): Credential {
	const { privateKey, publicKey } = generateKeyPairSync("ed25519");
	const jwk = publicKey.export({ format: "jwk" }) as { x: string };
	const cose = new Map<number | string, Cbor>([
		[1, 1],
		[3, -8],
		[-1, 6],
		[-2, new Uint8Array(Buffer.from(jwk.x, "base64url"))],
	]);
	return { privateKey, credentialId: new Uint8Array(randomBytes(32)), cose, alg: -8 };
}

function authData(options: {
	rpId?: string;
	flags: number;
	signCount: number;
	credential?: Credential;
	aaguid?: Uint8Array;
}): Uint8Array {
	const rpIdHash = sha256(options.rpId ?? RP_ID);
	const count = new Uint8Array(4);
	new DataView(count.buffer).setUint32(0, options.signCount);
	const parts: number[] = [...rpIdHash, options.flags, ...count];
	if (options.credential) {
		const aaguid = options.aaguid ?? new Uint8Array(16).fill(7);
		parts.push(...aaguid);
		parts.push(options.credential.credentialId.length >> 8, options.credential.credentialId.length & 0xff);
		parts.push(...options.credential.credentialId);
		parts.push(...cbor(options.credential.cose));
	}
	return new Uint8Array(parts);
}

function clientData(type: string, challenge: string, origin = ORIGIN, crossOrigin: boolean | undefined = false): Uint8Array {
	return encoder.encode(JSON.stringify({ type, challenge, origin, ...(crossOrigin === undefined ? {} : { crossOrigin }) }));
}

function registration(
	credential: Credential,
	challenge: string,
	options: { origin?: string; flags?: number; signCount?: number; attStmt?: Cbor | null } = {},
) {
	const data = authData({
		flags: options.flags ?? 0x45,
		signCount: options.signCount ?? 0,
		credential,
	});
	const attStmt = options.attStmt === undefined ? new Map() : options.attStmt;
	const object = new Map<number | string, Cbor>([["fmt", "none"], ["authData", data]]);
	if (attStmt !== null) object.set("attStmt", attStmt);
	const attestationObject = new Uint8Array(cbor(object));
	return {
		credential_id: base64url(credential.credentialId),
		attestation_object: base64url(attestationObject),
		client_data_json: base64url(clientData("webauthn.create", challenge, options.origin)),
	};
}

function assertion(
	credential: Credential,
	challenge: string,
	options: { origin?: string; flags?: number; signCount?: number; challengeOverride?: string; rpId?: string; signatureBitFlip?: boolean } = {},
) {
	const data = authData({ flags: options.flags ?? 0x05, signCount: options.signCount ?? 1, rpId: options.rpId });
	const client = clientData("webauthn.get", options.challengeOverride ?? challenge, options.origin);
	const signed = new Uint8Array(data.length + 32);
	signed.set(data);
	signed.set(sha256(client), data.length);
	let signature: Uint8Array;
	if (credential.alg === -8) {
		signature = new Uint8Array(nodeSign(null, Buffer.from(signed), credential.privateKey));
	} else {
		// WebAuthn carries ECDSA signatures in ASN.1 DER, as authenticators do.
		signature = new Uint8Array(
			nodeSign("sha256", Buffer.from(signed), credential.privateKey.export({ type: "pkcs8", format: "pem" }) as string),
		);
	}
	if (options.signatureBitFlip) signature[0] ^= 1;
	return {
		credential_id: base64url(credential.credentialId),
		authenticator_data: base64url(data),
		client_data_json: base64url(client),
		signature: base64url(signature),
	};
}

// ------------------------------------------------------------------- setup

const recoveryAuthKey = () => base64url(new Uint8Array(randomBytes(32)));
const recoveryWrappedKey = () => base64url(new Uint8Array(randomBytes(73)));
const accountId = () => `a_${Buffer.from(randomBytes(16)).toString("hex")}`;
const keyId = () => Buffer.from(randomBytes(8)).toString("hex");

async function startSignup(device: Device, username = "passkey-user") {
	const account = accountId();
	const response = await callJson("/v2/auth/passkey/signup/start", {
		json: { username, account_id: account, device: { id: device.id, proof: device.proof } },
		env: passportEnv(),
		now: NOW,
	});
	response.body.account_id = account;
	return response;
}

beforeEach(async () => {
	clock.now = NOW;
	await resetServer();
});

describe("health", () => {
	it("advertises passkeys only with a usable RP ID and origin", async () => {
		await resetServer();
		const configured = await callJson("/v2/health", { env: passportEnv(), now: NOW });
		expect(configured.body.auth.passkeys).toEqual({ rp_id: RP_ID, origins: [ORIGIN], security_key: true });
		const off = await callJson("/v2/health", { now: NOW });
		expect(off.body.auth.passkeys).toBeNull();
	});
});

describe("sign-up with a passkey", () => {
	it("creates a passwordless account, stores the credential and the wrap, and issues a session", async () => {
		const device = newDevice();
		const credential = newCredential();
		const wrap = base64url(new Uint8Array(randomBytes(73)));
		const start = await startSignup(device);
		expect(start.status).toBe(200);
		expect(start.body.pub_key_cred_params).toEqual([-7, -8, -257]);
		expect(start.body.authenticator_selection).toEqual({ residentKey: "required", userVerification: "required" });
		expect(start.body.user.name).toBe("passkey-user");
		expect(Buffer.from(start.body.user.id, "base64url")).toHaveLength(16);

		const finish = await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: ["internal"],
				passkey_wrapped_key: wrap,
				username: "passkey-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(finish.status).toBe(201);
		expect(finish.body).toMatchObject({ kind: "session", username: "passkey-user", has_password: false, passkey_wrapped_key: null });
		expect(finish.body.token).toMatch(/^kanso_/);
		const row = await env.DB.prepare("SELECT * FROM account_passkeys WHERE credential_id = ?1")
			.bind(base64url(credential.credentialId))
			.first<{ wrapped_key: string; alg: number; aaguid: string; transports: string }>();
		expect(row).toMatchObject({ wrapped_key: wrap, alg: -7, transports: "internal" });
		expect(row?.aaguid).toMatch(/^[0-9a-f]{32}$/);
		const account = await env.DB.prepare("SELECT has_password, username FROM accounts WHERE id = ?1")
			.bind(start.body.account_id)
			.first<{ has_password: number; username: string }>();
		expect(account).toEqual({ has_password: 0, username: "passkey-user" });
	});

	it("registers EdDSA credentials", async () => {
		const device = newDevice();
		const credential = newEd25519Credential();
		const start = await startSignup(device, "ed-user");
		const finish = await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: [],
				username: "ed-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(finish.status).toBe(201);
	});

	it("is idempotent after a lost answer", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "retry-user");
		const body = {
			flow_id: start.body.flow_id,
			...registration(credential, start.body.challenge),
			transports: ["internal"],
			username: "retry-user",
			account_id: start.body.account_id,
			recovery_auth_key: recoveryAuthKey(),
			recovery_wrapped_key: recoveryWrappedKey(),
			key_id: keyId(),
			device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
		};
		const first = await callJson("/v2/auth/passkey/signup/finish", { json: body, env: passportEnv(), now: NOW });
		const again = await callJson("/v2/auth/passkey/signup/finish", { json: body, env: passportEnv(), now: NOW + 1000 });
		expect(first.status).toBe(201);
		expect(again.status).toBe(201);
		expect(again.body.account_id).toBe(first.body.account_id);
		expect(again.body.token).not.toBe(first.body.token);
		const count = await env.DB.prepare("SELECT count(*) AS n FROM accounts WHERE username = 'retry-user'").first<{ n: number }>();
		expect(count?.n).toBe(1);
	});

	it("refuses a wrong RP ID hash, a missing UV flag, a wrong origin and a bad attestation", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "bad-user");
		const base = {
			flow_id: start.body.flow_id,
			transports: ["internal"],
			username: "bad-user",
			account_id: start.body.account_id,
			recovery_auth_key: recoveryAuthKey(),
			recovery_wrapped_key: recoveryWrappedKey(),
			key_id: keyId(),
			device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
		};
		const cases = [
			{ ...registration(credential, start.body.challenge), attestation_object: base64url(new Uint8Array([0xa0])) },
			registration(credential, start.body.challenge, { attStmt: null }),
			registration(credential, start.body.challenge, { attStmt: new Map([["unexpected", 1]]) }),
			{ ...registration(credential, start.body.challenge), client_data_json: base64url(clientData("webauthn.get", start.body.challenge)) },
			{ ...registration(credential, start.body.challenge, { origin: "https://evil.example" }) },
			{ ...registration(credential, "another-challenge") },
		];
		for (const value of cases) {
			const finish = await callJson("/v2/auth/passkey/signup/finish", { json: { ...base, ...value }, env: passportEnv(), now: NOW });
			expect(finish.status).toBe(401);
		}
	});

	it("refuses a reserved name, a taken name and a closed server at the start", async () => {
		const device = newDevice();
		const reserved = await callJson("/v2/auth/passkey/signup/start", {
			json: { username: "admin", account_id: accountId(), device: { id: device.id, proof: device.proof } },
			env: passportEnv(),
			now: NOW,
		});
		expect(reserved.status).toBe(400);
		expect(reserved.body.error).toBe("reserved_username");
		await signUp("taken-name");
		const taken = await startSignup(device, "taken-name");
		expect(taken.status).toBe(409);
		await resetServer({ signup: "closed" });
		const closed = await startSignup(device, "closed-name");
		expect(closed.status).toBe(403);
		expect(closed.body).toMatchObject({ error: "signup_closed", reason: "closed" });
	});
});

describe("sign-in with a passkey", () => {
	it("starts username-less with no allow list and signs in with the credential's wrap", async () => {
		const device = newDevice();
		const credential = newCredential();
		const wrap = base64url(new Uint8Array(randomBytes(73)));
		const start = await startSignup(device, "signin-user");
		await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge, { signCount: 4 }),
				transports: ["usb", "nfc"],
				passkey_wrapped_key: wrap,
				username: "signin-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});

		const other = newDevice();
		const begin = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
		expect(begin.status).toBe(200);
		expect(begin.body.allow_credentials).toEqual([]);
		expect(begin.body.user_verification).toBe("required");

		const finish = await callJson("/v2/auth/passkey/finish", {
			json: {
				flow_id: begin.body.flow_id,
				...assertion(credential, begin.body.challenge, { signCount: 5 }),
				device: { id: other.id, proof: other.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(finish.status).toBe(200);
		expect(finish.body).toMatchObject({ kind: "session", username: "signin-user", passkey_wrapped_key: wrap });
		expect(finish.body.wrapped_key).toBeUndefined();
		const row = await env.DB.prepare("SELECT sign_count, last_used_at FROM account_passkeys WHERE credential_id = ?1")
			.bind(base64url(credential.credentialId))
			.first<{ sign_count: number; last_used_at: number }>();
		expect(row).toEqual({ sign_count: 5, last_used_at: NOW });
	});

	it("lists a typed username's credentials in the order added", async () => {
		const device = newDevice();
		const first = newCredential();
		const second = newCredential();
		const start = await startSignup(device, "listed-user");
		const signup = await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(first, start.body.challenge),
				transports: ["internal"],
				username: "listed-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(signup.status).toBe(201);
		await env.DB.prepare(
			`INSERT INTO account_passkeys (credential_id, account_id, public_key, alg, sign_count, transports, created_at)
  VALUES (?1, ?2, ?3, -7, 0, 'internal', ?4)`,
		)
			.bind(base64url(second.credentialId), start.body.account_id, base64url(new Uint8Array(randomBytes(32))), NOW + 1)
			.run();
		const other = newDevice();
		const signIn = await callJson("/v2/auth/passkey/start", {
			json: { username: "listed-user", device: { id: other.id, proof: other.proof } },
			env: passportEnv(),
			now: NOW,
		});
		expect(signIn.status).toBe(200);
		expect(signIn.body.allow_credentials).toEqual([
			{ id: base64url(first.credentialId), transports: ["internal"] },
			{ id: base64url(second.credentialId), transports: ["internal"] },
		]);
		// An unknown name lists nothing.
		const unknown = await callJson("/v2/auth/passkey/start", {
			json: { username: "no-such-user", device: { id: other.id, proof: other.proof } },
			env: passportEnv(),
			now: NOW,
		});
		expect(unknown.body.allow_credentials).toEqual([]);
	});

	it("recovery removes registered passkeys and prevents their sign-in", async () => {
		const device = newDevice();
		const credential = newCredential();
		const oldRecoveryKey = recoveryAuthKey();
		const start = await startSignup(device, "recovered-passkey-user");
		const signup = await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: ["internal"],
				username: "recovered-passkey-user",
				account_id: start.body.account_id,
				recovery_auth_key: oldRecoveryKey,
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(signup.status).toBe(201);

		const replacement = newDevice();
		const recovery = await callJson("/v2/auth/recover/finish", {
			json: {
				username: "recovered-passkey-user",
				recovery_auth_key: oldRecoveryKey,
				kdf: { alg: "argon2id", version: 19, m_kib: 1048576, t: 3, p: 4 },
				salt: base64url(new Uint8Array(randomBytes(16))),
				auth_key: base64url(new Uint8Array(randomBytes(32))),
				wrapped_key: recoveryWrappedKey(),
				recovery_auth_key_new: recoveryAuthKey(),
				recovery_wrapped_key_new: recoveryWrappedKey(),
				device: { id: replacement.id, sealed_name: replacement.sealedName, proof: replacement.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(recovery.status).toBe(200);
		expect(
			await env.DB.prepare("SELECT count(*) AS n FROM account_passkeys WHERE account_id = ?1")
				.bind(start.body.account_id)
				.first<number>("n"),
		).toBe(0);

		const signInDevice = newDevice();
		const signIn = await callJson("/v2/auth/passkey/start", {
			json: { username: "recovered-passkey-user", device: { id: signInDevice.id, proof: signInDevice.proof } },
			env: passportEnv(),
			now: NOW,
		});
		expect(signIn.body.allow_credentials).toEqual([]);
		const finish = await callJson("/v2/auth/passkey/finish", {
			json: {
				flow_id: signIn.body.flow_id,
				...assertion(credential, signIn.body.challenge),
				device: { id: signInDevice.id, proof: signInDevice.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(finish.status).toBe(401);
	});

	it("refuses a bad signature, wrong challenge, wrong origin, wrong RP ID, missing UV and an unknown credential", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "refuse-user");
		await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: ["internal"],
				username: "refuse-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});

		const other = newDevice();
		const begin = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
		const challenge = begin.body.challenge as string;
		const cases = [
			assertion(credential, challenge, { signatureBitFlip: true }),
			assertion(credential, challenge, { challengeOverride: "wrong" }),
			assertion(credential, challenge, { origin: "https://evil.example" }),
			assertion(credential, challenge, { rpId: "other.test" }),
			assertion(credential, challenge, { flags: 0x01 }),
			{ ...assertion(credential, challenge), credential_id: base64url(new Uint8Array(randomBytes(32))) },
		];
		for (const value of cases) {
			const finish = await callJson("/v2/auth/passkey/finish", {
				json: { flow_id: begin.body.flow_id, ...value, device: { id: other.id, proof: other.proof } },
				env: passportEnv(),
				now: NOW,
			});
			expect(finish.status).toBe(401);
			expect(finish.body.error).toBe("invalid_credentials");
			// A consumed flow answers the same way after the first attempt. Use a
			// fresh challenge for each case instead.
			const fresh = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
			begin.body.flow_id = fresh.body.flow_id;
			begin.body.challenge = fresh.body.challenge;
		}
	});

	it("refuses a non-growing sign count, but a stored count of 0 is exempt", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "counter-user");
		await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge, { signCount: 9 }),
				transports: ["internal"],
				username: "counter-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		const other = newDevice();
		const attempt = async (signCount: number) => {
			const begin = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
			return callJson("/v2/auth/passkey/finish", {
				json: {
					flow_id: begin.body.flow_id,
					...assertion(credential, begin.body.challenge, { signCount }),
					device: { id: other.id, proof: other.proof },
				},
				env: passportEnv(),
				now: NOW,
			});
		};
		expect((await attempt(9)).status).toBe(401);
		expect((await attempt(8)).status).toBe(401);
		expect((await attempt(10)).status).toBe(200);
	});

	it("accepts a stored sign count of 0 without enforcing it", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "zero-counter");
		await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge, { signCount: 0 }),
				transports: [],
				username: "zero-counter",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		const other = newDevice();
		for (const signCount of [0, 0]) {
			const begin = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
			const finish = await callJson("/v2/auth/passkey/finish", {
				json: { flow_id: begin.body.flow_id, ...assertion(credential, begin.body.challenge, { signCount }), device: { id: other.id, proof: other.proof } },
				env: passportEnv(),
				now: NOW,
			});
			expect(finish.status).toBe(200);
		}
	});

	it("refuses to replay a consumed sign-in flow", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "replay-user");
		await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: [],
				username: "replay-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		const other = newDevice();
		const begin = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
		const body = {
			flow_id: begin.body.flow_id,
			...assertion(credential, begin.body.challenge),
			device: { id: other.id, proof: other.proof },
		};
		const first = await callJson("/v2/auth/passkey/finish", { json: body, env: passportEnv(), now: NOW });
		expect(first.status).toBe(200);
		const replay = await callJson("/v2/auth/passkey/finish", { json: body, env: passportEnv(), now: NOW });
		expect(replay.status).toBe(400);
		expect(replay.body.error).toBe("flow_expired");
	});

	it("returns 403 for a disabled account after the signature", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "disabled-user");
		const finish = await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: ["internal"],
				username: "disabled-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(finish.status).toBe(201);
		await env.DB.prepare("UPDATE accounts SET disabled_at = ?2 WHERE id = ?1").bind(start.body.account_id, NOW).run();
		const other = newDevice();
		const begin = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
		const denied = await callJson("/v2/auth/passkey/finish", {
			json: { flow_id: begin.body.flow_id, ...assertion(credential, begin.body.challenge), device: { id: other.id, proof: other.proof } },
			env: passportEnv(),
			now: NOW,
		});
		expect(denied.status).toBe(403);
		expect(denied.body.error).toBe("account_disabled");
	});

	it("refuses a flow made for another device or another kind", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "flow-user");
		await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: [],
				username: "flow-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		const other = newDevice();
		const begin = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
		const stolen = await callJson("/v2/auth/passkey/finish", {
			json: { flow_id: begin.body.flow_id, ...assertion(credential, begin.body.challenge), device: { id: device.id, proof: device.proof } },
			env: passportEnv(),
			now: NOW,
		});
		expect(stolen.status).toBe(400);
		const wrongKind = await callJson("/v2/auth/passkey/finish", {
			json: { flow_id: begin.body.flow_id, ...assertion(credential, begin.body.challenge), device: { id: other.id, proof: other.proof } },
			env: { ...passportEnv(), PASSKEY_RP_ID: undefined },
			now: NOW,
		});
		expect(wrongKind.status).toBe(400);
		expect(wrongKind.body.error).toBe("unsupported_method");
	});

	it("refuses an expired challenge and is single use", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "expiry-user");
		await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: [],
				username: "expiry-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		const other = newDevice();
		const begin = await callJson("/v2/auth/passkey/start", { json: { device: { id: other.id, proof: other.proof } }, env: passportEnv(), now: NOW });
		const expired = await callJson("/v2/auth/passkey/finish", {
			json: { flow_id: begin.body.flow_id, ...assertion(credential, begin.body.challenge), device: { id: other.id, proof: other.proof } },
			env: passportEnv(),
			now: NOW + 5 * 60_000 + 1,
		});
		expect(expired.status).toBe(400);
		expect(expired.body.error).toBe("flow_expired");
	});
});

describe("no Worker page and no password wrap", () => {
	it("never answers a passkey sign-in with the password wrap", async () => {
		const device = newDevice();
		const credential = newCredential();
		const start = await startSignup(device, "no-wrap-user");
		const finish = await callJson("/v2/auth/passkey/signup/finish", {
			json: {
				flow_id: start.body.flow_id,
				...registration(credential, start.body.challenge),
				transports: [],
				username: "no-wrap-user",
				account_id: start.body.account_id,
				recovery_auth_key: recoveryAuthKey(),
				recovery_wrapped_key: recoveryWrappedKey(),
				key_id: keyId(),
				device: { id: device.id, sealed_name: device.sealedName, proof: device.proof },
			},
			env: passportEnv(),
			now: NOW,
		});
		expect(finish.body).not.toHaveProperty("wrapped_key");
		const page = await callJson("/p/anything", { now: NOW });
		expect(page.status).toBe(404);
	});
});

beforeAll(() => {
	// The test's ORIGIN must be under the RP ID for the server's own origin list.
	expect(new URL(ORIGIN).hostname.endsWith(RP_ID)).toBe(true);
});
