export interface Env {
  DB: D1Database;
  KANSO_JWT_SECRET?: string;
}

type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

interface RegisterRequest {
  email: string;
  password: string;
}

interface LoginRequest {
  email: string;
  password: string;
}

interface Claims {
  sub: string;
  device_id: string;
  iat: number;
  exp: number;
}

interface OutboxEvent {
  id: string;
  entity_type: string;
  entity_id: string;
  operation: string;
  payload: JsonValue;
  local_sequence: number;
}

interface PushRequest {
  device_id: string;
  last_known_server_seq: number;
  events: OutboxEvent[];
}

interface EventRow {
  server_sequence: number;
  event_id: string;
  entity_type: string;
  entity_id: string;
  operation: string;
  payload_json: string;
  local_sequence: number;
}

interface StoredEventRow extends EventRow {
  origin_device_id: string;
  created_at: number;
}

interface UserRow {
  user_id: string;
  password_hash: string;
}

interface UserEmailRow {
  email: string;
}

type ShareResourceType = "note" | "notebook";
type ShareRole = "owner" | "editor" | "viewer";

interface ShareMemberRequest {
  resource_type?: string;
  resource_id?: string;
  email?: string;
  role?: string;
}

interface ShareMemberRow {
  id: string;
  share_id: string;
  resource_type: ShareResourceType;
  resource_id: string;
  email: string;
  role: ShareRole;
  status: string;
  created_at: number;
  updated_at: number;
}

interface PendingShareRow {
  owner_user_id: string;
  member_id: string;
  resource_type: ShareResourceType;
  resource_id: string;
}

interface ShareEventContext {
  resource_type: ShareResourceType;
  resource_id: string;
  recipient_user_ids: string[];
}

interface ShareMemberPayload {
  resource_type: ShareResourceType;
  resource_id: string;
  email: string;
  role: ShareRole;
  status: string;
  created_at: number;
  updated_at: number;
}

interface SharedWriteContext {
  owner_user_id: string;
  resource_type: ShareResourceType;
  resource_id: string;
  role: ShareRole;
}

const TOKEN_TTL_SECONDS = 60 * 60 * 24 * 30;
const PASSWORD_ITERATIONS = 100_000;
const encoder = new TextEncoder();

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: corsHeaders() });
    }

    try {
      return await route(request, env);
    } catch (error) {
      if (error instanceof HttpError) {
        return json({ error: error.message }, error.status);
      }
      console.error(error);
      return json({ error: "internal server error" }, 500);
    }
  },
};

async function route(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);
  const path = url.pathname.replace(/\/+$/, "") || "/";

  if (request.method === "GET" && path === "/health") {
    return json({ status: "ok" });
  }

  if (request.method === "POST" && path === "/v1/auth/register") {
    return handleRegister(request, env);
  }

  if (request.method === "POST" && path === "/v1/auth/login") {
    return handleLogin(request, env);
  }

  if (request.method === "POST" && path === "/v1/auth/refresh") {
    const claims = await requireAuth(request, env);
    return issueSession(env, claims.sub, claims.device_id);
  }

  if (request.method === "POST" && path === "/v1/sync/push") {
    return handlePush(request, env);
  }

  if (request.method === "GET" && path === "/v1/sync/pull") {
    return handlePull(request, env, url);
  }

  const shareMemberMatch = path.match(/^\/v1\/shares\/members\/([^/]+)$/);
  if (shareMemberMatch && request.method === "DELETE") {
    return handleRemoveShareMember(request, env, decodeURIComponent(shareMemberMatch[1]));
  }

  if (path === "/v1/shares/members") {
    if (request.method === "GET") return handleListShareMembers(request, env, url);
    if (request.method === "POST") return handleAddShareMember(request, env);
  }

  const blobMatch = path.match(/^\/v1\/blobs\/([a-f0-9]{64})(\/exists)?$/);
  if (blobMatch) {
    const hash = blobMatch[1];
    const existsOnly = Boolean(blobMatch[2]);
    if (request.method === "PUT" && !existsOnly) return putBlob(request, env, hash);
    if (request.method === "GET" && existsOnly) return blobExists(request, env, hash);
    if (request.method === "GET" && !existsOnly) return getBlob(request, env, hash);
  }

  return json({ error: "not found" }, 404);
}

async function handleRegister(request: Request, env: Env): Promise<Response> {
  const body = await readJson<RegisterRequest>(request);
  const email = normalizeEmail(body.email);
  if (!email || !body.password) throw new HttpError(400, "email and password are required");

  const userId = `user:${crypto.randomUUID()}`;
  const passwordHash = await hashPassword(body.password);
  const now = Date.now();

  try {
    await env.DB.prepare(
      "INSERT INTO users (user_id, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
    )
      .bind(userId, email, passwordHash, now)
      .run();
  } catch (error) {
    if (String(error).toLowerCase().includes("unique")) {
      throw new HttpError(409, "email already registered");
    }
    throw error;
  }

  await backfillPendingSharesForUser(env, userId, email);

  return issueSession(env, userId);
}

async function handleLogin(request: Request, env: Env): Promise<Response> {
  const body = await readJson<LoginRequest>(request);
  const email = normalizeEmail(body.email);
  const row = await env.DB.prepare("SELECT user_id, password_hash FROM users WHERE email = ?")
    .bind(email)
    .first<UserRow>();

  if (!row || !(await verifyPassword(body.password ?? "", row.password_hash))) {
    throw new HttpError(401, "invalid credentials");
  }

  await backfillPendingSharesForUser(env, row.user_id, email);

  return issueSession(env, row.user_id);
}

async function issueSession(env: Env, userId: string, existingDeviceId?: string): Promise<Response> {
  const deviceId = existingDeviceId ?? `device:${crypto.randomUUID()}`;
  const now = Date.now();

  if (!existingDeviceId) {
    await env.DB.prepare(
      "INSERT INTO devices (device_id, user_id, name, created_at) VALUES (?, ?, ?, ?)",
    )
      .bind(deviceId, userId, "device", now)
      .run();
  }

  const token = await signJwt(env, {
    sub: userId,
    device_id: deviceId,
    iat: Math.floor(now / 1000),
    exp: Math.floor(now / 1000) + TOKEN_TTL_SECONDS,
  });

  return json({ token, user_id: userId, device_id: deviceId });
}

async function handlePush(request: Request, env: Env): Promise<Response> {
  const claims = await requireAuth(request, env);
  const body = await readJson<PushRequest>(request);
  if (!Array.isArray(body.events)) throw new HttpError(400, "events must be an array");

  const acceptedIds: string[] = [];
  const now = Date.now();

  for (const event of body.events) {
    validateEvent(event);
    const preMutationContext = await shareEventContextBeforeMutation(env, claims.sub, event);
    const sharedWriteContexts = await sharedWriteContextsForEvent(env, claims.sub, event);
    const writableContexts = sharedWriteContexts.filter((context) => context.role !== "viewer");
    if (sharedWriteContexts.length > 0 && writableContexts.length === 0) {
      throw new HttpError(403, "viewer cannot modify shared resources");
    }
    const existing = await env.DB.prepare("SELECT server_sequence FROM events WHERE user_id = ? AND event_id = ?")
      .bind(claims.sub, event.id)
      .first<{ server_sequence: number }>();
    if (existing) {
      acceptedIds.push(event.id);
      continue;
    }

    const sequence = await nextServerSequence(env, claims.sub);
    let inserted = false;
    try {
      await env.DB.prepare(
        "INSERT INTO events " +
          "(user_id, server_sequence, event_id, origin_device_id, entity_type, entity_id, operation, payload_json, local_sequence, created_at) " +
          "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
      )
        .bind(
          claims.sub,
          sequence,
          event.id,
          claims.device_id,
          event.entity_type,
          event.entity_id,
          event.operation,
          JSON.stringify(event.payload ?? {}),
          event.local_sequence,
          now,
        )
        .run();
      inserted = true;
    } catch (error) {
      if (!String(error).toLowerCase().includes("unique")) throw error;
    }
    if (inserted) {
      const sharePayload = await applyShareIndexMutation(env, claims.sub, event, now);
      if (sharePayload?.resource_type === "note") {
        await backfillDirectNoteShare(env, claims.sub, sharePayload, event, claims.device_id, now);
      } else if (sharePayload?.resource_type === "notebook") {
        await backfillNotebookShare(env, claims.sub, sharePayload, event, claims.device_id, now);
      }
      await mirrorEventToShareRecipients(
        env,
        claims.sub,
        event,
        claims.device_id,
        now,
        preMutationContext,
      );
      await fanOutSharedMemberWrite(env, claims.sub, writableContexts, event, claims.device_id, now);
    }
    acceptedIds.push(event.id);
  }

  return json({
    accepted_ids: acceptedIds,
    server_high_water: await highWater(env, claims.sub),
  });
}

async function shareEventContextBeforeMutation(
  env: Env,
  ownerUserId: string,
  event: OutboxEvent,
): Promise<ShareEventContext | null> {
  if (event.operation === "attachment_deleted" || event.operation === "sketch_deleted") {
    const noteId = await noteIdForEntityHistory(env, ownerUserId, event.entity_type, event.entity_id);
    if (!noteId) return null;
    return {
      resource_type: "note",
      resource_id: noteId,
      recipient_user_ids: await recipientUserIdsForNoteId(env, ownerUserId, noteId),
    };
  }

  if (event.operation !== "share_member_removed") return null;

  const row = await env.DB.prepare(
    "SELECT s.resource_type, s.resource_id " +
      "FROM share_members m " +
      "JOIN shares s ON s.user_id = m.user_id AND s.id = m.share_id " +
      "WHERE m.user_id = ? AND m.id = ?",
  )
    .bind(ownerUserId, event.entity_id)
    .first<{ resource_type: ShareResourceType; resource_id: string }>();
  if (!row) return null;

  return {
    resource_type: row.resource_type,
    resource_id: row.resource_id,
    recipient_user_ids: await registeredMemberUserIds(
      env,
      ownerUserId,
      row.resource_type,
      row.resource_id,
    ),
  };
}

async function applyShareIndexMutation(
  env: Env,
  ownerUserId: string,
  event: OutboxEvent,
  now: number,
): Promise<ShareMemberPayload | null> {
  if (event.operation === "share_member_added") {
    const payload = shareMemberPayload(event.payload);
    if (!payload) return null;

    const shareId = await ensureShare(env, ownerUserId, payload.resource_type, payload.resource_id, now);
    await env.DB.prepare(
      "INSERT INTO share_members (user_id, id, share_id, email, role, status, created_at, updated_at) " +
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?) " +
        "ON CONFLICT(user_id, share_id, email) DO UPDATE SET " +
        "id = excluded.id, role = excluded.role, status = excluded.status, updated_at = excluded.updated_at",
    )
      .bind(
        ownerUserId,
        event.entity_id,
        shareId,
        payload.email,
        payload.role,
        payload.status,
        payload.created_at,
        payload.updated_at,
      )
      .run();

    return payload;
  }

  if (event.operation === "share_member_removed") {
    await env.DB.prepare("DELETE FROM share_members WHERE user_id = ? AND id = ?")
      .bind(ownerUserId, event.entity_id)
      .run();
  }

  return null;
}

async function backfillDirectNoteShare(
  env: Env,
  ownerUserId: string,
  payload: ShareMemberPayload,
  currentEvent: OutboxEvent,
  ownerDeviceId: string,
  now: number,
): Promise<void> {
  const recipient = await env.DB.prepare("SELECT user_id FROM users WHERE email = ?")
    .bind(payload.email)
    .first<{ user_id: string }>();
  if (!recipient || recipient.user_id === ownerUserId) return;

  const notebookIds = await notebookIdsReferencedByNoteHistory(env, ownerUserId, payload.resource_id);
  for (const notebookId of notebookIds) {
    await copyOwnerEventsToUser(env, ownerUserId, recipient.user_id, "notebook", notebookId);
  }
  await copyOwnerEventsToUser(env, ownerUserId, recipient.user_id, "note", payload.resource_id);
  await copyRelatedNoteEventsToUser(env, ownerUserId, recipient.user_id, payload.resource_id);
  await copyOutboxEventToUser(env, ownerUserId, recipient.user_id, currentEvent, ownerDeviceId, now);
}

async function backfillDirectNoteShareToUser(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  noteId: string,
): Promise<void> {
  const notebookIds = await notebookIdsReferencedByNoteHistory(env, ownerUserId, noteId);
  for (const notebookId of notebookIds) {
    await copyOwnerEventsToUser(env, ownerUserId, recipientUserId, "notebook", notebookId);
  }
  await copyOwnerEventsToUser(env, ownerUserId, recipientUserId, "note", noteId);
  await copyRelatedNoteEventsToUser(env, ownerUserId, recipientUserId, noteId);
}

async function backfillNotebookShare(
  env: Env,
  ownerUserId: string,
  payload: ShareMemberPayload,
  currentEvent: OutboxEvent,
  ownerDeviceId: string,
  now: number,
): Promise<void> {
  const recipient = await env.DB.prepare("SELECT user_id FROM users WHERE email = ?")
    .bind(payload.email)
    .first<{ user_id: string }>();
  if (!recipient || recipient.user_id === ownerUserId) return;

  await backfillNotebookShareToUser(env, ownerUserId, recipient.user_id, payload.resource_id);
  await copyOutboxEventToUser(env, ownerUserId, recipient.user_id, currentEvent, ownerDeviceId, now);
}

async function backfillNotebookShareToUser(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  notebookId: string,
): Promise<void> {
  await copyOwnerEventsToUser(env, ownerUserId, recipientUserId, "notebook", notebookId);
  const noteIds = await noteIdsCurrentlyInNotebook(env, ownerUserId, notebookId);
  for (const noteId of noteIds) {
    await copyOwnerEventsToUser(env, ownerUserId, recipientUserId, "note", noteId);
    await copyRelatedNoteEventsToUser(env, ownerUserId, recipientUserId, noteId);
  }
}

async function backfillPendingSharesForUser(
  env: Env,
  recipientUserId: string,
  email: string,
): Promise<void> {
  const result = await env.DB.prepare(
    "SELECT s.user_id AS owner_user_id, m.id AS member_id, s.resource_type, s.resource_id " +
      "FROM share_members m " +
      "JOIN shares s ON s.user_id = m.user_id AND s.id = m.share_id " +
      "WHERE m.email = ? AND m.role IN ('owner', 'editor', 'viewer')",
  )
    .bind(email)
    .all<PendingShareRow>();

  for (const share of result.results ?? []) {
    if (share.owner_user_id === recipientUserId) continue;
    if (share.resource_type === "note") {
      await backfillDirectNoteShareToUser(
        env,
        share.owner_user_id,
        recipientUserId,
        share.resource_id,
      );
    } else {
      await backfillNotebookShareToUser(
        env,
        share.owner_user_id,
        recipientUserId,
        share.resource_id,
      );
    }
    await copyOwnerEventsToUser(
      env,
      share.owner_user_id,
      recipientUserId,
      "share_member",
      share.member_id,
    );
  }
}

async function mirrorEventToShareRecipients(
  env: Env,
  ownerUserId: string,
  event: OutboxEvent,
  originDeviceId: string,
  now: number,
  preMutationContext: ShareEventContext | null,
): Promise<void> {
  let recipientUserIds: string[] = [];

  if (preMutationContext?.resource_type === "note") {
    recipientUserIds = preMutationContext.recipient_user_ids;
  } else if (preMutationContext?.resource_type === "notebook") {
    recipientUserIds = preMutationContext.recipient_user_ids;
  } else if (event.entity_type === "note") {
    recipientUserIds = await recipientUserIdsForNoteEvent(env, ownerUserId, event);
  } else if (event.entity_type === "attachment" || event.entity_type === "sketch") {
    const noteId =
      stringField(event.payload, "note_id") ??
      (await noteIdForEntityHistory(env, ownerUserId, event.entity_type, event.entity_id));
    recipientUserIds = noteId ? await recipientUserIdsForNoteId(env, ownerUserId, noteId) : [];
  } else if (event.entity_type === "notebook") {
    recipientUserIds = await registeredMemberUserIds(env, ownerUserId, "notebook", event.entity_id);
  } else if (event.operation === "share_member_added") {
    const payload = shareMemberPayload(event.payload);
    if (payload) {
      recipientUserIds = await registeredMemberUserIds(
        env,
        ownerUserId,
        payload.resource_type,
        payload.resource_id,
      );
    }
  }

  for (const recipientUserId of distinct(recipientUserIds)) {
    if (recipientUserId !== ownerUserId) {
      await copyOutboxEventToUser(env, ownerUserId, recipientUserId, event, originDeviceId, now);
    }
  }
}

async function sharedWriteContextsForEvent(
  env: Env,
  writerUserId: string,
  event: OutboxEvent,
): Promise<SharedWriteContext[]> {
  if (!isSharedContentMutation(event)) return [];

  const email = await emailForUser(env, writerUserId);
  if (!email) return [];

  const noteId = await noteIdForContentEvent(env, writerUserId, event);
  const contexts: SharedWriteContext[] = [];

  if (noteId) {
    contexts.push(...(await shareContextsForMember(env, writerUserId, email, "note", noteId)));
    const notebookId =
      stringField(event.payload, "notebook_id") ??
      (await latestNotebookIdForNote(env, writerUserId, noteId));
    if (notebookId) {
      contexts.push(
        ...(await shareContextsForMember(env, writerUserId, email, "notebook", notebookId)),
      );
    }
  } else if (event.entity_type === "notebook") {
    contexts.push(
      ...(await shareContextsForMember(env, writerUserId, email, "notebook", event.entity_id)),
    );
  }

  return dedupeWriteContexts(contexts);
}

function isSharedContentMutation(event: OutboxEvent): boolean {
  return (
    event.entity_type === "note" ||
    event.entity_type === "attachment" ||
    event.entity_type === "sketch" ||
    event.entity_type === "notebook"
  );
}

async function noteIdForContentEvent(
  env: Env,
  writerUserId: string,
  event: OutboxEvent,
): Promise<string | null> {
  if (event.entity_type === "note") return event.entity_id;
  if (event.entity_type === "attachment" || event.entity_type === "sketch") {
    return (
      stringField(event.payload, "note_id") ??
      (await noteIdForEntityHistory(env, writerUserId, event.entity_type, event.entity_id))
    );
  }
  return null;
}

async function shareContextsForMember(
  env: Env,
  writerUserId: string,
  email: string,
  resourceType: ShareResourceType,
  resourceId: string,
): Promise<SharedWriteContext[]> {
  const result = await env.DB.prepare(
    "SELECT s.user_id AS owner_user_id, s.resource_type, s.resource_id, m.role " +
      "FROM shares s " +
      "JOIN share_members m ON m.user_id = s.user_id AND m.share_id = s.id " +
      "WHERE s.user_id <> ? AND m.email = ? AND s.resource_type = ? AND s.resource_id = ? " +
      "AND m.role IN ('owner', 'editor', 'viewer')",
  )
    .bind(writerUserId, email, resourceType, resourceId)
    .all<SharedWriteContext>();

  return result.results ?? [];
}

async function fanOutSharedMemberWrite(
  env: Env,
  writerUserId: string,
  contexts: SharedWriteContext[],
  event: OutboxEvent,
  originDeviceId: string,
  now: number,
): Promise<void> {
  for (const context of dedupeWriteContexts(contexts)) {
    await copyOutboxEventToUser(env, writerUserId, context.owner_user_id, event, originDeviceId, now);
    const recipientUserIds = await registeredMemberUserIds(
      env,
      context.owner_user_id,
      context.resource_type,
      context.resource_id,
    );
    for (const recipientUserId of recipientUserIds) {
      if (recipientUserId !== writerUserId) {
        await copyOutboxEventToUser(env, writerUserId, recipientUserId, event, originDeviceId, now);
      }
    }
  }
}

async function emailForUser(env: Env, userId: string): Promise<string | null> {
  const row = await env.DB.prepare("SELECT email FROM users WHERE user_id = ?")
    .bind(userId)
    .first<UserEmailRow>();
  return row?.email ?? null;
}

function dedupeWriteContexts(contexts: SharedWriteContext[]): SharedWriteContext[] {
  const byKey = new Map<string, SharedWriteContext>();
  for (const context of contexts) {
    const key = `${context.owner_user_id}:${context.resource_type}:${context.resource_id}`;
    const existing = byKey.get(key);
    if (!existing || roleRank(context.role) > roleRank(existing.role)) {
      byKey.set(key, context);
    }
  }
  return Array.from(byKey.values());
}

function roleRank(role: ShareRole): number {
  if (role === "owner") return 3;
  if (role === "editor") return 2;
  return 1;
}

async function registeredMemberUserIds(
  env: Env,
  ownerUserId: string,
  resourceType: ShareResourceType,
  resourceId: string,
): Promise<string[]> {
  const result = await env.DB.prepare(
    "SELECT u.user_id " +
      "FROM shares s " +
      "JOIN share_members m ON m.user_id = s.user_id AND m.share_id = s.id " +
      "JOIN users u ON u.email = m.email " +
      "WHERE s.user_id = ? AND s.resource_type = ? AND s.resource_id = ? " +
      "AND m.role IN ('owner', 'editor', 'viewer')",
  )
    .bind(ownerUserId, resourceType, resourceId)
    .all<{ user_id: string }>();

  return (result.results ?? []).map((row) => row.user_id);
}

async function recipientUserIdsForNoteEvent(
  env: Env,
  ownerUserId: string,
  event: OutboxEvent,
): Promise<string[]> {
  return recipientUserIdsForNoteId(env, ownerUserId, event.entity_id);
}

async function recipientUserIdsForNoteId(
  env: Env,
  ownerUserId: string,
  noteId: string,
): Promise<string[]> {
  const recipients = await registeredMemberUserIds(env, ownerUserId, "note", noteId);
  const notebookId = await latestNotebookIdForNote(env, ownerUserId, noteId);
  if (notebookId) {
    recipients.push(...(await registeredMemberUserIds(env, ownerUserId, "notebook", notebookId)));
  }
  return distinct(recipients);
}

async function notebookIdsReferencedByNoteHistory(
  env: Env,
  ownerUserId: string,
  noteId: string,
): Promise<string[]> {
  const result = await env.DB.prepare(
    "SELECT payload_json FROM events " +
      "WHERE user_id = ? AND entity_type = 'note' AND entity_id = ? " +
      "AND operation IN ('note_created', 'note_moved') " +
      "ORDER BY server_sequence",
  )
    .bind(ownerUserId, noteId)
    .all<{ payload_json: string }>();

  const notebookIds: string[] = [];
  for (const row of result.results ?? []) {
    const payload = parsePayload(row.payload_json);
    const notebookId = stringField(payload, "notebook_id");
    if (notebookId) notebookIds.push(notebookId);
  }
  return distinct(notebookIds);
}

async function latestNotebookIdForNote(
  env: Env,
  ownerUserId: string,
  noteId: string,
): Promise<string | null> {
  const row = await env.DB.prepare(
    "SELECT payload_json FROM events " +
      "WHERE user_id = ? AND entity_type = 'note' AND entity_id = ? " +
      "AND operation IN ('note_created', 'note_moved') " +
      "ORDER BY server_sequence DESC LIMIT 1",
  )
    .bind(ownerUserId, noteId)
    .first<{ payload_json: string }>();
  return row ? stringField(parsePayload(row.payload_json), "notebook_id") : null;
}

async function noteIdsCurrentlyInNotebook(
  env: Env,
  ownerUserId: string,
  notebookId: string,
): Promise<string[]> {
  const result = await env.DB.prepare(
    "SELECT entity_id, payload_json FROM events " +
      "WHERE user_id = ? AND entity_type = 'note' " +
      "AND operation IN ('note_created', 'note_moved') " +
      "ORDER BY server_sequence",
  )
    .bind(ownerUserId)
    .all<{ entity_id: string; payload_json: string }>();

  const noteNotebookIds = new Map<string, string>();
  for (const row of result.results ?? []) {
    const nextNotebookId = stringField(parsePayload(row.payload_json), "notebook_id");
    if (nextNotebookId) noteNotebookIds.set(row.entity_id, nextNotebookId);
  }

  return Array.from(noteNotebookIds.entries())
    .filter(([, currentNotebookId]) => currentNotebookId === notebookId)
    .map(([noteId]) => noteId);
}

async function noteIdForEntityHistory(
  env: Env,
  ownerUserId: string,
  entityType: string,
  entityId: string,
): Promise<string | null> {
  const row = await env.DB.prepare(
    "SELECT payload_json FROM events " +
      "WHERE user_id = ? AND entity_type = ? AND entity_id = ? " +
      "ORDER BY server_sequence DESC LIMIT 1",
  )
    .bind(ownerUserId, entityType, entityId)
    .first<{ payload_json: string }>();
  return row ? stringField(parsePayload(row.payload_json), "note_id") : null;
}

async function copyRelatedNoteEventsToUser(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  noteId: string,
): Promise<void> {
  await copyOwnerEventsForPayloadNote(env, ownerUserId, recipientUserId, "attachment", noteId);
  await copyOwnerEventsForPayloadNote(env, ownerUserId, recipientUserId, "sketch", noteId);
}

async function copyOwnerEventsForPayloadNote(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  entityType: string,
  noteId: string,
): Promise<void> {
  const result = await env.DB.prepare(
    "SELECT server_sequence, event_id, origin_device_id, entity_type, entity_id, operation, payload_json, local_sequence, created_at " +
      "FROM events WHERE user_id = ? AND entity_type = ? " +
      "ORDER BY server_sequence",
  )
    .bind(ownerUserId, entityType)
    .all<StoredEventRow>();

  const entityIdsForNote = new Set<string>();
  for (const row of result.results ?? []) {
    const payloadNoteId = stringField(parsePayload(row.payload_json), "note_id");
    if (payloadNoteId === noteId) entityIdsForNote.add(row.entity_id);
    if (entityIdsForNote.has(row.entity_id)) {
      await copyStoredEventToUser(env, ownerUserId, recipientUserId, row);
    }
  }
}

async function copyOwnerEventsToUser(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  entityType: string,
  entityId: string,
): Promise<void> {
  const result = await env.DB.prepare(
    "SELECT server_sequence, event_id, origin_device_id, entity_type, entity_id, operation, payload_json, local_sequence, created_at " +
      "FROM events WHERE user_id = ? AND entity_type = ? AND entity_id = ? " +
      "ORDER BY server_sequence",
  )
    .bind(ownerUserId, entityType, entityId)
    .all<StoredEventRow>();

  for (const row of result.results ?? []) {
    await copyStoredEventToUser(env, ownerUserId, recipientUserId, row);
  }
}

async function copyStoredEventToUser(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  row: StoredEventRow,
): Promise<void> {
  const existing = await env.DB.prepare("SELECT 1 AS exists_row FROM events WHERE user_id = ? AND event_id = ?")
    .bind(recipientUserId, row.event_id)
    .first<{ exists_row: number }>();
  if (existing) {
    await copyBlobsReferencedByEvent(env, ownerUserId, recipientUserId, row.entity_type, parsePayload(row.payload_json));
    return;
  }

  const sequence = await nextServerSequence(env, recipientUserId);
  await env.DB.prepare(
    "INSERT INTO events " +
      "(user_id, server_sequence, event_id, origin_device_id, entity_type, entity_id, operation, payload_json, local_sequence, created_at) " +
      "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
  )
    .bind(
      recipientUserId,
      sequence,
      row.event_id,
      row.origin_device_id,
      row.entity_type,
      row.entity_id,
      row.operation,
      row.payload_json,
      row.local_sequence,
      row.created_at,
    )
    .run();
  await copyBlobsReferencedByEvent(env, ownerUserId, recipientUserId, row.entity_type, parsePayload(row.payload_json));
}

async function copyOutboxEventToUser(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  event: OutboxEvent,
  originDeviceId: string,
  now: number,
): Promise<void> {
  const existing = await env.DB.prepare("SELECT 1 AS exists_row FROM events WHERE user_id = ? AND event_id = ?")
    .bind(recipientUserId, event.id)
    .first<{ exists_row: number }>();
  if (existing) {
    await copyBlobsReferencedByEvent(env, ownerUserId, recipientUserId, event.entity_type, event.payload);
    return;
  }

  const sequence = await nextServerSequence(env, recipientUserId);
  await env.DB.prepare(
    "INSERT INTO events " +
      "(user_id, server_sequence, event_id, origin_device_id, entity_type, entity_id, operation, payload_json, local_sequence, created_at) " +
      "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
  )
    .bind(
      recipientUserId,
      sequence,
      event.id,
      originDeviceId,
      event.entity_type,
      event.entity_id,
      event.operation,
      JSON.stringify(event.payload ?? {}),
      event.local_sequence,
      now,
    )
    .run();
  await copyBlobsReferencedByEvent(env, ownerUserId, recipientUserId, event.entity_type, event.payload);
}

async function copyBlobsReferencedByEvent(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  entityType: string,
  payload: JsonValue,
): Promise<void> {
  if (entityType !== "attachment") return;
  const hash = stringField(payload, "content_hash");
  if (!hash) return;
  await copyBlobToUser(env, ownerUserId, recipientUserId, hash);
}

async function copyBlobToUser(
  env: Env,
  ownerUserId: string,
  recipientUserId: string,
  hash: string,
): Promise<void> {
  if (ownerUserId === recipientUserId) return;
  const blob = await env.DB.prepare(
    "SELECT body_base64, size_bytes, created_at FROM blobs WHERE user_id = ? AND hash = ?",
  )
    .bind(ownerUserId, hash)
    .first<{ body_base64: string; size_bytes: number; created_at: number }>();
  if (!blob) return;

  await env.DB.prepare(
    "INSERT INTO blobs (user_id, hash, body_base64, size_bytes, created_at) VALUES (?, ?, ?, ?, ?) " +
      "ON CONFLICT(user_id, hash) DO UPDATE SET body_base64 = excluded.body_base64, size_bytes = excluded.size_bytes",
  )
    .bind(recipientUserId, hash, blob.body_base64, blob.size_bytes, blob.created_at)
    .run();
}

async function copyBlobToCurrentShareRecipients(
  env: Env,
  ownerUserId: string,
  hash: string,
): Promise<void> {
  const noteIds = await noteIdsForAttachmentHash(env, ownerUserId, hash);
  const recipientUserIds: string[] = [];
  for (const noteId of noteIds) {
    recipientUserIds.push(...(await recipientUserIdsForNoteId(env, ownerUserId, noteId)));
  }

  for (const recipientUserId of distinct(recipientUserIds)) {
    await copyBlobToUser(env, ownerUserId, recipientUserId, hash);
  }
}

async function noteIdsForAttachmentHash(
  env: Env,
  ownerUserId: string,
  hash: string,
): Promise<string[]> {
  const result = await env.DB.prepare(
    "SELECT payload_json FROM events WHERE user_id = ? AND entity_type = 'attachment' AND payload_json LIKE ?",
  )
    .bind(ownerUserId, `%${hash}%`)
    .all<{ payload_json: string }>();

  const noteIds: string[] = [];
  for (const row of result.results ?? []) {
    const payload = parsePayload(row.payload_json);
    if (stringField(payload, "content_hash") === hash) {
      const noteId = stringField(payload, "note_id");
      if (noteId) noteIds.push(noteId);
    }
  }
  return distinct(noteIds);
}

async function nextServerSequence(env: Env, userId: string): Promise<number> {
  const row = await env.DB.prepare(
    "INSERT INTO user_sequences (user_id, value) VALUES (?, 1) " +
      "ON CONFLICT(user_id) DO UPDATE SET value = value + 1 RETURNING value",
  )
    .bind(userId)
    .first<{ value: number }>();
  if (!row) throw new HttpError(500, "failed to allocate server sequence");
  return row.value;
}

async function handlePull(request: Request, env: Env, url: URL): Promise<Response> {
  const claims = await requireAuth(request, env);
  const since = integerParam(url, "since", 0);
  const limit = Math.min(Math.max(integerParam(url, "limit", 500), 1), 5_000);

  const result = await env.DB.prepare(
    "SELECT server_sequence, event_id, entity_type, entity_id, operation, payload_json, local_sequence " +
      "FROM events WHERE user_id = ? AND server_sequence > ? AND origin_device_id <> ? " +
      "ORDER BY server_sequence LIMIT ?",
  )
    .bind(claims.sub, since, claims.device_id, limit)
    .all<EventRow>();

  const changes = (result.results ?? []).map((row) => ({
    server_sequence: row.server_sequence,
    id: row.event_id,
    entity_type: row.entity_type,
    entity_id: row.entity_id,
    operation: row.operation,
    payload: parsePayload(row.payload_json),
    local_sequence: row.local_sequence,
  }));

  return json({
    changes,
    server_high_water: await highWater(env, claims.sub),
  });
}

async function putBlob(request: Request, env: Env, hash: string): Promise<Response> {
  const claims = await requireAuth(request, env);
  const bytes = new Uint8Array(await request.arrayBuffer());
  const actual = await sha256Hex(bytes);
  if (actual !== hash) {
    return json({ error: "content hash mismatch", expected: hash, actual }, 400);
  }

  await env.DB.prepare(
    "INSERT INTO blobs (user_id, hash, body_base64, size_bytes, created_at) VALUES (?, ?, ?, ?, ?) " +
      "ON CONFLICT(user_id, hash) DO UPDATE SET body_base64 = excluded.body_base64, size_bytes = excluded.size_bytes",
  )
    .bind(claims.sub, hash, bytesToBase64Url(bytes), bytes.byteLength, Date.now())
    .run();

  await copyBlobToCurrentShareRecipients(env, claims.sub, hash);

  return json({ hash, size: bytes.byteLength });
}

async function getBlob(request: Request, env: Env, hash: string): Promise<Response> {
  const claims = await requireAuth(request, env);
  const row = await env.DB.prepare("SELECT body_base64 FROM blobs WHERE user_id = ? AND hash = ?")
    .bind(claims.sub, hash)
    .first<{ body_base64: string }>();
  if (!row) return new Response(null, { status: 404, headers: corsHeaders() });

  return new Response(base64UrlToBytes(row.body_base64), {
    status: 200,
    headers: {
      ...corsHeaders(),
      "content-type": "application/octet-stream",
    },
  });
}

async function blobExists(request: Request, env: Env, hash: string): Promise<Response> {
  const claims = await requireAuth(request, env);
  const row = await env.DB.prepare("SELECT 1 AS exists_blob FROM blobs WHERE user_id = ? AND hash = ?")
    .bind(claims.sub, hash)
    .first<{ exists_blob: number }>();
  return json({ exists: Boolean(row) });
}

async function handleListShareMembers(request: Request, env: Env, url: URL): Promise<Response> {
  const claims = await requireAuth(request, env);
  const resourceType = normalizeShareResourceType(url.searchParams.get("resource_type"));
  const resourceId = normalizeRequired(url.searchParams.get("resource_id"), "resource_id");

  const result = await env.DB.prepare(
    "SELECT m.id, m.share_id, s.resource_type, s.resource_id, m.email, m.role, m.status, m.created_at, m.updated_at " +
      "FROM share_members m " +
      "JOIN shares s ON s.user_id = m.user_id AND s.id = m.share_id " +
      "WHERE m.user_id = ? AND s.resource_type = ? AND s.resource_id = ? " +
      "ORDER BY CASE m.role WHEN 'owner' THEN 0 WHEN 'editor' THEN 1 ELSE 2 END, m.email",
  )
    .bind(claims.sub, resourceType, resourceId)
    .all<ShareMemberRow>();

  return json({ members: result.results ?? [] });
}

async function handleAddShareMember(request: Request, env: Env): Promise<Response> {
  const claims = await requireAuth(request, env);
  const body = await readJson<ShareMemberRequest>(request);
  const resourceType = normalizeShareResourceType(body.resource_type);
  const resourceId = normalizeRequired(body.resource_id, "resource_id");
  const email = normalizeShareEmail(body.email);
  const role = normalizeShareRole(body.role);
  const now = Date.now();
  const shareId = await ensureShare(env, claims.sub, resourceType, resourceId, now);
  const memberId = `sharemember:${crypto.randomUUID()}`;

  await env.DB.prepare(
    "INSERT INTO share_members (user_id, id, share_id, email, role, status, created_at, updated_at) " +
      "VALUES (?, ?, ?, ?, ?, 'invited', ?, ?) " +
      "ON CONFLICT(user_id, share_id, email) DO UPDATE SET " +
      "role = excluded.role, status = 'invited', updated_at = excluded.updated_at",
  )
    .bind(claims.sub, memberId, shareId, email, role, now, now)
    .run();

  const member = await env.DB.prepare(
    "SELECT m.id, m.share_id, s.resource_type, s.resource_id, m.email, m.role, m.status, m.created_at, m.updated_at " +
      "FROM share_members m " +
      "JOIN shares s ON s.user_id = m.user_id AND s.id = m.share_id " +
      "WHERE m.user_id = ? AND s.id = ? AND m.email = ?",
  )
    .bind(claims.sub, shareId, email)
    .first<ShareMemberRow>();
  if (!member) throw new HttpError(500, "failed to save share member");

  return json({ member }, 201);
}

async function handleRemoveShareMember(
  request: Request,
  env: Env,
  memberId: string,
): Promise<Response> {
  const claims = await requireAuth(request, env);
  if (!memberId) throw new HttpError(400, "member_id is required");

  const result = await env.DB.prepare("DELETE FROM share_members WHERE user_id = ? AND id = ?")
    .bind(claims.sub, memberId)
    .run();
  if ((result.meta.changes ?? 0) === 0) throw new HttpError(404, "share member not found");

  return json({ removed: true });
}

async function ensureShare(
  env: Env,
  userId: string,
  resourceType: ShareResourceType,
  resourceId: string,
  now: number,
): Promise<string> {
  const existing = await env.DB.prepare(
    "SELECT id FROM shares WHERE user_id = ? AND resource_type = ? AND resource_id = ?",
  )
    .bind(userId, resourceType, resourceId)
    .first<{ id: string }>();
  if (existing) return existing.id;

  const shareId = `share:${crypto.randomUUID()}`;
  try {
    await env.DB.prepare(
      "INSERT INTO shares (user_id, id, resource_type, resource_id, created_at, updated_at) " +
        "VALUES (?, ?, ?, ?, ?, ?)",
    )
      .bind(userId, shareId, resourceType, resourceId, now, now)
      .run();
    return shareId;
  } catch (error) {
    if (!String(error).toLowerCase().includes("unique")) throw error;
    const row = await env.DB.prepare(
      "SELECT id FROM shares WHERE user_id = ? AND resource_type = ? AND resource_id = ?",
    )
      .bind(userId, resourceType, resourceId)
      .first<{ id: string }>();
    if (!row) throw error;
    return row.id;
  }
}

async function requireAuth(request: Request, env: Env): Promise<Claims> {
  const token = request.headers.get("authorization")?.match(/^Bearer\s+(.+)$/i)?.[1];
  if (!token) throw new HttpError(401, "missing bearer token");
  const claims = await verifyJwt(env, token);
  if (!claims) throw new HttpError(401, "invalid token");
  return claims;
}

async function highWater(env: Env, userId: string): Promise<number> {
  const row = await env.DB.prepare(
    "SELECT COALESCE(MAX(server_sequence), 0) AS server_high_water FROM events WHERE user_id = ?",
  )
    .bind(userId)
    .first<{ server_high_water: number }>();
  return row?.server_high_water ?? 0;
}

async function readJson<T>(request: Request): Promise<T> {
  try {
    return (await request.json()) as T;
  } catch {
    throw new HttpError(400, "invalid json");
  }
}

function validateEvent(event: OutboxEvent): void {
  if (!event || typeof event !== "object") throw new HttpError(400, "invalid event");
  for (const key of ["id", "entity_type", "entity_id", "operation"] as const) {
    if (typeof event[key] !== "string" || !event[key]) {
      throw new HttpError(400, `event.${key} is required`);
    }
  }
  if (!Number.isInteger(event.local_sequence)) {
    throw new HttpError(400, "event.local_sequence must be an integer");
  }
}

function integerParam(url: URL, key: string, fallback: number): number {
  const value = Number.parseInt(url.searchParams.get(key) ?? "", 10);
  return Number.isFinite(value) ? value : fallback;
}

function parsePayload(value: string): JsonValue {
  try {
    return JSON.parse(value) as JsonValue;
  } catch {
    return {};
  }
}

function shareMemberPayload(value: JsonValue): ShareMemberPayload | null {
  if (!plainObject(value)) return null;

  return {
    resource_type: normalizeShareResourceType(stringField(value, "resource_type")),
    resource_id: normalizeRequired(stringField(value, "resource_id"), "resource_id"),
    email: normalizeShareEmail(stringField(value, "email")),
    role: normalizeShareRole(stringField(value, "role")),
    status: stringField(value, "status") ?? "invited",
    created_at: numberField(value, "created_at") ?? Date.now(),
    updated_at: numberField(value, "updated_at") ?? Date.now(),
  };
}

function plainObject(value: JsonValue): value is { [key: string]: JsonValue } {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function stringField(value: JsonValue, key: string): string | null {
  if (!plainObject(value)) return null;
  const raw = value[key];
  return typeof raw === "string" && raw.trim() ? raw : null;
}

function numberField(value: JsonValue, key: string): number | null {
  if (!plainObject(value)) return null;
  const raw = value[key];
  return typeof raw === "number" && Number.isFinite(raw) ? raw : null;
}

function distinct<T>(values: T[]): T[] {
  return Array.from(new Set(values));
}

function normalizeEmail(email: string | undefined): string {
  return (email ?? "").trim().toLowerCase();
}

function normalizeShareResourceType(value: string | null | undefined): ShareResourceType {
  const normalized = (value ?? "").trim().toLowerCase();
  if (normalized === "note" || normalized === "notebook") return normalized;
  throw new HttpError(400, "resource_type must be note or notebook");
}

function normalizeShareRole(value: string | null | undefined): ShareRole {
  const normalized = (value ?? "").trim().toLowerCase();
  if (normalized === "owner" || normalized === "editor" || normalized === "viewer") {
    return normalized;
  }
  throw new HttpError(400, "role must be owner, editor, or viewer");
}

function normalizeShareEmail(value: string | null | undefined): string {
  const email = normalizeEmail(value ?? undefined);
  if (email.includes("@") && email.length >= 3) return email;
  throw new HttpError(400, "email is invalid");
}

function normalizeRequired(value: string | null | undefined, key: string): string {
  const normalized = (value ?? "").trim();
  if (normalized) return normalized;
  throw new HttpError(400, `${key} is required`);
}

async function hashPassword(password: string): Promise<string> {
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const key = await crypto.subtle.importKey("raw", encoder.encode(password), "PBKDF2", false, [
    "deriveBits",
  ]);
  const bits = await crypto.subtle.deriveBits(
    { name: "PBKDF2", hash: "SHA-256", salt, iterations: PASSWORD_ITERATIONS },
    key,
    256,
  );
  return [
    "pbkdf2",
    "sha256",
    String(PASSWORD_ITERATIONS),
    bytesToBase64Url(salt),
    bytesToBase64Url(new Uint8Array(bits)),
  ].join("$");
}

async function verifyPassword(password: string, encoded: string): Promise<boolean> {
  const [scheme, hashName, iterations, salt64, expected64] = encoded.split("$");
  if (scheme !== "pbkdf2" || hashName !== "sha256" || !iterations || !salt64 || !expected64) {
    return false;
  }

  const key = await crypto.subtle.importKey("raw", encoder.encode(password), "PBKDF2", false, [
    "deriveBits",
  ]);
  const bits = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      hash: "SHA-256",
      salt: base64UrlToBytes(salt64),
      iterations: Number.parseInt(iterations, 10),
    },
    key,
    256,
  );
  return timingSafeEqual(new Uint8Array(bits), base64UrlToBytes(expected64));
}

async function signJwt(env: Env, claims: Claims): Promise<string> {
  const header = bytesToBase64Url(encoder.encode(JSON.stringify({ alg: "HS256", typ: "JWT" })));
  const payload = bytesToBase64Url(encoder.encode(JSON.stringify(claims)));
  const data = `${header}.${payload}`;
  const key = await hmacKey(env);
  const signatureBuffer = await crypto.subtle.sign("HMAC", key, encoder.encode(data));
  const signature = new Uint8Array(signatureBuffer);
  return `${data}.${bytesToBase64Url(signature)}`;
}

async function verifyJwt(env: Env, token: string): Promise<Claims | null> {
  const parts = token.split(".");
  if (parts.length !== 3) return null;
  const data = `${parts[0]}.${parts[1]}`;
  const valid = await crypto.subtle.verify(
    "HMAC",
    await hmacKey(env),
    base64UrlToBytes(parts[2]),
    encoder.encode(data),
  );
  if (!valid) return null;

  const payload = JSON.parse(new TextDecoder().decode(base64UrlToBytes(parts[1]))) as Claims;
  if (!payload.sub || !payload.device_id || payload.exp < Math.floor(Date.now() / 1000)) {
    return null;
  }
  return payload;
}

async function hmacKey(env: Env): Promise<CryptoKey> {
  const secret = env.KANSO_JWT_SECRET || "dev-only-insecure-secret";
  return crypto.subtle.importKey(
    "raw",
    encoder.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign", "verify"],
  );
}

async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  return Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function base64UrlToBytes(value: string): Uint8Array {
  let base64 = value.replace(/-/g, "+").replace(/_/g, "/");
  base64 += "=".repeat((4 - (base64.length % 4)) % 4);
  const binary = atob(base64);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function timingSafeEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.byteLength !== b.byteLength) return false;
  let diff = 0;
  for (let i = 0; i < a.byteLength; i += 1) diff |= a[i] ^ b[i];
  return diff === 0;
}

function json(body: JsonValue | Record<string, unknown>, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      ...corsHeaders(),
      "content-type": "application/json; charset=utf-8",
    },
  });
}

function corsHeaders(): HeadersInit {
  return {
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "GET, POST, PUT, DELETE, OPTIONS",
    "access-control-allow-headers": "authorization, content-type",
  };
}

class HttpError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
  }
}
