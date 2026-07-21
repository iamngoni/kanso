# Kanso Cloudflare Worker

Cloudflare Worker backup target for Kanso. It mirrors the existing Rust cloud API:

- `GET /health`
- `POST /v1/auth/register`
- `POST /v1/auth/login`
- `POST /v1/auth/refresh`
- `POST /v1/sync/push`
- `GET /v1/sync/pull`
- `PUT|GET /v1/blobs/:sha256`
- `GET|POST /v1/shares/members`
- `DELETE /v1/shares/members/:member_id`

Sharing notes:

- Sync still uses one cursor per signed-in account/device. The Worker mirrors shared events into each recipient account's event log with recipient-local server sequences.
- Direct note shares backfill the note's notebook event history, note event history, attachment/sketch event history, attachment blobs, and the share-member event. Future note, attachment, sketch, and blob changes mirror while the member remains on the share.
- Notebook shares backfill the notebook, notes currently in that notebook, their attachment/sketch event histories, attachment blobs, and the share-member event. Future note, attachment, sketch, and blob changes inside the shared notebook mirror while the member remains on the share.
- Editors can push note, notebook, attachment, and sketch changes back into the owner's shared stream; viewers can pull shared resources but their writes are rejected.
- If a recipient registers or logs in after being invited, pending share rows for that email are backfilled idempotently.

Local run:

```sh
npm install
npm run db:migrate:local
npm run dev
```

KansoMac requires backup encryption before it will push or pull through this
target. The Worker stores event payloads as opaque JSON and must not be trusted
with plaintext note bodies or sketch data.

Production should set `KANSO_JWT_SECRET` with `wrangler secret put KANSO_JWT_SECRET`.
