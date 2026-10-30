// Kanso's sync server, protocol v2 (docs/ACCOUNTS.md): accounts with
// end-to-end encrypted libraries. The Worker stores and returns sealed values
// it can't read, keyed by account; D1 holds accounts, sessions, documents and
// the counters that enforce quotas and limits, R2 holds sealed file bytes
// under `accounts/<account_id>/<wire_id>`.
//
//   GET    /v2/health                   liveness, protocol, whether sign-ups are open
//   POST   /v2/auth/prelogin            KDF parameters, salt and account ID for a username
//   POST   /v2/auth/signup              create an account and this device's session
//   POST   /v2/auth/login               sign in a device
//   POST   /v2/auth/recover/start       prove the recovery key, get the recovery-wrapped data key
//   POST   /v2/auth/recover/finish      set a new password and recovery key
//   POST   /v2/auth/logout              sign out this device                          (session)
//   GET    /v2/account                  account, usage and quotas                     (session)
//   POST   /v2/account/password         change the password                           (session)
//   POST   /v2/account/recovery-key     replace the recovery key                      (session)
//   POST   /v2/account/delete           delete the account and its server data        (session)
//   PUT    /v2/account/device           set this device's sealed name                 (session)
//   GET    /v2/account/devices          list active devices                           (session)
//   DELETE /v2/account/devices/<id>     sign out one device                           (session)
//   PUT    /v2/account/devices/<id>/push  set this device's push token                (session)
//   DELETE /v2/account/devices/<id>/push  turn this device's pushes off                (session)
//   POST   /v2/push                     write sealed documents                        (session)
//   GET    /v2/changes?since=&limit=    the account's change feed                     (session)
//   PUT    /v2/files/<wire_id>          store sealed file bytes                       (session)
//   GET    /v2/files/<wire_id>          fetch sealed file bytes                       (session)
//   PUT    /v2/shares/<id>              publish a sealed note, or replace it          (session)
//   DELETE /v2/shares/<id>              unpublish it                                  (session)
//   GET    /v2/public/<id>              a published note's sealed bytes               (public)
//   GET    /p/<id>                      the page that opens it with the link's key    (public)
//   PUT    /v2/account/identity         set this account's identity keys, once         (session)
//   GET    /v2/account/identity         this account's identity keys                  (session)
//   GET    /v2/users/<username>         another account's public keys                 (session)
//   POST   /v2/collab/shares            share a note                                  (session)
//   GET    /v2/collab/shares            the shares you are in or invited to           (session)
//   DELETE /v2/collab/shares/<id>       stop sharing                                  (session)
//   POST   /v2/collab/shares/<id>/accept       accept an invitation                  (session)
//   DELETE /v2/collab/shares/<id>/membership   leave, or decline                     (session)
//   POST   /v2/collab/shares/<id>/epoch        a new member list and note key        (session)
//   GET    /v2/collab/shares/<id>/log          the note's snapshot and log           (session)
//   POST   /v2/collab/shares/<id>/log          append an update or comment           (session)
//   POST   /v2/collab/shares/<id>/snapshot     compact the log                       (session)
//   GET    /v2/collab/shares/<id>/live         the live WebSocket                    (session)
//   POST   /v2/collab/shares/<id>/links        make an invite link                   (session)
//   DELETE /v2/collab/shares/<id>/links/<link> cancel it                             (session)
//   DELETE /v2/collab/shares/<id>/requests/<account>  decline a request to join      (session)
//   POST   /v2/collab/join/<link_id>    ask to join through a link                    (session)
//   POST   /v2/collab/notifications     notify a member (sealed)                      (session)
//   GET    /v2/collab/notifications     your notifications                            (session)
//   DELETE /v2/collab/notifications/<id> dismiss one                                  (session)
//   GET    /j/<link_id>                 the page an invite link opens                 (public)
//   POST   /v2/auth/apple/start         Sign in with Apple: the web flow's URL
//   POST /v2/auth/apple/callback        Apple's form_post (the one Origin exception)
//   POST   /v2/auth/apple/poll          the app's completion of an Apple flow
//   POST   /v2/auth/google/start        Sign in with Google: loopback and PKCE
//   POST   /v2/auth/google/finish       the loopback's code and the PKCE verifier
//   POST   /v2/auth/github/start        Sign in with GitHub: the device flow's code
//   POST   /v2/auth/github/poll         one device-flow poll
//   POST   /v2/auth/social/signup       the account a provider flow's ticket names
//   GET    /v2/account/sign-in          the account's ways in                         (session)
//   POST   /v2/account/sign-in/password               set the first password (proof)       (session)
//   POST   /v2/account/sign-in/proof/start            a passkey proof's challenge          (session)
//   POST   /v2/account/sign-in/passkeys/start         add a passkey: its options (proof)   (session)
//   POST   /v2/account/sign-in/passkeys/finish        add a passkey: its registration      (session)
//   POST   /v2/account/sign-in/passkeys/<id>/remove   remove a passkey (proof)             (session)
//   POST   /v2/account/sign-in/identities/<provider>/start   link, or a proof flow   (session)
//   POST   /v2/account/sign-in/identities/<provider>/unlink  unlink a provider       (session)
//   POST   /v2/account/sign-in/approvals              ask another device to approve this one (session)
//   GET    /v2/account/sign-in/approvals              other devices' pending requests       (session)
//   GET    /v2/account/sign-in/approvals/<id>         the requester's poll                  (session)
//   POST   /v2/account/sign-in/approvals/<id>/challenge  the approver's ephemeral key       (session)
//   POST   /v2/account/sign-in/approvals/<id>/name    the requester's key and sealed name   (session)
//   POST   /v2/account/sign-in/approvals/<id>/approve the approver's sealed data key       (session)
//   DELETE /v2/account/sign-in/approvals/<id>         cancel or decline                     (session)
//   GET    /v2/account/events           live approval hints, a WebSocket (session + device proof)
//
//   POST   /v1/crash-reports            a crash report the person chose to send       (public, no session)
//   POST   /v1/metrics                  an anonymous usage count, passed on to PostHog (public, no session)
//
// Every other /v1 path and every other path answer 404. Any request with an Origin header gets
// 403 before anything else, except on the three public routes, which hold no
// credentials and only ever read, and Apple's callback, which takes Apple's
// origin (or none) and relies on its state; a body with the wrong Content-Type
// gets 415.
//
// Each shared note's log lives in its own Durable Object, NoteRoom (room.ts),
// and each account's live event sockets in AccountRoom (account_room.ts),
// both exported here so the runtime can find the classes.

import * as approvals from "./approvals";
import * as auth from "./auth";
import * as collab from "./collab";
import * as crashReports from "./crash_reports";
import { pepperKeys } from "./crypto";
import type { Env } from "./env";
import { error, isContentType, methodNotAllowed, notFound } from "./http";
import * as identity from "./identity";
import * as providers from "./providers";
import { bindingFilter } from "./limits";
import * as methods from "./methods";
import * as metrics from "./metrics";
import * as passkeys from "./passkeys";
import * as proofs from "./proofs";
import * as pushes from "./push";
import { runCron } from "./purge";
import * as shares from "./shares";
import * as signIn from "./sign_in";
import * as sync from "./sync";

export type { Env } from "./env";
export { runCron } from "./purge";
export { NoteRoom } from "./room";
export { AccountRoom } from "./account_room";

type Handler = (request: Request, env: Env, now: number) => Promise<Response>;
type PepperHandler = (request: Request, env: Env, now: number, keys: import("./crypto").PepperKeys) => Promise<Response>;

interface Method {
	handler: Handler | PepperHandler;
	/** The Content-Type a body must have. */
	body?: "json" | "bytes";
	/** Needs AUTH_PEPPER: 503 `misconfigured` without a valid one. */
	pepper?: boolean;
}

interface Route {
	/** For logs: the route's name, never its parameters. */
	name: string;
	methods: Record<string, Method>;
	/** Behind the AUTH_LIMITER binding (every /v2/auth/* route and /v2/health). */
	binding?: boolean;
	/** Readable by anyone, from a browser too: the Origin rule doesn't apply. */
	public?: boolean;
	/** The only Origin values this route accepts besides none (Apple's form post). */
	origins?: readonly string[];
}

const json = (handler: Handler | PepperHandler, pepper = false): Method => ({ handler, body: "json", pepper });

const aasa: Handler = async (request, env) => signIn.appSiteAssociation(request, env);

const ROUTES: Record<string, Route> = {
	"/v2/health": { name: "health", binding: true, methods: { GET: { handler: auth.health, pepper: true } } },
	"/v1/crash-reports": { name: "crash_reports", methods: { POST: json(crashReports.submit, true) } },
	"/v1/metrics": { name: "metrics", methods: { POST: json(metrics.submit, true) } },
	"/v2/auth/prelogin": { name: "prelogin", binding: true, methods: { POST: json(auth.prelogin, true) } },
	"/v2/auth/signup": { name: "signup", binding: true, methods: { POST: json(auth.signup, true) } },
	"/v2/auth/login": { name: "login", binding: true, methods: { POST: json(auth.login, true) } },
	"/v2/auth/recover/start": { name: "recover_start", binding: true, methods: { POST: json(auth.recoverStart, true) } },
	"/v2/auth/recover/finish": { name: "recover_finish", binding: true, methods: { POST: json(auth.recoverFinish, true) } },
	"/v2/auth/passkey/start": { name: "passkey_start", binding: true, methods: { POST: json(passkeys.passkeyStart, true) } },
	"/v2/auth/passkey/finish": { name: "passkey_finish", binding: true, methods: { POST: json(passkeys.passkeyFinish, true) } },
	"/v2/auth/passkey/signup/start": { name: "passkey_signup_start", binding: true, methods: { POST: json(passkeys.passkeySignupStart, true) } },
	"/v2/auth/passkey/signup/finish": { name: "passkey_signup_finish", binding: true, methods: { POST: json(passkeys.passkeySignupFinish, true) } },
	"/v2/auth/logout": { name: "logout", binding: true, methods: { POST: { handler: auth.logout } } },
	"/v2/account": { name: "account", methods: { GET: { handler: auth.account } } },
	"/v2/account/password": { name: "password", methods: { POST: json(auth.password) } },
	"/v2/account/recovery-key": { name: "recovery_key", methods: { POST: json(methods.recoveryKey) } },
	"/v2/account/delete": { name: "delete", methods: { POST: json(methods.deleteAccount) } },
	"/v2/account/device": { name: "set_device", methods: { PUT: json(auth.setDevice) } },
	"/v2/account/devices": { name: "devices", methods: { GET: { handler: auth.devices } } },
	"/v2/push": { name: "push", methods: { POST: json(sync.push) } },
	"/v2/changes": { name: "changes", methods: { GET: { handler: sync.changes } } },
	"/v2/account/identity": { name: "identity", methods: { PUT: json(identity.putIdentity), GET: { handler: identity.getIdentity } } },
	"/v2/collab/shares": { name: "collab_shares", methods: { POST: json(collab.createShare), GET: { handler: collab.listShares } } },
	"/v2/collab/notifications": { name: "collab_notifications", methods: { POST: json(collab.notify), GET: { handler: collab.listNotifications } } },
	"/v2/auth/apple/start": { name: "apple_start", binding: true, methods: { POST: json(providers.start("apple"), true) } },
	"/v2/auth/apple/poll": { name: "apple_poll", binding: true, methods: { POST: json(providers.applePoll, true) } },
	// Apple posts application/x-www-form-urlencoded from its own page (form_post, even for a
	// cancel), so there is no GET; the handler checks the type.
	"/v2/auth/apple/callback": {
		name: "apple_callback",
		binding: true,
		origins: [providers.APPLE_FORM_ORIGIN],
		methods: { POST: { handler: providers.appleCallback, pepper: true } },
	},
	"/v2/auth/google/start": { name: "google_start", binding: true, methods: { POST: json(providers.start("google"), true) } },
	"/v2/auth/google/finish": { name: "google_finish", binding: true, methods: { POST: json(providers.googleFinish, true) } },
	"/v2/auth/github/start": { name: "github_start", binding: true, methods: { POST: json(providers.start("github"), true) } },
	"/v2/auth/github/poll": { name: "github_poll", binding: true, methods: { POST: json(providers.githubPoll, true) } },
	"/v2/auth/social/signup": { name: "social_signup", binding: true, methods: { POST: json(providers.socialSignup, true) } },
	"/v2/account/sign-in": { name: "sign_in_methods", methods: { GET: { handler: providers.methods } } },
	"/v2/account/sign-in/password": { name: "set_password", methods: { POST: json(methods.setPassword) } },
	"/v2/account/sign-in/proof/start": { name: "proof_start", methods: { POST: json(proofs.proofStart, true) } },
	"/v2/account/sign-in/passkeys/start": { name: "passkey_register_start", methods: { POST: json(passkeys.registerStart, true) } },
	"/v2/account/sign-in/passkeys/finish": { name: "passkey_register_finish", methods: { POST: json(passkeys.registerFinish) } },
	"/v2/account/sign-in/approvals": { name: "approvals", methods: { POST: json(approvals.create), GET: { handler: approvals.list } } },
	"/v2/account/events": { name: "account_events", methods: { GET: { handler: approvals.events } } },
	// Read by Apple's CDN, which may send an Origin-less GET or HEAD; no secret, so public.
	[signIn.AASA_PATH]: {
		name: "aasa",
		public: true,
		methods: { GET: { handler: aasa }, HEAD: { handler: aasa } },
	},
};

/** The routes under /v2/account/sign-in/approvals/<id>, by what follows the ID. */
function approvalRoute(id: string, rest: string): Route | null {
	if (rest === "") return { name: "approval", methods: { GET: { handler: approvals.poll(id) }, DELETE: { handler: approvals.cancel(id) } } };
	if (rest === "/challenge") return { name: "approval_challenge", methods: { POST: json(approvals.challenge(id)) } };
	if (rest === "/name") return { name: "approval_name", methods: { POST: json(approvals.name(id)) } };
	if (rest === "/approve") return { name: "approval_approve", methods: { POST: json(approvals.approve(id)) } };
	return null;
}

/** The routes under /v2/collab/shares/<id>/, by what follows the ID. */
function collabShareRoute(id: string, rest: string): Route | null {
	if (rest === "") return { name: "collab_share", methods: { DELETE: { handler: collab.stopShare(id) } } };
	if (rest === "/accept") return { name: "collab_accept", methods: { POST: { handler: collab.accept(id) } } };
	if (rest === "/membership") return { name: "collab_membership", methods: { DELETE: { handler: collab.leave(id) } } };
	if (rest === "/epoch") return { name: "collab_epoch", methods: { POST: json(collab.newEpoch(id)) } };
	if (rest === "/log") {
		return { name: "collab_log", methods: { GET: { handler: collab.readLog(id) }, POST: { handler: collab.appendLog(id), body: "bytes" } } };
	}
	if (rest === "/snapshot") return { name: "collab_snapshot", methods: { POST: { handler: collab.postSnapshot(id), body: "bytes" } } };
	if (rest === "/live") return { name: "collab_live", methods: { GET: { handler: collab.live(id) } } };
	if (rest === "/links") return { name: "collab_links", methods: { POST: json(collab.createLink(id)) } };
	const link = /^\/links\/([^/]+)$/.exec(rest);
	if (link) return { name: "collab_link", methods: { DELETE: { handler: collab.deleteLink(id, segment(link[1])) } } };
	const request = /^\/requests\/([^/]+)$/.exec(rest);
	if (request) return { name: "collab_request", methods: { DELETE: { handler: collab.declineRequest(id, segment(request[1])) } } };
	return null;
}

/** A path segment, percent-decoded; "" (which no handler accepts) when it can't be. */
function segment(raw: string): string {
	try {
		return decodeURIComponent(raw);
	} catch {
		return "";
	}
}

function match(path: string): Route | null {
	const fixed = ROUTES[path];
	if (fixed) return fixed;
	const devicePush = /^\/v2\/account\/devices\/([^/]+)\/push$/.exec(path);
	if (devicePush) {
		const id = segment(devicePush[1]);
		return { name: "device_push", methods: { PUT: json(pushes.registerPush(id)), DELETE: { handler: pushes.unregisterPush(id) } } };
	}
	const device = /^\/v2\/account\/devices\/([^/]+)$/.exec(path);
	if (device) return { name: "revoke_device", methods: { DELETE: { handler: auth.revokeDevice(segment(device[1])) } } };
	const file = /^\/v2\/files\/([^/]+)$/.exec(path);
	if (file) {
		const id = segment(file[1]);
		return { name: "files", methods: { GET: { handler: sync.getFile(id) }, PUT: { handler: sync.putFile(id), body: "bytes" } } };
	}
	const share = /^\/v2\/shares\/([^/]+)$/.exec(path);
	if (share) {
		const id = segment(share[1]);
		return { name: "shares", methods: { PUT: { handler: shares.putShare(id), body: "bytes" }, DELETE: { handler: shares.deleteShare(id) } } };
	}
	const published = /^\/v2\/public\/([^/]+)$/.exec(path);
	if (published) return { name: "public", public: true, methods: { GET: { handler: shares.getPublic(segment(published[1])) } } };
	const user = /^\/v2\/users\/([^/]+)$/.exec(path);
	if (user) return { name: "user", methods: { GET: { handler: identity.getUser(segment(user[1])) } } };
	const collabShare = /^\/v2\/collab\/shares\/([^/]+)((?:\/[^/]+)*)$/.exec(path);
	if (collabShare) return collabShareRoute(segment(collabShare[1]), collabShare[2]);
	const joinLink = /^\/v2\/collab\/join\/([^/]+)$/.exec(path);
	if (joinLink) return { name: "collab_join", methods: { POST: json(collab.join(segment(joinLink[1]))) } };
	const identityRoute = /^\/v2\/account\/sign-in\/identities\/([^/]+)\/(start|unlink)$/.exec(path);
	if (identityRoute) {
		const provider = segment(identityRoute[1]);
		return identityRoute[2] === "start"
			? { name: "identity_start", methods: { POST: json(providers.identityStart(provider), true) } }
			: { name: "identity_unlink", methods: { POST: json(providers.unlink(provider), true) } };
	}
	const passkeyRemove = /^\/v2\/account\/sign-in\/passkeys\/([^/]+)\/remove$/.exec(path);
	if (passkeyRemove) return { name: "passkey_remove", methods: { POST: json(passkeys.removePasskey(segment(passkeyRemove[1]))) } };
	const notification = /^\/v2\/collab\/notifications\/([^/]+)$/.exec(path);
	if (notification) return { name: "collab_notification", methods: { DELETE: { handler: collab.deleteNotification(segment(notification[1])) } } };
	const approval = /^\/v2\/account\/sign-in\/approvals\/([^/]+)(\/[^/]+)?$/.exec(path);
	if (approval) return approvalRoute(segment(approval[1]), approval[2] ?? "");
	const invite = /^\/j\/([^/]+)$/.exec(path);
	if (invite) return { name: "invite", public: true, methods: { GET: { handler: collab.invitePage(segment(invite[1])) } } };
	const viewer = /^\/p\/([^/]+)$/.exec(path);
	if (viewer) return { name: "viewer", public: true, methods: { GET: { handler: shares.getViewer(segment(viewer[1])) } } };
	return null;
}

/** Answers one request as of `now` (Unix ms); tests pass their own clock. */
export async function handle(request: Request, env: Env, now: number): Promise<Response> {
	let name = "unknown";
	try {
		const route = match(new URL(request.url).pathname);
		// Browsers always send Origin on the requests that matter; the native client never does (M7).
		const origin = request.headers.get("origin");
		if (origin !== null && !route?.public && !route?.origins?.includes(origin)) {
			return error(403, "forbidden_origin", "Browser requests are refused.");
		}
		if (!route) return notFound();
		name = route.name;
		const method = route.methods[request.method];
		if (!method) return methodNotAllowed(Object.keys(route.methods).join(", "));
		if (route.binding) {
			const limited = await bindingFilter(env.AUTH_LIMITER, request);
			if (limited) return limited;
		}
		const type = request.headers.get("content-type");
		if (method.body === "json" && !isContentType(type, "application/json", true)) {
			return error(415, "unsupported_media_type", "Send application/json.");
		}
		if (method.body === "bytes" && !isContentType(type, "application/octet-stream", false)) {
			return error(415, "unsupported_media_type", "Send application/octet-stream.");
		}
		if (method.pepper) {
			const keys = pepperKeys(env.AUTH_PEPPER);
			if (!keys) return error(503, "misconfigured", "The server has no valid AUTH_PEPPER.");
			return await (method.handler as PepperHandler)(request, env, now, await keys);
		}
		return await (method.handler as Handler)(request, env, now);
	} catch (err) {
		// Route names and error codes only: no paths, bodies or values.
		console.error(JSON.stringify({ route: name, error: "internal", kind: (err as Error)?.name ?? typeof err }));
		return error(500, "internal", "Internal error.");
	}
}

export default {
	fetch(request, env): Promise<Response> {
		return handle(request, env, Date.now());
	},
	scheduled(_controller, env, ctx): void {
		ctx.waitUntil(
			runCron(env, Date.now()).then(
				() => undefined,
				(err) => console.error(JSON.stringify({ route: "cron", error: "internal", kind: (err as Error)?.name ?? typeof err })),
			),
		);
	},
} satisfies ExportedHandler<Env>;
