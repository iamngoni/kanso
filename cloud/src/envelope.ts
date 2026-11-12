// The envelope everything written to a shared note travels in
// (docs/ACCOUNTS.md › Collaboration › Updates, snapshots, comments and
// presence), and the collaboration limits. The server reads only the header;
// it never holds a note key, so it can't open the rest.
//
//   header    = 0x01 || kind || u32(epoch) || lp(author_account) || lp(author_device) || u64(counter)
//   envelope  = header || nonce(24) || sealed (ciphertext and a 16-byte tag) || signature(64)
//
// `lp(x)` is a 2-byte big-endian length and the UTF-8 bytes. The author must
// be an account ID and the device a device ID, so a well-formed header is
// always 86 bytes. test/fixtures/collab.json holds the cases this parser and
// crates/kanso-collab's both run.

export const ENVELOPE_FORMAT_V1 = 0x01;
export const NONCE_BYTES = 24;
export const TAG_BYTES = 16;
export const SIGNATURE_BYTES = 64;
/** Format, kind, epoch, two length prefixes, an account ID, a device ID and the counter. */
export const HEADER_BYTES = 1 + 1 + 4 + 2 + 34 + 2 + 34 + 8;
/** The shortest envelope: a header, a nonce, an empty plaintext's tag, and a signature. */
export const MIN_ENVELOPE = HEADER_BYTES + NONCE_BYTES + TAG_BYTES + SIGNATURE_BYTES;

export type Kind = "u" | "s" | "c" | "p";
/** `manager` is for shared notebooks only; a shared note's member list refuses it (collab.ts). */
export type Role = "owner" | "manager" | "edit" | "comment" | "view";

/** The most bytes a whole envelope of each kind may have. */
export const MAX_ENVELOPE: Record<Kind, number> = {
	u: 256 * 1024,
	s: 4 * 1024 * 1024,
	c: 16 * 1024,
	p: 2 * 1024,
};

/** Who may write each kind: updates and snapshots need edit, comments comment, presence any member. */
export const ROLES_FOR: Record<Kind, readonly Role[]> = {
	u: ["edit", "manager", "owner"],
	s: ["edit", "manager", "owner"],
	c: ["comment", "edit", "manager", "owner"],
	p: ["view", "comment", "edit", "manager", "owner"],
};

export const mayWrite = (role: string, kind: Kind): boolean => (ROLES_FOR[kind] as readonly string[]).includes(role);

/** Updates and comments past the last snapshot after which the room wants a snapshot first. */
export const COMPACT_ENTRIES = 10_000;
export const COMPACT_BYTES = 16 * 1024 * 1024;
/** Bytes a room stores per UTC day. */
export const ROOM_BYTES_PER_DAY = 64 * 1024 * 1024;
/** WebSocket frames per second per socket, and the burst. */
export const SOCKET_RATE = 30;
export const SOCKET_BURST = 60;
/** Presence frames per second per account in a room, and the burst. */
export const PRESENCE_RATE = 10;
export const PRESENCE_BURST = 20;
/** Open sockets per account per room, and the `retry_after` past that. */
export const MAX_SOCKETS_PER_ACCOUNT = 8;
export const SOCKETS_RETRY_MS = 5_000;
/** Comments a room keeps, in envelope bytes (they are never compacted). */
export const MAX_COMMENT_BYTES = 8 * 1024 * 1024;
/** A room writes its size to D1 once it has moved this much, or this long after any change. */
export const FLUSH_BYTES = 64 * 1024;
export const FLUSH_DELAY_MS = 30_000;
/** A log page: entries by default and at most, and the envelope bytes it aims to stay under. */
export const DEFAULT_LOG_LIMIT = 500;
export const MAX_LOG_LIMIT = 1000;
export const LOG_PAGE_BYTES = 8 * 1024 * 1024;

export interface Header {
	kind: Kind;
	epoch: number;
	author: string;
	device: string;
	/** u64, as a bigint: it can pass 2^53. */
	counter: bigint;
}

const KINDS: Record<number, Kind> = { 0x75: "u", 0x73: "s", 0x63: "c", 0x70: "p" };
const ACCOUNT_ID = /^a_[0-9a-f]{32}$/;
const DEVICE_ID = /^d_[0-9a-f]{32}$/;
const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });

/**
 * The envelope's header, or null when the bytes aren't a well-formed envelope:
 * another format byte, an unknown kind, epoch 0, a length prefix running past
 * the end, an author or device that isn't UTF-8 or isn't an account or device
 * ID, or too few bytes left for the nonce, a tag and the signature. Never throws.
 */
export function parseEnvelope(bytes: Uint8Array): Header | null {
	if (bytes.length < MIN_ENVELOPE || bytes[0] !== ENVELOPE_FORMAT_V1) return null;
	const kind = KINDS[bytes[1]];
	if (!kind) return null;
	const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
	const epoch = view.getUint32(2);
	if (epoch === 0) return null;
	let at = 6;
	const lp = (): string | null => {
		if (at + 2 > bytes.length) return null;
		const length = view.getUint16(at);
		at += 2;
		if (at + length > bytes.length) return null;
		let text: string;
		try {
			text = utf8.decode(bytes.subarray(at, at + length));
		} catch {
			return null;
		}
		at += length;
		return text;
	};
	const author = lp();
	if (author === null || !ACCOUNT_ID.test(author)) return null;
	const device = lp();
	if (device === null || !DEVICE_ID.test(device)) return null;
	if (at + 8 > bytes.length) return null;
	const counter = view.getBigUint64(at);
	at += 8;
	if (bytes.length - at < NONCE_BYTES + TAG_BYTES + SIGNATURE_BYTES) return null;
	return { kind, epoch, author, device, counter };
}

/** `u64(seq) || envelope`: a frame the room sends to its sockets. */
export function frame(seq: number, envelope: Uint8Array): Uint8Array {
	const out = new Uint8Array(8 + envelope.length);
	new DataView(out.buffer).setBigUint64(0, BigInt(seq));
	out.set(envelope, 8);
	return out;
}

// ------------------------------------------------------------- notebook rooms
//
// A shared notebook's room multiplexes many logs (the notebook's manifest and
// each of its notes) over one set of sockets. A log is named by a 16-byte
// stream ID: all zeros for the manifest, the note's own 16 bytes otherwise.
// A frame to the room is `stream || envelope`; a frame from it is
// `stream || u64(seq) || envelope` (seq 0 for presence). `seq` counts every
// stored entry of the notebook, across its streams, so it orders them all and
// is what a new epoch's `from_seq` names.

export const STREAM_BYTES = 16;
export const MANIFEST_STREAM = new Uint8Array(STREAM_BYTES);
/** Notes per shared notebook (the manifest is not one). */
export const MAX_BOOK_NOTES = 2000;
/** Notes one socket may have open at a time (the manifest is always open). */
export const MAX_OPEN_STREAMS = 32;
/** Bytes a notebook's room stores per UTC day. */
export const BOOK_BYTES_PER_DAY = 256 * 1024 * 1024;

export const isManifest = (stream: Uint8Array): boolean => stream.every((b) => b === 0);

/** The manifest takes updates, snapshots and presence, never comments; notes take every kind their role allows. */
export const mayWriteStream = (role: string, kind: Kind, manifest: boolean): boolean => (!manifest || kind !== "c") && mayWrite(role, kind);

/** `stream || u64(seq) || envelope`. */
export function bookFrame(stream: Uint8Array, seq: number, envelope: Uint8Array): Uint8Array {
	const out = new Uint8Array(STREAM_BYTES + 8 + envelope.length);
	out.set(stream, 0);
	new DataView(out.buffer).setBigUint64(STREAM_BYTES, BigInt(seq));
	out.set(envelope, STREAM_BYTES + 8);
	return out;
}
