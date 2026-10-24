// Device-approval helpers: a second device of an account, the approval routes
// as one device calls them, commitments computed independently of the Worker
// (node:crypto), and the account event socket.
/// <reference types="node" />

import { createHash } from "node:crypto";
import { type Account, type Call, type Device, base64url, call, callJson, login, newDevice } from "./helpers";

/** One signed-in device of an account: its token and its device ID and proof. */
export interface Signed {
	token: string;
	device: Device;
}

export const first = (a: Account): Signed => ({ token: a.token, device: a.device });

/** Signs `a` in on a new device. */
export async function another(a: Account, init: Call = {}): Promise<Signed> {
	const device = newDevice(false);
	const r = await login(a.creds, device, init);
	if (r.status !== 200) throw new Error(`login: ${r.status} ${JSON.stringify(r.body)}`);
	return { token: r.body.token, device };
}

export const random32 = (): Uint8Array => crypto.getRandomValues(new Uint8Array(32));

/** `SHA-256("kanso/v1/approve-commit" || 0x01 || epk)`, base64url, by node's own hash. */
export function commitOf(epk: Uint8Array): string {
	return createHash("sha256").update(Buffer.concat([Buffer.from("kanso/v1/approve-commit"), Buffer.from([1]), Buffer.from(epk)])).digest("base64url");
}

/** An ephemeral key pair as the Worker sees it: only the public half and its commitment. */
export function ephemeral(): { epk: Uint8Array; text: string; commit: string } {
	const epk = random32();
	return { epk, text: base64url(epk), commit: commitOf(epk) };
}

/** A sealed name of `length` base64url characters. */
export const sealedName = (length = 60): string => base64url(crypto.getRandomValues(new Uint8Array(Math.ceil((length * 3) / 4)))).slice(0, length);
/** A wrapped key: 98 base64url characters (73 bytes). */
export const wrappedKey = (): string => base64url(crypto.getRandomValues(new Uint8Array(73)));

const PATH = "/v2/account/sign-in/approvals";

/** The approval routes as device `s`. */
export function approvalsAs(s: Signed, init: Call = {}) {
	const as = (path: string, more: Call = {}) => callJson(path, { ...init, ...more, token: s.token });
	return {
		create: (commit: string) => as(PATH, { json: { commit } }),
		list: () => as(PATH),
		poll: (id: string) => as(`${PATH}/${id}`),
		challenge: (id: string, epk: string) => as(`${PATH}/${id}/challenge`, { json: { epk } }),
		name: (id: string, epk: string, sealed_name: string) => as(`${PATH}/${id}/name`, { json: { epk, sealed_name } }),
		approve: (id: string, wrapped_key: string) => as(`${PATH}/${id}/approve`, { json: { wrapped_key } }),
		cancel: (id: string) => as(`${PATH}/${id}`, { method: "DELETE" }),
	};
}

// -------------------------------------------------------------------- socket

export type Frame = { text: any } | { raw: string };

export interface EventSocket {
	ws: WebSocket;
	next(ms?: number): Promise<Frame>;
	drain(ms?: number): Promise<Frame[]>;
	closed: Promise<{ code: number; reason: string }>;
}

/** Opens the account event socket as `s`, with `headers` overriding the device ones; the response when it isn't 101. */
export async function openEvents(s: Signed, headers: Record<string, string | null> = {}, init: Call = {}): Promise<EventSocket | Response> {
	const all: Record<string, string> = {};
	const merged: Record<string, string | null> = { upgrade: "websocket", "x-kanso-device": s.device.id, "x-kanso-device-proof": s.device.proof, ...headers };
	for (const [k, v] of Object.entries(merged)) if (v !== null) all[k] = v;
	const response = await call("/v2/account/events", { ...init, token: s.token, headers: all });
	const ws = response.webSocket;
	if (response.status !== 101 || !ws) return response;
	const queue: Frame[] = [];
	const waiters: ((f: Frame) => void)[] = [];
	let closedResolve: (v: { code: number; reason: string }) => void = () => {};
	const closed = new Promise<{ code: number; reason: string }>((resolve) => (closedResolve = resolve));
	ws.accept();
	ws.addEventListener("message", (event) => {
		const data = String(event.data);
		let f: Frame;
		try {
			f = { text: JSON.parse(data) };
		} catch {
			f = { raw: data };
		}
		const waiter = waiters.shift();
		if (waiter) waiter(f);
		else queue.push(f);
	});
	ws.addEventListener("close", (event) => closedResolve({ code: event.code, reason: event.reason }));
	const next = (ms = 2000): Promise<Frame> => {
		const queued = queue.shift();
		if (queued) return Promise.resolve(queued);
		return new Promise((resolve, reject) => {
			const timer = setTimeout(() => {
				const i = waiters.indexOf(done);
				if (i >= 0) waiters.splice(i, 1);
				reject(new Error("no frame"));
			}, ms);
			const done = (f: Frame) => {
				clearTimeout(timer);
				resolve(f);
			};
			waiters.push(done);
		});
	};
	const drain = async (ms = 200): Promise<Frame[]> => {
		await new Promise((r) => setTimeout(r, ms));
		return queue.splice(0, queue.length);
	};
	return { ws, next, drain, closed };
}

/** Opens the socket and reads its hello; fails the test otherwise. */
export async function listening(s: Signed): Promise<EventSocket> {
	const socket = await openEvents(s);
	if (socket instanceof Response) throw new Error(`events: ${socket.status} ${await socket.text()}`);
	const hello = await socket.next();
	if (!("text" in hello) || hello.text.type !== "hello") throw new Error(`no hello: ${JSON.stringify(hello)}`);
	return socket;
}
