// The sync routes (docs/ACCOUNTS.md › POST /v2/push, GET /v2/changes, PUT and
// GET /v2/files): v1's semantics, scoped to the caller's account, over sealed
// values the Worker never decodes.
//
// Quotas, caps and the server-wide daily write and upload caps live in the
// migration's triggers, so every growth is checked inside the batch that
// makes it, deltas computed in SQL. A refusal rolls the batch back; the
// Worker reads the account's usage afterwards to say what was over.

import { type Session, withSession } from "./auth";
import type { Env } from "./env";
import {
	JSON_TYPE,
	badRequest,
	constraint,
	error,
	json,
	notFound,
	rateLimited,
	readBody,
	readJson,
	refusal,
	tooLarge,
	unauthorized,
	untilMidnight,
} from "./http";
import { accountKey, count } from "./limits";
import { wakeAccount } from "./push";
import {
	DEFAULT_CHANGES_LIMIT,
	DOC_ROW_COST,
	FILE_ROW_COST,
	MAX_CHANGES_LIMIT,
	MAX_PUSH_BODY,
	MAX_PUSH_DOCS,
	MAX_SEALED_FILE,
	MIN_SEALED_FILE,
	PAGE_SEALED_BUDGET,
	checkPush,
	isSha256Hex,
	isWireFileId,
	natParam,
} from "./protocol";
import { fileKey } from "./purge";

// ------------------------------------------------------------------- quotas

type Family = "doc" | "file";

/** 413 `quota_exceeded` for a trigger's refusal: `used` is the usage now (the batch rolled back), `limit` the quota or cap. */
async function quotaExceeded(env: Env, accountId: string, quota: string, family: Family): Promise<Response> {
	const row = await env.DB.prepare(
		`SELECT a.docs, a.doc_bytes, a.files, a.file_bytes,
  coalesce(a.quota_docs, d.docs, 0) AS quota_docs, coalesce(a.quota_doc_bytes, d.doc_bytes, 0) AS quota_doc_bytes,
  coalesce(a.quota_files, d.files, 0) AS quota_files, coalesce(a.quota_file_bytes, d.file_bytes, 0) AS quota_file_bytes,
  coalesce(d.global_doc_bytes, 0) AS global_doc_bytes, coalesce(d.global_file_bytes, 0) AS global_file_bytes,
  (SELECT coalesce(sum(doc_bytes), 0) FROM accounts) AS all_doc_bytes, (SELECT coalesce(sum(file_bytes), 0) FROM accounts) AS all_file_bytes
  FROM accounts a, defaults d WHERE a.id = ?1`,
	)
		.bind(accountId)
		.first<Record<string, number>>();
	if (!row) return unauthorized("unknown");
	const figures: Record<string, [number, number]> = {
		docs: [row.docs, row.quota_docs],
		doc_bytes: [row.doc_bytes, row.quota_doc_bytes],
		files: [row.files, row.quota_files],
		file_bytes: [row.file_bytes, row.quota_file_bytes],
		server: family === "doc" ? [row.all_doc_bytes, row.global_doc_bytes] : [row.all_file_bytes, row.global_file_bytes],
	};
	const [used, limit] = figures[quota] ?? figures.server;
	return error(413, "quota_exceeded", "Over quota.", { quota: quota in figures ? quota : "server", used, limit });
}

/** The response for a batch the triggers or a CHECK refused, or null when `err` is something else. */
async function refused(env: Env, accountId: string, err: unknown, family: Family, now: number): Promise<Response | null> {
	const r = refusal(err);
	if (r?.kind === "quota_exceeded") return quotaExceeded(env, accountId, r.detail, family);
	if (r?.kind === "rate_limited") return rateLimited(r.detail, untilMidnight(now));
	if (constraint(err, "CHECK")) return badRequest("A value isn't storable (sealed text must be base64url).");
	return null;
}

// --------------------------------------------------------------------- push

// ?1 account, ?2 wire id, ?3 base_rev, ?4 rev, ?5 deleted (0/1), ?6 sealed, ?7 length(sealed), ?8 now
//
// Stored lengths are read with octet_length(), which SQLite answers from the record header:
// length() on TEXT reads the whole value, overflow pages included, so a feed or a conflict
// over large documents would read every byte of them. `sealed` is base64url (a CHECK
// enforces it), so its bytes are its characters.
const ACCEPT = `((?3 IS NULL AND NOT EXISTS (SELECT 1 FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2))
  OR EXISTS (SELECT 1 FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2 AND d.rev = ?3))`;
/** Takes the account's next seq and charges the document, only when it is accepted. */
const BUMP = `UPDATE accounts SET
  last_seq = last_seq + 1,
  docs = docs + (1 - ?5)
       - coalesce((SELECT 1 - d.deleted FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2), 0),
  doc_bytes = doc_bytes + ?7 + ${DOC_ROW_COST}
       - coalesce((SELECT octet_length(d.sealed) + ${DOC_ROW_COST} FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2), 0)
  WHERE accounts.id = ?1 AND ${ACCEPT} RETURNING last_seq AS value`;
/**
 * On a conflict, the server's version at that point of the batch. Sealed
 * text over INLINE_SEALED characters is fetched after the batch, within the
 * response's budget, so 200 conflicts can't make one huge D1 result.
 */
const INLINE_SEALED = Math.floor(PAGE_SEALED_BUDGET / MAX_PUSH_DOCS / 1.05);
const CURRENT = `SELECT id, rev, deleted, octet_length(sealed) AS len,
  CASE WHEN octet_length(sealed) <= ${INLINE_SEALED} THEN sealed END AS sealed
  FROM account_docs WHERE account_id = ?1 AND id = ?2 AND NOT ${ACCEPT}`;
const WRITE = `INSERT INTO account_docs (account_id, id, rev, seq, deleted, sealed, updated_at)
  SELECT ?1, ?2, ?4, last_seq, ?5, ?6, ?8 FROM accounts WHERE accounts.id = ?1 AND ${ACCEPT}
  ON CONFLICT (account_id, id) DO UPDATE SET rev = excluded.rev, seq = excluded.seq,
    deleted = excluded.deleted, sealed = excluded.sealed, updated_at = excluded.updated_at`;

interface CurrentRow {
	id: string;
	rev: string;
	deleted: number;
	len: number;
	sealed: string | null;
}

type WireDoc = { id: string; rev: string; deleted: boolean; sealed: string };
type PushResult =
	| { id: string; status: "ok"; seq: number }
	| { id: string; status: "conflict"; current: WireDoc | null }
	| { id: string; status: "behind" };

export const push = withSession(async (request, env, now, session) => {
	const writes = accountKey("writes-account", session.accountId);
	// A refused body still cost a read of up to 4 MiB and a parse: it counts one write, so
	// failed pushes aren't free (security review 2, finding 2). Over the budget, the 429 wins.
	const refuse = async (response: Response): Promise<Response> => (await count(env.DB, "writes-account", writes, now, 1)) ?? response;
	const body = await readJson(request, MAX_PUSH_BODY);
	if (!body.ok) return refuse(body.response);
	const checked = checkPush(body.value);
	if (!checked.ok) return refuse(error(checked.status, checked.code, checked.message));
	const docs = checked.value;
	if (docs.length === 0) return json({ results: [] });
	const limited = await count(env.DB, "writes-account", writes, now, docs.length);
	if (limited) return limited;

	// One batch is one transaction; its statements run in order, so each
	// document sees the ones before it.
	const bump = env.DB.prepare(BUMP);
	const current = env.DB.prepare(CURRENT);
	const write = env.DB.prepare(WRITE);
	const statements: D1PreparedStatement[] = [];
	for (const doc of docs) {
		const deleted = doc.deleted ? 1 : 0;
		statements.push(
			bump.bind(session.accountId, doc.id, doc.baseRev, doc.rev, deleted, doc.sealed, doc.sealed.length),
			current.bind(session.accountId, doc.id, doc.baseRev),
			write.bind(session.accountId, doc.id, doc.baseRev, doc.rev, deleted, doc.sealed, doc.sealed.length, now),
		);
	}
	let outcomes: D1Result[];
	try {
		outcomes = await env.DB.batch(statements);
	} catch (err) {
		const response = await refused(env, session.accountId, err, "doc", now);
		if (response) return response;
		throw err;
	}

	const results: PushResult[] = [];
	const large: { index: number; row: CurrentRow }[] = [];
	let budget = PAGE_SEALED_BUDGET;
	for (const [i, doc] of docs.entries()) {
		const seq = (outcomes[3 * i].results as { value: number }[])[0]?.value;
		if (seq !== undefined) {
			results.push({ id: doc.id, status: "ok", seq });
			continue;
		}
		const row = (outcomes[3 * i + 1].results as CurrentRow[])[0];
		if (!row) {
			results.push({ id: doc.id, status: "conflict", current: null });
		} else if (row.len > budget) {
			results.push({ id: doc.id, status: "behind" });
		} else {
			budget -= row.len;
			if (row.sealed === null) large.push({ index: i, row });
			results.push({
				id: doc.id,
				status: "conflict",
				current: { id: row.id, rev: row.rev, deleted: row.deleted !== 0, sealed: row.sealed ?? "" },
			});
		}
	}
	if (large.length > 0) {
		const { results: rows } = await env.DB.prepare(
			"SELECT id, rev, deleted, sealed FROM account_docs WHERE account_id = ?1 AND id IN (SELECT value FROM json_each(?2))",
		)
			.bind(session.accountId, JSON.stringify(large.map((l) => l.row.id)))
			.all<{ id: string; rev: string; deleted: number; sealed: string }>();
		const byId = new Map(rows.map((r) => [r.id, r]));
		for (const { index, row } of large) {
			const fresh = byId.get(row.id);
			// Changed since the batch (by a later document or another push): the client pulls instead.
			results[index] =
				fresh && fresh.rev === row.rev && fresh.deleted === row.deleted
					? { id: row.id, status: "conflict", current: { id: row.id, rev: row.rev, deleted: row.deleted !== 0, sealed: fresh.sealed } }
					: { id: row.id, status: "behind" };
		}
	}

	await dropDeletedFiles(env, session, docs, results);
	if (results.some((r) => r.status === "ok")) await wakeAccount(env, session.accountId, session.deviceId, now);
	return json({ results });
});

/**
 * Accepted file tombstones: their bytes go from R2 (one call), then their
 * `account_files` rows and charges from D1 (one batch). A file whose last
 * accepted version in this push is live keeps its bytes.
 */
async function dropDeletedFiles(env: Env, session: Session, docs: { id: string; deleted: boolean }[], results: PushResult[]): Promise<void> {
	const last = new Map<string, boolean>();
	docs.forEach((doc, i) => {
		if (results[i].status === "ok" && doc.id.startsWith("file:")) last.set(doc.id, doc.deleted);
	});
	const gone = [...last].filter(([, deleted]) => deleted).map(([id]) => id);
	if (gone.length === 0) return;
	try {
		await env.FILES.delete(gone.map((id) => fileKey(session.accountId, id)));
	} catch {
		// The rows and charges still go: objects left without a row are orphans, which
		// reconciliation deletes. Keeping the rows would charge the account for them forever.
	}
	const ids = JSON.stringify(gone);
	const rows = `FROM account_files WHERE account_id = ?1 AND id IN (SELECT value FROM json_each(?2))`;
	await env.DB.batch([
		env.DB.prepare(
			`UPDATE accounts SET files = files - (SELECT count(*) ${rows}),
  file_bytes = file_bytes - (SELECT coalesce(sum(size + ${FILE_ROW_COST}), 0) ${rows}) WHERE id = ?1`,
		).bind(session.accountId, ids),
		env.DB.prepare(`DELETE ${rows}`).bind(session.accountId, ids),
	]);
}

// ------------------------------------------------------------------ changes

export const changes = withSession(async (request, env, _now, session) => {
	const params = new URL(request.url).searchParams;
	const since = natParam(params.get("since"), 0);
	if (since === null) return badRequest("since must be a non-negative integer.");
	const limit = natParam(params.get("limit"), DEFAULT_CHANGES_LIMIT);
	if (limit === null || limit < 1 || limit > MAX_CHANGES_LIMIT) return badRequest(`limit must be between 1 and ${MAX_CHANGES_LIMIT}.`);

	// First the sequence numbers and lengths, then the rows that fit the
	// budget of sealed text, always at least one.
	const { results: lengths } = await env.DB.prepare(
		"SELECT seq, octet_length(sealed) AS len FROM account_docs WHERE account_id = ?1 AND seq > ?2 ORDER BY seq LIMIT ?3",
	)
		.bind(session.accountId, since, limit)
		.all<{ seq: number; len: number }>();
	let cut = 0;
	let total = 0;
	while (cut < lengths.length && (cut === 0 || total + lengths[cut].len <= PAGE_SEALED_BUDGET)) {
		total += lengths[cut].len;
		cut += 1;
	}
	const more = cut < lengths.length || lengths.length === limit;
	const head = env.DB.prepare("SELECT last_seq, epoch, key_id FROM accounts WHERE id = ?1").bind(session.accountId);
	let rows: { id: string; rev: string; deleted: number; sealed: string; seq: number }[] = [];
	let account: { last_seq: number; epoch: string; key_id: string } | undefined;
	if (cut > 0) {
		const [page, heads] = await env.DB.batch<Record<string, unknown>>([
			env.DB.prepare(
				"SELECT id, rev, deleted, sealed, seq FROM account_docs WHERE account_id = ?1 AND seq > ?2 AND seq <= ?3 ORDER BY seq",
			).bind(session.accountId, since, lengths[cut - 1].seq),
			head,
		]);
		rows = page.results as typeof rows;
		account = heads.results[0] as typeof account;
	} else {
		account = (await head.first<{ last_seq: number; epoch: string; key_id: string }>()) ?? undefined;
	}
	if (!account) return unauthorized("unknown");
	// Each string goes in through JSON.stringify; the sealed text is spliced, not re-parsed.
	const docs = rows.map(
		(r) =>
			`{"id":${JSON.stringify(r.id)},"rev":${JSON.stringify(r.rev)},"deleted":${r.deleted !== 0},"sealed":${JSON.stringify(r.sealed)},"seq":${r.seq}}`,
	);
	const lastSeq = rows.length > 0 ? rows[rows.length - 1].seq : since;
	return new Response(
		`{"docs":[${docs.join(",")}],"last_seq":${lastSeq},"more":${more},"head_seq":${account.last_seq},"epoch":${JSON.stringify(account.epoch)},"key_id":${JSON.stringify(account.key_id)}}`,
		{ headers: { "content-type": JSON_TYPE } },
	);
});

// -------------------------------------------------------------------- files

const PUT_FILE = [
	"SELECT size FROM account_files WHERE account_id = ?1 AND id = ?2",
	`UPDATE accounts SET
  files = files + (NOT EXISTS (SELECT 1 FROM account_files WHERE account_id = ?1 AND id = ?2)),
  file_bytes = file_bytes + ?3 + ${FILE_ROW_COST}
       - coalesce((SELECT size + ${FILE_ROW_COST} FROM account_files WHERE account_id = ?1 AND id = ?2), 0)
  WHERE id = ?1 RETURNING id`,
	`INSERT INTO account_files (account_id, id, size, uploaded_at) SELECT ?1, ?2, ?3, ?4 FROM accounts WHERE id = ?1
  ON CONFLICT (account_id, id) DO UPDATE SET size = excluded.size, uploaded_at = excluded.uploaded_at`,
];

/** Undoes an upload's D1 batch after R2 failed, if the row still holds this upload's size. */
function compensation(db: D1Database, accountId: string, id: string, size: number, old: number | null, now: number): D1PreparedStatement[] {
	const holds = "EXISTS (SELECT 1 FROM account_files WHERE account_id = ?1 AND id = ?2 AND size = ?3)";
	if (old === null) {
		return [
			db
				.prepare(`UPDATE accounts SET files = files - 1, file_bytes = file_bytes - (?3 + ${FILE_ROW_COST}) WHERE id = ?1 AND ${holds}`)
				.bind(accountId, id, size),
			db.prepare("DELETE FROM account_files WHERE account_id = ?1 AND id = ?2 AND size = ?3").bind(accountId, id, size),
		];
	}
	// Restoring a larger old size grows file_bytes; recounted_at exempts the correction from the quota triggers.
	return [
		db
			.prepare(
				`UPDATE accounts SET file_bytes = file_bytes - ?3 + ?4, recounted_at = CASE WHEN ?4 > ?3 THEN ?5 ELSE recounted_at END
  WHERE id = ?1 AND ${holds}`,
			)
			.bind(accountId, id, size, old, now),
		db.prepare("UPDATE account_files SET size = ?4 WHERE account_id = ?1 AND id = ?2 AND size = ?3").bind(accountId, id, size, old),
	];
}

/** What streaming a file into R2 came to. */
type Streamed = "stored" | "checksum_mismatch" | "short" | "failed";

/**
 * Streams `body`, which must be exactly `size` bytes, into R2 under `key`,
 * never holding the file in memory: a 50 MiB PDF buffered (and copied once
 * more to join its chunks) would take most of an isolate's 128 MB, shared
 * with every other request it serves. R2 checks `sha256` and
 * FixedLengthStream the length before anything is stored, so a wrong
 * checksum, a short body or a failed write leaves the stored bytes as they
 * were. Our own digest of what passed says which of those it was.
 */
async function streamFile(env: Env, key: string, body: ReadableStream<Uint8Array>, size: number, sha256: string): Promise<Streamed> {
	const digest = new crypto.DigestStream("SHA-256");
	const digestWriter = digest.getWriter();
	const tap = new TransformStream<Uint8Array, Uint8Array>({
		async transform(chunk, controller) {
			await digestWriter.write(chunk);
			controller.enqueue(chunk);
		},
		async flush() {
			await digestWriter.close();
		},
	});
	const fixed = new FixedLengthStream(size);
	const abort = new AbortController();
	// Set from the pipe's callbacks, so in an object TypeScript doesn't narrow.
	const state: { stopped: boolean; piped: "pending" | "done" | "short" | "stopped" } = { stopped: false, piped: "pending" };
	const piping = body
		.pipeThrough(tap)
		.pipeTo(fixed.writable, { signal: abort.signal })
		.then(
			() => {
				state.piped = "done";
			},
			() => {
				state.piped = state.stopped ? "stopped" : "short";
			},
		);
	try {
		await env.FILES.put(key, fixed.readable, {
			sha256,
			httpMetadata: { contentType: "application/octet-stream" },
		});
		await piping;
		return "stored";
	} catch {
		// R2 may give up without reading the rest; stop the pipe so it can't wait on it.
		state.stopped = true;
		abort.abort();
		await piping;
		if (state.piped === "short") return "short";
		if (state.piped === "done") {
			const actual = [...new Uint8Array(await digest.digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
			if (actual !== sha256) return "checksum_mismatch";
		}
		return "failed";
	}
}

export function putFile(wireId: string) {
	return withSession(async (request, env, now, session) => {
		if (!isWireFileId(wireId)) return badRequest("Not a wire file ID.");
		const claimed = request.headers.get("x-kanso-sha256");
		if (claimed === null || !isSha256Hex(claimed)) return badRequest("x-kanso-sha256 must be 64 lowercase hex digits.");
		const limited = await count(env.DB, "writes-account", accountKey("writes-account", session.accountId), now, 1);
		if (limited) return limited;
		// The day's upload bytes, counted from Content-Length before the body is read,
		// refused uploads included (the largest file when it's missing; anything over
		// the largest counts as one byte more).
		const declared = request.headers.get("content-length");
		const known = declared !== null && /^[0-9]{1,15}$/.test(declared);
		const counted = known ? Math.min(Number(declared), MAX_SEALED_FILE + 1) : MAX_SEALED_FILE;
		const overBytes = await count(env.DB, "uploads-account", accountKey("uploads-account", session.accountId), now, counted);
		if (overBytes) return overBytes;
		let size: number;
		let body: ReadableStream<Uint8Array> | null;
		if (known) {
			// Every Kanso client says the length: the body is streamed to R2, never held.
			size = Number(declared);
			if (size > MAX_SEALED_FILE) return tooLarge(`A file is at most ${MAX_SEALED_FILE} bytes.`);
			body = request.body;
		} else {
			// A body of unknown length (chunked) is read whole, up to the largest file, to learn it.
			const bytes = await readBody(request, MAX_SEALED_FILE);
			if (!bytes) return tooLarge(`A file is at most ${MAX_SEALED_FILE} bytes.`);
			size = bytes.length;
			body = new Response(bytes).body;
		}
		if (size < MIN_SEALED_FILE || !body) return badRequest(`A sealed file is at least ${MIN_SEALED_FILE} bytes.`);

		let outcomes: D1Result[];
		try {
			outcomes = await env.DB.batch([
				env.DB.prepare(PUT_FILE[0]).bind(session.accountId, wireId),
				env.DB.prepare(PUT_FILE[1]).bind(session.accountId, wireId, size),
				env.DB.prepare(PUT_FILE[2]).bind(session.accountId, wireId, size, now),
			]);
		} catch (err) {
			const response = await refused(env, session.accountId, err, "file", now);
			if (response) return response;
			throw err;
		}
		if (outcomes[1].results.length !== 1) return unauthorized("unknown");
		const old = (outcomes[0].results as { size: number }[])[0]?.size ?? null;
		const streamed = await streamFile(env, fileKey(session.accountId, wireId), body, size, claimed);
		if (streamed === "stored") return json({ ok: true, size });
		// Nothing was stored: give back what the batch charged.
		await env.DB.batch(compensation(env.DB, session.accountId, wireId, size, old, now));
		if (streamed === "checksum_mismatch") return error(400, "checksum_mismatch", "The body doesn't match x-kanso-sha256.");
		if (streamed === "short") return badRequest("The body doesn't match Content-Length.");
		return error(500, "internal", "Couldn't store the file.");
	});
}

/**
 * The largest sealed file a client takes when it doesn't say
 * (`x-kanso-max-file`): the 10 MiB container of every Kanso before 50 MiB
 * PDFs. Such a client fails its whole sync on an answer over its cap, so a
 * bigger file is a 404 to it, which it counts as missing and gets past.
 */
export const LEGACY_MAX_FILE = 10_485_760 + 41 + 4_096;

/** The bytes only when the account has an `account_files` row for the ID, even if R2 holds an object. */
export function getFile(wireId: string) {
	return withSession(async (request, env, _now, session) => {
		if (!isWireFileId(wireId)) return badRequest("Not a wire file ID.");
		const row = await env.DB.prepare("SELECT size FROM account_files WHERE account_id = ?1 AND id = ?2")
			.bind(session.accountId, wireId)
			.first<{ size: number }>();
		if (!row) return notFound();
		const takes = request.headers.get("x-kanso-max-file");
		const max = takes !== null && /^[0-9]{1,15}$/.test(takes) ? Number(takes) : LEGACY_MAX_FILE;
		if (row.size > max) return error(404, "not_found", "This file is larger than this version of Kanso takes. Update Kanso to get it.");
		const object = await env.FILES.get(fileKey(session.accountId, wireId));
		if (!object) return notFound();
		return new Response(object.body, {
			headers: { "content-type": "application/octet-stream", "content-length": String(object.size) },
		});
	});
}
