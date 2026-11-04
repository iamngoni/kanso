# In-app feedback: the Worker

Issue #232. Anyone using Kanso can send feedback from inside the app; the sync Worker stores it
privately, has Haiku triage it, and files a short summary as a public GitHub issue. This page
covers what the **Worker** does (build-order step 1). The clients, the build workflow and the
credit trailer are later steps and are not described here.

The rules are those of [CRASH-REPORTS.md](CRASH-REPORTS.md): nothing is sent without the person's
choice, the Worker refuses anything off-pattern, a `400` names the failing field and never echoes
a value. It does not change sync, E2EE or any existing route.

## What is stored, and where

| What | Where | Who can read it |
|---|---|---|
| The validated report (text, kind, optional app info, credit name, screenshot declaration) | D1 `feedback.report`, re-serialized in a fixed key order, never the body that arrived | The owner, with the admin tooling |
| The screenshot | R2 `feedback/<id>/screenshot`, a prefix outside `accounts/`, so no account's quota or reconcile counts it | The owner |
| Haiku's validated answer | D1 `feedback.triage` | The owner |
| The status chain | D1 `feedback_events` | The reporter (My feedback) |
| A summary | A public GitHub issue | Everyone |

The row holds the account ID (signed in) or `r_` and a keyed hash of the anonymous key
(anonymous). It never holds an IP address, a device ID or an email. `wrangler.jsonc` keeps
`invocation_logs: false`, so request bodies are not logged either.

**The issue holds Haiku's summary only, never the raw text or the screenshot.**

## Routes

All are on the Kanso Worker (`sync.kansoapp.app`), even for libraries that sync to a
self-hosted server, and none is `public`, so the Origin rule keeps browsers out.

| Route | |
|---|---|
| `POST /v1/feedback` | A report. `201` stored, `200` a repeat of the same `id` from the same reporter (idempotent), `400`, `401`/`403`, `409` the ID belongs to another reporter, `413` over 48 KiB, `415`, `429`, `503 feedback_closed` |
| `PUT /v1/feedback/<id>/screenshot` | The declared PNG or JPEG (raw bytes, up to 5 MiB). `204`, `400` bytes don't match the declaration, `404`, `409` already stored, `410` the slot (one hour) has closed, `413`, `415` |
| `GET /v1/feedback[?released=X.Y.Z]` | This reporter's reports, newest first, at most 100, each with its status chain. `released` filters to the reports a release fixed |
| `DELETE /v1/feedback/<id>` | `204`. Deletes the row, its events and the screenshot; the issue stays, and the Worker removes the "Reported by" credit from it |
| `POST /v1/github/webhook` | GitHub's events (see below) |

### Who is calling

- **Signed in:** `Authorization: Bearer kanso_<43>`, `X-Kanso-Device: d_<32hex>`,
  `X-Kanso-Device-Proof: <43>`. The Worker checks the session, then SHA-256 of the proof against
  the session's `proof_hash` in constant time, then `SESSION_LIMITER`. A stolen token alone can't
  file reports. Once a bearer token is sent it is the only way in.
- **Anonymous:** `X-Kanso-Feedback-Key: <43 base64url>`, 32 random bytes the install keeps. The
  row stores `r_` + HMAC (pepper `limit` subkey) of it, so the key itself is never stored. The
  `FEEDBACK_LIMITER` binding (5 a minute by IP group) runs first.

### The body

```json
{ "v": 1, "id": "fb_01jbx9k2m4n6p8q0r2s4t6v8w0", "kind": "bug",
  "text": "Export to PDF cuts off wide tables on the right.",
  "credit": { "name": "tendai" },
  "app": { "platform": "macos", "version": "0.9.1", "build": "57", "patch": null, "os": "15.1",
           "device": "Mac14,9", "locale": "en-GB", "screen": "editor", "sync": "kanso",
           "errors": ["update: check failed: timeout"] },
  "screenshot": { "type": "image/png", "bytes": 482113, "sha256": "9f2c…" } }
```

Exactly these keys; `credit`, `app` and `screenshot` may be `null`.

| Field | Rule |
|---|---|
| `id` | `^fb_[0-9a-z]{26}$` (29 characters) |
| `kind` | `bug`, `idea` or `question` |
| `text` | 10–4,000 characters after NFC. Control characters other than `\n` and lone surrogates are refused |
| `credit.name` | `^[\p{L}\p{N} ._-]{1,39}$`, not blank. Plain text, never turned into an `@`-mention |
| `app.platform` | `macos` `windows` `linux` `ios` `android` |
| `app.version` | `^[0-9]{1,4}(\.[0-9]{1,4}){2}$`; `build` `^[0-9]{1,10}$`; `patch` an integer or `null` |
| `app.os` | `^[0-9A-Za-z._-]{1,32}$`; `device` `^[A-Za-z0-9 ,._()-]{1,64}$`; `locale` a BCP 47-shaped tag |
| `app.screen` | `editor` `note-list` `settings` `capture` `ask` `board` `graph` `calendar` `sync` `account` `other` (never a note ID) |
| `app.sync` | `none` `kanso` `self-hosted` |
| `app.errors` | Up to 20 lines of up to 200 characters, each `^(update\|theme\|sync\|ai\|voice\|export\|import): [a-z][a-z ,:_-]{0,180}$` or `crash: <type name>` |
| `screenshot` | `type` `image/png` or `image/jpeg`, `bytes` 1 to 5 MiB, `sha256` 64 lowercase hex. The bytes go up separately and must match all three, and start with the type's magic bytes |

There is no field for a note title, note text, notebook name or file name. The client builds the
payload from an allow-list (`kanso-engine::feedback`, a later step); the Worker is the second wall.

## Limits

| Rule | Window | Maximum |
|---|---|---|
| `feedback-account` (signed in) | day | 10 |
| `feedback-ip` (anonymous, by IP group) | day | 5 |
| `feedback-global` | day | `settings.feedback_per_day` (200); a missing setting refuses |
| Issues filed | UTC day | `settings.feedback_issues_per_day` (50); past it, reports wait as `received` |
| `FEEDBACK_LIMITER` binding (anonymous) | minute | 5 |

A repeat of a stored `id` is not counted. `settings.feedback_open = 0` is the kill switch (`503`);
a missing row also refuses. (The existing `settings` table only takes numbers, so the switch is
`1`/`0`, not `open`/`closed`.) Set them with the admin script's SQL or `wrangler d1 execute`.

## Triage

The cron (every 15 minutes) triages up to 10 `received` reports that are not waiting for a
screenshot (a report waits up to 10 minutes for one). The call goes through **Cloudflare AI
Gateway on unified billing**: the request carries `cf-aig-authorization` and **no `x-api-key`**,
so there is no Anthropic key anywhere. The gateway's rate limit, spend limit and logs apply.

| Header | |
|---|---|
| `cf-aig-authorization: Bearer <AIG_TOKEN>` | unified billing |
| `cf-aig-collect-log-payload: false` | the raw report never sits in gateway logs |
| `cf-aig-skip-cache: true` | every report is unique |
| `cf-aig-metadata` | `{feedback, kind}` |

The model is the `TRIAGE_MODEL` var (`claude-haiku-5-5`), `temperature: 0`, `max_tokens: 1200`,
`tool_choice` forced to the one tool `file_triage`, whose `input_schema` is strict
(`additionalProperties: false`, every field required, enums for category, area, platform,
severity and size). The system prompt (`TRIAGE_SYSTEM`, `src/feedback_triage.ts`) tells the model
the report is untrusted data. The user message is the report with `<`, `>` and `&`
entity-encoded (so nothing can close `</report>`), the app info, and the 50 most recently updated
open issues labelled `from-app`, `bug` or `enhancement` as `#N: title` only. If the person
attached a screenshot it is sent as an image block (up to 3.5 MB; larger ones go without it; the
Worker does not resize).

**The Worker does not trust the answer.** `validateTriage` (pure) re-checks every field:

- the exact keys, enums, lengths and types of the schema;
- no `@` + word character, `http`, `://`, backtick, `<`, `>`, `[`, `](`, `#` + digit, phone or
  email shapes, 20+ base64/hex characters, bare domains, file paths, or invisible/control
  characters in the title, summary, steps, expected or actual;
- no run of 12 words shared with the raw text ("never verbatim");
- `duplicate_of` must be one of the issue numbers that were shown;
- a `noise` verdict is never published, so only its shape is checked.

A failure (gateway error, `429`, spend limit hit, no tool call, or an answer that fails the
checks) leaves the report `received` and counts one attempt. After **5** attempts the report is
filed **bare**: title `Untriaged app report fb_…`, labels `from-app` and `needs-triage`, and only
the report's short ID in the body. A good answer is stored before anything is filed, so a GitHub
failure does not ask the model again.

### Filing

The Worker, not the model, picks the labels and renders the body from a fixed template (the
summary, numbered steps, expected and actual, a table of area, platforms, app and severity, and a
footer with the short report ID and, when the person opted in, `Reported by <name>`).

- Labels: `from-app`, `needs-triage`, `bug`/`enhancement`/`question`, `size:s|m|l`. **`build` is
  never set**: every label passes an allow-list that does not hold it (tested).
- `injection_suspected`: filed, but without steps, expected or actual.
- `contains_personal_data`: a bare issue, as above.
- A duplicate becomes a comment ("Another report from the app (Kanso 0.9.1, macos): …") and a
  thumbs-up on the original; the report's status is `duplicate`.
- Noise files nothing; status `noise`, kept for the owner.

## The GitHub App

"Kanso Feedback", installed on `iamngoni/kanso` only, **Issues: read & write** and **Metadata:
read**, webhook events `issues`, `pull_request` and `release`. The Worker signs a JWT (RS256,
WebCrypto) with the App's private key, trades it for a one-hour installation token (cached per
isolate) and files issues as `kanso-feedback[bot]`. It can't push code or add `build`.

## The webhook and the status table

`POST /v1/github/webhook` checks, in order: `Content-Type: application/json`;
`X-Hub-Signature-256` (HMAC-SHA256 of the raw body under `GITHUB_WEBHOOK_SECRET`, constant time);
the event is `issues`, `pull_request`, `release` or `ping`; `repository.full_name` equals
`FEEDBACK_REPO`; `X-GitHub-Delivery` has not been seen in 24 hours (a redelivery is a no-op; a
delivery that fails is forgotten so GitHub's retry is handled).

| Event | Report moves to |
|---|---|
| `issues.labeled` `build`, sender is `FEEDBACK_OWNER` | `building` (from `filed`) |
| `pull_request.opened`, head `feedback/issue-N` in this repo, body has `Closes`/`Fixes #N` | `pr_open`, `pr = M` |
| `pull_request.closed`, merged | `merged`, `merge_sha` |
| `pull_request.closed`, not merged | back to `filed` |
| `issues.closed`, `state_reason = not_planned` | `closed` (not from `merged`/`released`) |
| `issues.closed`, any other reason (`completed`, none) | `closed` from `received`/`filed`/`building`; a report with an open PR is left to the PR's events |
| `release.published`, tag `vX.Y.Z` | one `compare` call from the previous release's tag; every `merged` report whose `merge_sha` is in it becomes `released` with `release = vX.Y.Z`. With no earlier release, all merged ones. A silent wake-up push goes to each signed-in reporter |

Statuses: `received`, `noise`, `duplicate`, `filed`, `building`, `pr_open`, `merged`, `released`,
`closed`. Every change adds a `feedback_events` row, which is what My feedback lists.

## Retention and deletion

- **180 days** after a report's last status change, whatever the status (an unprocessed report
  counts from receipt), the cron blanks `report` (text, app info, credit), clears `triage`, deletes
  the screenshot and sets `purged_at`. No status keeps raw words or a screenshot forever. The status
  chain stays, so My feedback keeps showing "Released in v0.9.2".
- **Upload slots** close an hour after the report (`screenshot_state = expired`).
- **`DELETE /v1/feedback/<id>`** removes the row, its events and the screenshot, and edits the
  "Reported by" credit out of the issue (the issue itself stays: it holds no raw words). The edit
  is queued in `feedback_credit_removals` first, so if GitHub is down the cron retries it. GitHub
  keeps an issue's edit history, so if a summary ever leaked something, delete the revision or
  the issue, not just the text.
- **Account deletion** (`deletionStatements` in `auth.ts`) deletes the account's reports and
  events, queues their screenshots for the cron, and queues a credit removal for each filed issue
  that carries the reporter's name. The cron edits them out through the App (Issues: write) and
  retries any that GitHub refuses.
- Anonymous reports are deletable with the install's key. If it is lost, the reporter emails the
  owner with the report ID.

## Owner setup (nothing here is done by the code)

1. **Run the migration:** `npm run migrate:remote` (adds `0016_feedback.sql`).
2. **AI Gateway:** create a gateway named `kanso-feedback` on the Cloudflare account. Turn
   **Authenticated gateway** on and create its token. Turn **Unified billing** on and add credits.
   Keep **Require provider credentials** off (on, the call fails with error 2044). Set the
   gateway's **rate limit** to 60 requests / 60 s (sliding), **spend limits** to $5 a day and
   $30 a month for provider `anthropic` (alert at 50%), **caching off**, and logs on with payload
   collection off.
3. **Config:** in `cloud/wrangler.jsonc` replace the `CF_ACCOUNT_ID` placeholder with the account
   ID (32 hex characters). Until then triage does nothing and reports wait. `TRIAGE_MODEL`,
   `AIG_GATEWAY`, `FEEDBACK_REPO` and `FEEDBACK_OWNER` are already set.
4. **GitHub App:** create "Kanso Feedback" (Issues: read & write, Metadata: read; webhook URL
   `https://sync.kansoapp.app/v1/github/webhook`, content type JSON, a webhook secret; events
   `issues`, `pull_request`, `release`). Install it on `iamngoni/kanso` only. Generate a private
   key and convert it: `openssl pkcs8 -topk8 -nocrypt -in app.pem -out app.pkcs8.pem`.
5. **Secrets**, each with `npx wrangler secret put NAME`: `AIG_TOKEN`, `GITHUB_APP_ID`,
   `GITHUB_APP_PRIVATE_KEY` (the PKCS#8 PEM), `GITHUB_WEBHOOK_SECRET`.
6. **Labels:** create `from-app`, `needs-triage`, `size:s`, `size:m`, `size:l` on the repo once
   (and keep `bug`, `enhancement`, `question`). The App can't create labels, and `build` is for
   the owner alone. An issue filed with a missing label simply lacks it.
7. **Deploy** (`npm run deploy`) after 1 to 5. The `FEEDBACK_LIMITER` binding (namespace 1004)
   is created by the deploy.
8. Optional: `UPDATE settings SET value = '0' WHERE name = 'feedback_open'` closes feedback;
   `feedback_per_day` and `feedback_issues_per_day` change the budgets.

## Not in the Worker yet

The `kanso-engine::feedback` payload builder, the client routes and outbox, both UIs, the build
and review workflow, the `Reported-by:` credit trailer in release notes, the website privacy
section, and the `npm run admin feedback list|show` commands that let the owner read a raw report
(until then: `wrangler d1 execute kanso-sync --remote --command "SELECT report FROM feedback
WHERE id = 'fb_…'"` and `wrangler r2 object get kanso-files/feedback/<id>/screenshot`).
