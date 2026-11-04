# In-app feedback: the client

The app side of issue #232. The server is described in `docs/FEEDBACK.md`
(Worker PR). This file defines what the apps build and send.

## What leaves the device

One JSON object, `POST /v1/feedback`, keys in this order and no others:
`v`, `id`, `kind`, `text`, `credit`, `app`, `screenshot`.

- `text` is what the person typed (NFC, 10 to 4,000 characters).
- `credit` is null unless the person turned on "Credit me by name".
- `app` is null when "Include app info" is off, otherwise version, build,
  platform, OS, device, locale, screen, sync mode and up to 20 recent error
  lines.
- `screenshot` declares the type and size of an image that follows as a
  separate `PUT`. The image is only ever one the person chose or captured,
  and is drawn in the sheet before Send. A JPEG loses its EXIF, XMP, IPTC and
  comments (a photo's GPS position and camera) and a PNG its text, EXIF and
  time chunks when it is chosen (`strip_metadata`), so what is declared, shown
  in "Show what's sent" and uploaded are the same bytes.

Never included: note titles or bodies, notebook or tag names, file paths,
account email, any identifier that is not the report id.

Recent error lines pass a whitelist (`scrub_log_line`): a line is sent whole
only if it matches the grammar the Worker enforces
(`update|theme|sync|ai|voice|export|import: <lowercase words>`, or
`crash: <type>`). A line that does not match is dropped, never cleaned.

`build_payload` takes a draft and the app info, never a store, so a note
cannot reach it. Property tests check that note text never appears in the
payload.

## The outbox

A report is written to the on-device outbox before any network call. The id
(`fb_` plus 26 base32 characters) is the idempotency key: a retry sends the
same id and the server answers 200 instead of 201. Retries wait 1, 5 and 30
minutes, then the next launch. A 400 or 429 is shown to the person; the
report is kept unless the server refused its content. A 429 waits for the
Worker's `retry_after` (seconds).

Each row records who sent it (`anonymous` or `account:<id>`). The Worker
only lets that reporter list, upload to or delete a report, and answers 404
to anyone else, which the client reads as "already gone". So a row is only
acted on by its own reporter: the apps flush and refresh with the signed-in
account and, when one was ever made, the anonymous key too. A report sent
before signing in can still be deleted after.

## Auth

Signed in to Kanso's own server: the account bearer token with the device
proof headers. Signed out, or signed in to a self-hosted server: a random
43-character key kept in the secret store, sent as `X-Kanso-Feedback-Key`.
A self-hosted server's token and device proof are never sent to Kanso
(`is_kanso_origin`). Always sent to `https://sync.kansoapp.app`, whatever
sync server the person has set.

## Where it lives

| Piece | Path |
| --- | --- |
| Payload, scrubber, status chips | `crates/kanso-engine/src/feedback.rs` |
| Send, retry and refresh flow | `crates/kanso-engine/src/feedback_flow.rs` |
| Outbox table | `crates/kanso-store/src/feedback_outbox.rs` |
| HTTP client | `crates/kanso-sync/src/feedback.rs` |
| Desktop UI | `crates/kanso-app/src/feedback_sheet.rs`, `my_feedback.rs` |
| Mobile facade | `crates/kanso-mobile/src/feedback.rs` |
| Mobile UI | `mobile/lib/feedback/` |
