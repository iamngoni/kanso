# Accounts and end-to-end encryption

Status: implementation spec, revision 3 (2026-10-23). Revision 2 (2026-10-18) followed one plan
review and one security review. It froze when step 1 of the build order merged. From then on,
changing a frozen file needs approval from every builder that consumes it (see "Build order").
Revision 3 adds platform passkeys, Linux external security keys, live approval events and
third-party sign-in ([issue #32](https://github.com/iamngoni/kanso/issues/32)); it does not change
the frozen v2 contract, it adds to it. The doc replaces the single shared-token
sync in [PLAN.md](PLAN.md) › Sync. Step 1 applied a security re-check while freezing the contract;
its decisions are listed under "Build order" › "Contract notes".

Kanso's sync moves from one library behind one shared secret to accounts. Anyone with the app can
create an account on a Kanso sync Worker, by default the owner's. Each account's library is
private, and everything in it is encrypted on the Mac before it leaves. The Worker and its owner
hold ciphertext, sizes and timestamps. They can't read notes, notebooks, tags, file names, file
bytes or device names.

Out of scope: email, one-time-code two-factor sign-in and a web client. Sharing between accounts is
specified in "Collaboration".

## Changes since review

Owner's answers, final:

- **Recovery key.** "New recovery key" is a route (`POST /v2/account/recovery-key`), and recovery rotates the key it used.
- **Plan.** The account is on Workers Paid and R2 Paid. Every default and capacity sum is re-sized for a 10 GB D1 database, with the arithmetic shown.
- **Password change** keeps signing out the other Macs.
- **Sessions** expire after 180 days unseen. A sign-out made offline stays pending until the server accepts it.
- **Distribution.** Friends run ad-hoc-signed builds the owner makes. The threat model and the rollout now say what that means, including Gatekeeper.

Plan review (all applied):

1. **Rollback lost edits.** The old app would have inherited v2's `synced_rev` and cursor. Fix:
   - v2 keeps its own meta keys (`sync.v2.*`).
   - `kanso-sync unbind` resets the bookkeeping for v1.
   - The config keeps `endpoint`.
   - The rollback steps say deleted notes may come back.
   - A test proves no edit is lost.

   Merged with security H5.
2. **Re-binding during a sync could mix accounts.** Every re-bind first cancels and joins the running sync. The store refuses sync writes whose account doesn't match the library's binding, checked in the same transaction. Merged with M6.
3. **Replace is one transaction** (`Store::replace_library`). The recovery key goes into the Keychain right after sign-up succeeds, before the library is bound.
4. **Step 1 freezes everything the other builders compile against:** every type, stubs, `MemoryServer` controls, Store APIs, and fixtures with named owners. A Rust milestone, 2a, unblocks the app's flow tests. `crypto-v1.json` moves out of step 1.
5. **Problem model.** It has one label per variant. Problems found in a successful sync are reported separately (`report_problem`). Each problem says whether the schedule keeps running. Unreadable conflicts no longer land in `rejected`.
6. **Wire details.** Status codes for every limit; which fields are required; unknown keys ignored; a mandatory SHA-256 on file uploads; exact username trimming and its fixture cases; the order quota errors are reported in; and codes for an unsupported KDF, a mismatched username and a disabled logout. All key and ID inputs are pinned, and auth conformance cases are added.
7. **No `test-kdf` feature.** The KDF floor comes from the backend, so tests use `MemoryServer`'s low floor and no build can lower the production one.
8. **Capacity re-sized for Workers Paid.** The numbers are under "Quotas and capacity".
9. **WrongKey has one trigger.** It comes from a failed unwrap or a key-id mismatch, never from document failures. An unknown format byte means "Update Kanso". Merged with M3.
10. **Merging is explained.** The question says notes deleted elsewhere may come back. Sign-up into an empty account doesn't push tombstones. The rollout syncs every Mac first and adds a step per other Mac.
11. **Device identity.** A random library secret makes a device ID per account. A hash of the hardware UUID notices a cloned data folder. There is one device-name rule.
12. **The v2 review questions are resolved.** Revision 3's remaining choices are under "Open questions"; only explicit deferrals are under "Deferred".
13. **Tests.** New tests are added, the existing tests that must change are listed, and a decrypted test view replaces raw server access.
14. **Minor points.** Second 413 → stop; storage counts all item kinds; MiB flag names; eviction text; `enabled`; where `Session.server` comes from; how the CLI is run; exact purge checks; the Keychain recovery-key item; the CPU bench.

Security review (all applied):

- **H1. Races.** Limits, caps and quotas are enforced inside D1 (triggers, counting upserts, one-batch device eviction), and a parallel-request test proves it.
- **H2. Storage.** Sign-up starts closed. Global caps, a file-count quota and charging real row and file costs are added. Only base64url can be stored. The admin script refuses to overcommit. Global and IPv6-/48 sign-up limits are added.
- **H3. Recovery key.** It rotates on use and is replaceable while signed in. 401s carry a reason. The clipboard copy is host-only, concealed and cleared after 60 s. Saving to iCloud warns. Sign-out deletes an unconfirmed key.
- **H4. Legacy purge.** It requires a manifest from the old server and a client report proving everything in it is on the Mac and on the encrypted server. Backups of the old tables and files are kept.
- **M1. Account rollback** is detectable through `head_seq` and `epoch` on every page, after which the client re-merges. A document the server lost is pushed again.
- **M2. Revision generations only grow.** Replayed older revisions are refused as stale.
- **M3. Undecryptable documents** are repaired from the Mac's copy.
- **M4. Rekey** is specified as a later route, without a format change. The threat model is corrected: a password change doesn't replace the data key.
- **M5. The Argon2 salt input** binds the server's origin and the username, and so do the recovery derivations.
- **M7. Browser requests are refused:** the wrong content type gets 415, and any `Origin` header gets 403.
- **M8.** Sign-ins from known devices don't count against the username's limit. `recover-user` is gone. A Workers rate-limit binding sits in front of D1. IPv6 is grouped by /56. A custom domain with a WAF rule is needed to survive a flood.
- **L1. Auth hashes** are SHA-256; the pepper only keys fake salts and rate-limit keys.
- **L2. Device IDs** are per account, and device names are sealed.
- **L3. The client** caps responses per route and follows no redirects. KDF ceilings are lower.
- **L4. Session expiry** after 180 days unseen, and pending logouts.
- **L5. Usernames:** a strict ASCII check comes first, with fixture cases.
- **L6. Time Travel retention** is documented. A purge row is kept an hour after its prefix empties.
- **L7.** No test feature to misuse. Device revocation is scoped to the account.
- **L8. Invocation logs off** and no D1 read replicas.

Security review 2 of the Worker (2026-10-18, all applied; the migration was edited in place, as
nothing was deployed yet):

1. **A password change replaces the recovery key** in the same compare-and-swap update, so a
   recovery key planted by someone who once knew the password can't outlive the owner's password
   change. `POST /v2/account/password` gains `recovery_auth_key` and `recovery_wrapped_key`.
2. **Expensive session reads are bounded.** Lengths are read with `octet_length(sealed)`, which
   doesn't read the text. A second write-free binding, `SESSION_LIMITER`, allows about 120
   requests a minute per account on the session routes (429 `session-account`). A push refused as
   malformed or too large counts one write.
3. **Fair shares of the server-wide daily caps.** Past `global_writes_per_day` or
   `global_upload_bytes_per_day`, the triggers refuse only accounts above their fair share (the cap
   ÷ `max_accounts`), so five accounts at their own budgets can't pause a sixth. A day's total
   stays under about twice each cap. The admin script prints the shares.
4. **Only signed-in devices are exempt from `login-user`.** A device signed out for any reason
   counts like a stranger. `DELETE /v2/account/devices/<id>` on a row that is already signed out
   marks it `revoked` and answers 204 instead of 404.
5. **Logs and traces pinned.** `wrangler.jsonc` sets `observability.logs.enabled: true` with
   `invocation_logs: false`, and `observability.traces.enabled: false`, so a changed default
   can't start recording requests. `cloud/DEPLOYED.md` says what `wrangler tail` still shows.
6. **`signups only` right before the sign-up.** The deploy steps, the README and the admin help
   say to run it immediately before the owner's sign-up, since anyone who guesses the name can
   take it while it is reserved, and what to do if they did: `delete`, then `signups only` again.
7. **Accuracy.** The Decisions row on deletion names what Time Travel keeps (ciphertext, the
   username and the password-protected key, for up to 30 days). The migration's comment on
   `recounted_at` names both corrections that set it, and the cron's reconciliation is described
   as it is: it doesn't set `recounted_at`.
8. **Day rollover.** Tests now send requests at different times of one UTC day and on both sides
   of midnight, for the server-wide writes cap and `signups_per_day`. They found one fault: a
   request stamped just before a window's boundary but counted just after it reset that row of
   `limits` to the old window, losing the new day's count. It now counts against the newer window,
   as the `daily` counters do.
9. **IPv6 per /48 everywhere.** `login-ip` and `recover-ip` count IPv6 per /48, as `signup-ip`
   and the binding already did, so one subscriber's 256 /56s are one group.
10. **`admin restore`.** D1 is restored only through `restore <time | bookmark>`, which runs the
    Time Travel restore and then, at once, the epoch rotation of `restored --all`. Without
    `--confirm` it prints the commands; the tests never run wrangler.

Rejected: no finding. Where the result differs from a review's suggestion:

- **The global document cap is 6 GB, not the suggested 5 GB.** 50 accounts × 100 MiB is 5.24 GB,
  so a 5 GB cap would overcommit the defaults. 6 GB still leaves 36% of the database free.
- **No `test-kdf` feature, so L7's `compile_error!` guard has nothing to guard.** The floor comes
  from the backend instead (fix 7).
- **Auth hashes are SHA-256, not the brief's `HMAC(pepper, auth_key)`** (L1).
- **File uploads have no "drop the header" escape hatch.** The Worker hashes every body itself;
  CPU is no concern on Paid.

Two places go beyond the reviews:

- **Client-chosen account IDs and an `account_id` in prelogin answers.** A per-account device ID
  (L2) has to be computed before the first request, so the client must know the account ID first.
- **A daily write budget per account.** On Paid, abuse costs money rather than causing an outage,
  so the budget bounds it.

### Sign-in methods (revision 3)

Owner's decisions, final:

- **Methods.** Platform passkeys on macOS and Windows, external FIDO2 security keys on Linux, Sign
  in with Apple, Google and GitHub. X is dropped. Username and password accounts stay, and can link
  any of these. The provider is a string row, so Microsoft and others need no schema change.
- **Passkeys and security keys sign in and unlock.** Where the WebAuthn PRF extension (or CTAP2
  `hmac-secret` on Linux) is available, the credential produces a key-encryption key that wraps the
  data key, beside the password KEK and the recovery key. Without that output it only signs in; the
  device then unlocks by device approval.
- **New-device unlock.** After Apple, Google or GitHub sign-in, or a passkey or security key without
  a usable PRF, an already signed-in device approves the new one and sends it the data key
  end-to-end encrypted. The recovery key is the fallback.
- **No readable content.** The server still can't read the library. The new keys and the approval
  protocol are designed so that it can't.

The design below adds four tables, one column and a migration; it changes no existing route or
fixture except where it says so, and it keeps the sessions, quotas, rate limits and sign-up gates as
they are.

Later owner decisions (recorded while the Worker was finished, PR A of issue #32):

- **GitHub uses the device flow only.** There is no web + PKCE fallback and no
  `GITHUB_CLIENT_SECRET`; open question 5 is closed.
- **A username-first passkey start returns credential IDs.** `POST /v2/auth/passkey/start` with a
  username lists that account's credential IDs and transports (an unknown name gets an empty list).
  This is accepted: a credential ID is an opaque handle, and sign-up's `username_taken` already
  says whether a name exists.
- **Sign in with Apple works on every platform.** macOS catches the callback's
  `kanso://auth/apple?flow=…&s=…` redirect with ASWebAuthenticationSession. Windows and Linux
  register a `kanso://` scheme handler that hands the URL to the running app. The Worker doesn't
  depend on the platform: the callback page meta-refreshes to that URL and shows a "Return to
  Kanso." link, and the poll needs `s` whatever the platform. On Windows and Linux any local
  program could register the same scheme, so the app registers it at install and claims the flow
  with `s` before anyone else can; the device proof still binds the flow to the device that
  started it.

Where the finished Worker differs from the text below, the code wins and this document follows it:

- **The Apple callback is POST only.** Apple always uses `form_post`, even for a cancel, so there
  is no GET route.
- **The callback redirects to `kanso://auth/apple?flow=<flow_id>&s=<secret>`.** The callback makes
  a fresh 32-byte secret and stores only its SHA-256 (migration `0008_apple_app_secret`:
  `app_secret_hash`, and `app_claimed_at` set by the first poll that sends it). `POST
  /v2/auth/apple/poll` must send `s`, so whoever started a phished `auth_url` can't finish it. A
  ticket or proof from an Apple flow counts only once the app has claimed it.
- **The `smart-card` transport is stored as `smartcard`** (the `transports` column allows only
  `[a-z ]`) and read back as `smart-card`.
- **Approvals take a client `request_id`**, a stable canonical 32-byte ID, so a retried create
  returns the original request instead of making a second one.
- **The pending Keychain item holds the ephemeral approval secret**, so a quit doesn't lose the
  request: on launch the app resumes the same request while it is alive.
- **Linux needs libfido2 1.12 or newer.**
- **An approver may decline only after it has challenged.** Until then, only the requester can
  cancel; another device's `DELETE` is 404.
- **A used, unknown, expired or other session's proof flow is 400 `flow_expired`**, as the error
  table says. No proof, an unknown kind, or a kind the change doesn't take is 401
  `reauth_required`. A provider proof flow whose identity isn't linked to the account answers its
  poll with 401 `invalid_credentials`.
- **`POST /v2/auth/passkey/signup/start` counts `signup-ip`**, as its route text says, not
  `passkey-challenge`. The rate-limit table now says so.
- **Account deletion removes the account's `auth_flows` rows in its own batch**, after the account
  row (a proof's flow guards every earlier statement). The cron's sweep stays for anything left.
- **`POST /v2/auth/recover/finish` sets `has_password = 1`**: the recovered account has a password
  from then on.
- **The AASA route is `GET` and `HEAD`, and 404 when `APP_ASSOC_APPS` is unset or malformed.**
- **A link ends with the session that started it** (security review of PR #67, finding 1). The
  identity insert and the completion's success mark hold only while the account is enabled and a
  session of the flow's device, proving itself with the poll's device proof, is not revoked. A
  session's end also consumes its device's link, proof and passkey-registration flows (migration
  `0010`), so a link started with a stolen password and finished after the owner's password change,
  a revocation, an eviction or an admin `disable` links nothing; its poll answers 400
  `flow_expired`.
- **Event sockets close on every session end.** These close at once: sign-out, revocation, a
  password change or first password, a recovery, eviction by the ten-device limit (every sign-in
  and sign-up route), account deletion, and the 180-day expiry (on the expired session's next
  request, and in the cron's sweep). The admin script's `disable` and `delete` can't reach a
  Durable Object, so they insert a row in `socket_closes` (migration `0009_socket_closes`) and the
  cron closes those sockets within 15 minutes.

## Decisions

| Topic | Decision |
|---|---|
| Sign-up | Open to anyone with the app, once the owner opens it; the migration seeds it closed. A `signup` setting (`open`, `closed`, or `only:<username>`), `max_accounts` and `signups_per_day` are enforced by D1 triggers. No CAPTCHA. Every method — username, platform passkey, Linux security key, Apple, Google, GitHub — creates an account through the same insert, so every gate applies to all of them. |
| Sign-in methods | Username and password; platform passkeys on macOS and Windows; external security keys on Linux; Sign in with Apple, Google and GitHub. A deployment advertises the ones it has configured, and the app shows only those it can actually run. X is not supported. Providers are rows, so another provider needs a module and secrets, not a schema change. |
| Identity | Every account has a username. A social or passkey sign-up suggests one and lets the user change it before creating the account; no account is username-less, so invites, `GET /v2/users/<username>`, recovery and the account row keep working. Provider identities are `(provider, subject)` rows; emails, names and profile data are never stored. |
| Abuse controls | A write-free Workers rate-limit binding in front, then fixed-window counters in D1 (sign-up per IP; sign-ins per account and per IP; recovery per IP; password checks per account; writes per account per day). Per-account quotas on live documents, document bytes, files and file bytes, plus global caps and server-wide daily caps on sign-ups, document writes and upload bytes, all enforced by triggers that fail closed. The daily write and upload caps are shared fairly: an account within its fair share (the cap ÷ `max_accounts`) is never refused by them. |
| Identifier | A username: ASCII, case-folded, 3–32 characters. There is no email anywhere. |
| Keys | Argon2id(password, salt bound to the server's origin and the username) → HKDF splits it into an auth key (sent) and a key-encryption key (never leaves). A random data key encrypts everything. It is wrapped by the password KEK, by a recovery key that rotates on use, and by each passkey's PRF-derived KEK (where the passkey has PRF). A passwordless account stores a random auth hash and an unopenable password wrap, so its row looks like any other. |
| Server secrets | The server keeps SHA-256(auth key) and compares in constant time. It hands out a wrapped data key only after the matching proof. It holds the provider client IDs and secrets needed to complete a sign-in, never a provider access or refresh token. |
| Unlocking a new device | A signed-in device approves it: a six-digit code derived from both devices' ephemeral X25519 keys is shown on both, the approver sends the data key sealed to the requester's ephemeral key, and the server sees only ciphertext. The recovery key is the fallback when no other device is available. |
| Passkeys | The Worker is the relying party (`PASSKEY_RP_ID`). A platform credential's PRF output, HKDF'd with the credential and the account, wraps the data key under a new wrap purpose. A credential without PRF signs in only. Several passkeys per account. |
| Linux security keys | The app is the CTAP2 client, over USB HID and NFC through libfido2; no Worker page and no platform passkey. The CTAP2 `hmac-secret` output is mapped to the same `prf_out` and `passkey_kek`. A key without `hmac-secret` signs in only. |
| Content | Documents, files and device names are sealed with XChaCha20-Poly1305. The associated data binds the account, document, revision and deleted flag. Wire IDs and revisions are keyed hashes, and revision generations only grow. |
| Sessions | One per device; device IDs are per account. Tokens are stored hashed and expire after 180 days unseen. At most 10 active devices; signing in on an 11th signs out the least recently seen. Revoked sessions are kept 90 days to explain 401s and to keep a returning device's name. Only a signed-in device, proving itself with a per-account device proof, is exempt from the per-account sign-in limit. Every sign-in method issues the same kind of session and counts against the same limits. |
| Approval notifications | An unlocked app holds one authenticated WebSocket per account to `/v2/account/events`. Request creation, a sealed name and cancellation publish an event ID; the app fetches the approval row over HTTPS. Notifications are best-effort hints; launch, Settings and every reconnect do one `GET`. There is no approval timer. |
| Changing how you sign in | A session alone can sync, never add, remove or replace a way in. Linking or unlinking a provider, adding or removing a passkey, setting a password, replacing the recovery key and deleting the account each take a fresh proof: the current password, a passkey assertion, or a fresh provider sign-in. A provider sign-in can't add a method, and it needs the recovery key to set a password, replace the recovery key or delete. |
| Device storage | The platform secret store (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux) holds the session token, the data key, any pending logouts, and the recovery key until the user confirms they saved it. A device that has signed in but not yet unlocked holds a separate pending item with the token and no data key. The app never stores the password, master key, KEKs or PRF output. |
| Server choice | The owner's Worker is compiled in as the default. "Use a different server…" points at another deployment. Accounts are bound to their server's origin. |
| Signing out | Secret-store items go (a logout is retried until the server accepts it). Local notes and the library's link to the account stay. |
| Deleting the account | Removes the account's D1 rows, R2 objects and sessions. Local notes stay. Cloudflare D1 Time Travel still holds what the deleted rows held: ciphertext, your username and your password-protected key, for up to 30 days. |
| Protocol | `/v2/`. The new Worker drops `/v1/` and the shared token; the client has one mode. |
| Legacy data | The migration only adds tables. Legacy plaintext is removed only by an explicit admin command, which requires a verified manifest. |
| Server CPU | On Workers Paid (30 s of CPU per request), the Worker still never runs Argon2 and never decodes document or file payloads. It does hash file bodies (about 25 ms for 10 MiB). |

## Starting point

Today the Worker (`cloud/src/index.ts`) serves `/v1/push`, `/v1/changes` and `/v1/files/<id>` for
one library behind `SYNC_TOKEN`. D1 holds plaintext `docs(id, rev, seq, deleted, fields,
updated_at)` and `counter`, and R2 holds bytes under `files/<id>`. The client (`kanso-sync`) runs
`sync_once`: it pulls after the cursor `sync.last_seq` in `meta`, then pushes every doc whose `rev
!= synced_rev` with `base_rev = synced_rev`. Conflicts resolve by `updatedAt`, with the loser kept
in `revisions`. `MemoryRemote` mirrors the Worker, and `cloud/test/fixtures/conformance.json` (renamed
`conformance-v1.json` in step 1) holds cases both pass.

Branch `claude/vigorous-stonebraker-0ea41a` (commit `b4e2733`) added per-user isolation. It is not
merged: its migration drops the plaintext tables. The builders copy parts of it with `git checkout
b4e2733 -- <path>` and adapt them:

| Reused | Replaced |
|---|---|
| Per-user scoping of every statement: `ACCEPT` with the user ID, primary key `(user, id)`, `UNIQUE (user, seq)`, and a per-user `last_seq` counter. | Migration `0002_users.sql`. The new `0002_accounts.sql` only adds tables. |
| Per-user R2 prefixes, without the owner special case at `files/`. | The `users` and `tokens` tables and admin-minted tokens. Sign-up and sign-in create sessions instead. |
| `newToken`, `hashToken`, `isTokenShape` (`kanso_` + 43 base64url characters, stored as SHA-256 hex). | `SYNC_TOKEN` signing in as the owner. `isLabel` (device names are sealed now). |
| The admin runner `scripts/admin.ts` (`wrangler d1 execute`, SQL quoting, `UsageError`, report tables), `scripts/tsconfig.json`, and the "no admin HTTP route" rule. | The admin commands themselves (new list below). |
| Tests: `isolation.test.ts` (model-based plus non-interference); the helpers `client(token)`, `r2Keys`, `admin()`, `execute()`; the injection properties; the `MIGRATION_DB` pattern. | `migration.test.ts`'s expectations. The new migration leaves legacy rows byte-identical. |

## What the server can and can't see

The server sees:

- **Accounts:** usernames, account IDs, creation times, disabled flags, and an epoch (random, changed only by the admin after a restore).
- **Key material:** each account's KDF parameters and salt, SHA-256 hashes of its auth key and
  recovery auth key, its two wrapped data keys (ciphertext), and a 64-bit key ID (an HKDF output
  of the data key that only says whether two parties hold the same key).
- **Documents:** for each one, its kind (`note`, `book`, `tag` or `file`), an opaque ID that lasts
  for the document's life, and an opaque revision token that changes on every edit. Also whether it
  is deleted, its ciphertext length (the plaintext JSON length plus 41 bytes, ×4/3 for base64), the
  server time of every write, and the order of writes.
- **Activity:** counts per kind and usage totals, which documents are written together, and when the account is active.
- **Files:** each one's opaque ID, its ciphertext size (the file size plus 41 bytes) and upload times.
- **Devices:** per-account device IDs, which can't be linked across accounts, and a hash of each
  device's proof. Sign-in and last-seen
  times, the latter to the hour. How many devices there are, and why each was signed out. Not their
  names: those are sealed.
- **Sign-in methods:** which providers are linked to an account, the provider's stable subject ID
  for each, and when it was linked; each passkey's or security key's credential ID, public key,
  COSE algorithm, sign count, transports, whether it carries a PRF wrap, and when it was added and
  last used (to the hour). Not who the provider says the person is: emails, names and profile data
  are discarded. The provider's access and refresh tokens are discarded too.
- **Device approvals:** for each request, the account, the two device IDs, the ephemeral X25519
  public keys, the sealed device name, the sealed data key, and the times it was created, answered
  or expired. Not the six-digit code, which is derived from the X25519 shared secret, and not the
  data key.
- **Account event sockets:** while one is open, the account ID, device ID, connect and disconnect
  times, and the transient event type and approval ID. Not note content, ciphertext, names, keys or
  codes, and no event log. The Durable Object keeps only the account ID and a push alarm.
- **Push tokens:** a device's APNs or FCM token, its platform and environment, when it registered,
  and when it was last woken, on its session row (migration `0011`). The token reaches Apple or
  Google when a push is sent. No push carries a note, a title, a count, a name or an ID.
- **Network:** IP addresses, times and the `kanso-sync/<version>` user agent, in Cloudflare's own
  request handling. Workers invocation logs and traces are pinned off
  (`observability.logs.invocation_logs: false`, `observability.traces.enabled: false`), and the
  Worker logs only route names and error codes. `wrangler tail` (or the dashboard's live logs)
  still shows each request's URL, headers and Cloudflare metadata live to whoever runs it, with
  some values redacted by Cloudflare's heuristics; nothing from it is stored unless they save it. D1 stores IPs and usernames only as keyed
  hashes, inside rate-limit rows kept at most 2 days.
- **Backups:** Cloudflare D1 Time Travel keeps 30 days of history. Deleted accounts' rows and the
  purged legacy plaintext stay restorable by the owner for that window.

The server can't see note titles or bodies; notebook names or hierarchy; tag names or colors; which
notes are in which notebook or carry which tag; statuses or pins; documents' own created and updated
times; links between notes; file names, content types or bytes; device names; local document IDs or
revisions; the password; the recovery key; the data key; any provider email, name or profile data;
any provider access or refresh token; a passkey's private key, PRF output or PRF-derived KEK; or the
device-approval code.

### Published notes

Publishing (the note's ⋯ menu or the palette › Publish to the web…) puts one note's web page at
`<server>/p/<share id>#<key>`. Anyone with the whole link can read the page; the server can't. When
the server has short links on, the link the sheet shows and copies is `<short url>#<key>` instead
(› Short links); the long one keeps working.

**How.** The device renders the note with the web page export (images inside as data URIs, no
scripts), in the preview's theme when it is light and Kanso's light theme otherwise. It picks a
random 128-bit share ID and a random 256-bit key for the share, and seals the page as
`0x01 || nonce(12) || AES-256-GCM(key, nonce, aad, html)` with
`aad = "kanso-share" || 0x01 || share_id` and a fresh random nonce on every upload
(`crates/kanso-sync/src/share.rs`). It uploads that to `PUT /v2/shares/<id>` with its session, and
keeps `{shareId, key, publishedAt, publishedRev, expiresAt?, contentHash, shortUrl?}` in the note as
`note.extra.kansoPublish`. That travels inside the sealed note, so the account's other devices can
update the same link (same ID and key, new ciphertext) or unpublish it, and the server never
receives the key. The note's own `share` field, part of the API's note shape, is left alone. Only the Publish sheet writes
`kansoPublish`: note writes through the local API and MCP keep the note's existing value and
ignore any they are given. The key lives only in the link's `#fragment`, which browsers never send
to a server.

**Routes.** `PUT /v2/shares/<id>` (session; `application/octet-stream`; `x-kanso-sha256`; optional
`x-kanso-expires-at`, Unix ms, at most 366 days ahead; `x-kanso-title`, the note's title as
percent-encoded UTF-8, at most 200 characters) stores or replaces the account's own share and
answers `{ok, id, size, expires_at, short_url}`, `short_url` null when there is no short link.
Another account's ID is 409 `share_id_taken`. An ID that has ever stopped being served
(unpublished, expired, taken down, or of a deleted account) is retired for good and answers 409
`share_retired`: whoever holds an old link knows its ID and key, so nobody, its owner included, may
publish under it again. On either 409, and for a link already past its expiry, the device publishes
under a new ID and key. A body that isn't `0x01` plus at least a nonce and tag is refused, so only
sealed bytes are stored, up to 10 MiB of page. A share counts as one file (its size + 1,024 bytes)
towards the account's file quotas, and its upload towards the daily writes and upload bytes.
`DELETE /v2/shares/<id>` (session) unpublishes; 404 when the account has no such share.
`GET /v2/public/<id>` (no session) returns the ciphertext; 410 `expired` past its expiry; 404 once
unpublished, while its account is disabled, or while R2's object doesn't have the SHA-256 the row
records (two racing uploads of one share). It is rate-limited per IP group by the `PUBLIC_LIMITER`
binding only (60 a minute), so a public read never writes to D1, and a malformed ID never reaches
D1. `GET /p/<id>` is the viewer page. D1 holds `shares` (owner, size, SHA-256, times, expiry),
`retired_shares` (every retired ID, filled by a trigger whenever a share's row is deleted) and
`share_purges`; R2 holds `shares/<id>` (migration `0003_shares`; `0004_short_links` adds the
short-link columns and `short_link_purges`). The cron unpublishes expired
shares and deletes purged objects, and its reconciliation takes a slice of `shares/` each run:
objects over an hour old without a row are deleted, and a row left describing another upload than
the one R2 holds is moved to that object (its size counted against the quotas, or the share is
unpublished when they refuse it). Deleting an account (by its owner or `admin delete`) unpublishes
all of its shares in the same batch. For abuse, `admin shares <username>` lists an account's
shares and `admin unpublish <share id>` takes one down for good; `admin disable` stops serving all
of an account's shares until `admin enable`.

**Short links.** With the `REROUT_API_KEY` secret set (and `REROUT_PROJECT_ID`, the Kanso project
on Rerout, in `wrangler.jsonc`), the Worker gives each share a Rerout short link after storing its
upload (`cloud/src/rerout.ts`): `POST https://api.rerout.co/v1/links` with `target_url`
`<origin>/p/<id>` (no fragment, so no key), `seo_title` the note's title (whitespace collapsed,
controls removed, cut to 90 characters; "A note" without one), one line about Kanso as
`seo_description`, Kanso's preview image `<origin>/og/kanso.png` (a static asset,
`assets/icon/make-og.swift` draws it) as `seo_image_url`, the target as `seo_canonical_url`, and
`seo_noindex`. With `REROUT_DOMAIN` set (`go.kansoapp.app`, added and verified in the Rerout
project, its CNAME in the kansoapp.app zone) it also sends `domain_hostname`, and a link Rerout answers
on any other host is deleted and not used, so the device shows the long link rather than one on
another domain. The share's row keeps the link's code, URL and title; a later upload with another
title retitles it (`PATCH`), and two racing uploads keep one link and delete the other. rerout.co
answers a browser with a plain 302 to the target, and browsers carry the fragment of the link they
opened across a redirect, so `<short url>#<key>` opens the note and Rerout never receives the key.
Link-preview bots get Rerout's page with the title, the line and the image. Short links are best
effort: without the secret, on an origin that isn't https (`wrangler dev`), or when Rerout fails or
takes over 5 seconds, publishing succeeds with `short_url` null and the device shows the long link;
the next upload tries again. Whatever deletes a share's row (unpublish, expiry, `admin unpublish`,
account deletion, the reconcile) queues its link in `short_link_purges` by trigger; unpublishing
deletes it from Rerout at once, and the cron deletes the rest (25 a run, retrying failures behind
the others). A link Rerout made for another project, to another target, or with a short URL a key
can't follow (a query, a fragment, not https) is deleted and not used.

**The viewer page** is static HTML with one inline script and one inline style, each pinned in a
`Content-Security-Policy` by its SHA-256 (`default-src 'none'`, `connect-src 'self'`, `img-src
data:`, `frame-src 'none'`, `frame-ancestors 'none'`, no third-party code), sent with
`Referrer-Policy: no-referrer` and `X-Robots-Tag: noindex`. Its preview tags are the same for every
share ("A note", the line about Kanso, the preview image), so it never reads D1. Its script reads the key from
`location.hash`, fetches the ciphertext, opens it with WebCrypto, and shows the page in an
`<iframe srcdoc>` sandboxed without `allow-scripts` and without `allow-same-origin`, after taking
out anything active (scripts, templates and so declarative shadow roots, frames, objects, forms,
event attributes, links other than http, https and mailto) and gathering the page's styles into
one `data:` stylesheet the policy allows. A missing key, a wrong key, an expired link and an
unpublished one each get a calm sentence instead of the note.

The viewer page comes from the same Worker as the ciphertext, so the reader trusts that Worker
to serve this script. A compromised Worker, or anyone able to deploy to it, could serve a viewer
that sends the key in the `#fragment` somewhere: end-to-end encryption protects a published note
from the stored data and its backups, not from whoever controls the code the reader's browser
runs.

**The server sees,** for each share: its ID, the account that owns it, the ciphertext's size (the
page's size plus 29 bytes) and its SHA-256, when it was published and updated, when it expires,
the note's title as it was published (sent so previews can show it, and kept on the share's row
while short links are on), and, like any request, the IP addresses and times of the people who open
it (live in `wrangler tail`, and counted by the rate-limit binding, which keeps nothing in D1).
With short links on, **Rerout sees** the share's ID, the note's title, and the clicks on its short
link, as it does for any link. **Neither receives** the key, so neither can see the page, the
note's text or the images, and neither can tell which note a share came from beyond its title (but
see the viewer page's caveat above). Whoever has the link can read the note, and
can pass the link on: the key is the link. Unpublishing stops the server serving it (and deletes
the ciphertext), but can't take back a copy someone already saved.

### Published notebooks

Publishing a notebook (its sidebar menu › Publish Notebook…, Note › Publish Notebook…, or the
palette › Publish notebook…, which takes the notebook the list shows, else the open note's) puts
its notes, and those of the notebooks inside it, at one link as a small site: an index grouped by
sub-notebook, a page for each note, and working links between them. The sheet lists what goes
before anything is sent: the notes, the locked notes left out, and how many images and other
files they use.

**One share.** A notebook is published exactly as a note is (› Published notes): the same route,
the same seal, AAD, 10 MiB limit, quotas, expiry, retirement, short links and unpublishing. Only
the plaintext differs. Instead of a page it is one JSON bundle (`crates/kanso-sync/src/site.rs`):

    {"kanso": "site", "v": 1, "title": <notebook name>, "index": <html>,
     "pages": {<slug>: {"title", "html", "downloads": [<file index>]}},
     "files": [{"name", "type", "data": <base64>}]}

Each page is the note's web page export (`kanso_render::html::page_in`), no scripts, in the same
theme as a note's. A link to another published note, by ID or as a wiki-link, becomes
`kanso-page:<slug>`; a link to a note that isn't published (in another notebook, or locked) is
plain text, so the site never links out to a note it doesn't hold.
Images are put in `files` once each and shown as `kanso-file:<n>`; other attachments a note links
to are offered to download on its page. Slugs come from titles (`a-z0-9-`, at most 64, unique). The
device checks the bundle (every link and file resolves) before sealing it.

**Locked notes are never published.** The plan reads every note again and leaves out any that is
locked, with its attachments; their titles are listed in the sheet. A note locked after the
notebook was published stays in the published copy until the notebook is updated or unpublished:
the sheet then says the notebook has changed.

**What the notebook keeps.** `book.extra.kansoPublish`, the same shape as a note's, travels inside
the sealed `book:` document, so the account's other devices see the link and can update or
unpublish it. Its `contentHash` is of the notebook's name and a fingerprint of every published
note's ID, title and text, so the sheet can tell when the notebook has changed; `publishedRev` is
empty. As for notes, only the sheet writes it: the local API and MCP keep a notebook's value and
ignore any they are given. The title sent with the upload (`x-kanso-title`) is the notebook's
name.

**The viewer page** opens a bundle as well as a page: plaintext that parses as a site shows the
index at `#<key>` and a page at `#<key>/<slug>`, under a bar with the notebook's name (a link back
to the index) and the page's downloads. Links between pages become links to the viewer itself at
the other page's fragment, opened in the top window: the frame is sandboxed with
`allow-top-navigation-by-user-activation` as well, so a click on a link can move the reader to
another page and nothing else can, and as only the fragment changes the page never reloads and the
key never leaves the fragment. The frame still has no scripts and no origin of its own. Images are
put back as `data:` URLs (only `image/*` types), and downloads are made in the viewer from the
bundle's bytes. The server sees nothing more than for a note: one share, its size, and the
notebook's name.

## Constructions

### Encodings

- Binary values in JSON and headers are unpadded base64url (RFC 4648 §5), except hashes, which are lowercase hex.
- The Worker decodes base64url only for 32-byte proofs (auth keys, recovery auth keys, device
  proofs). It rejects
  input of the wrong length, or non-canonical input (where re-encoding doesn't reproduce it), with
  400 `bad_request`.
- Fixed lengths: 16 bytes → 22 characters, 32 → 43, 73 → 98.
- Integers are JSON numbers. Times are Unix milliseconds.
- `lp(x)` means a 2-byte big-endian byte length of `x`, then the UTF-8 bytes of `x`.
- Format version 1 is the byte `0x01`.
- **Origins.** `origin` is `url::Url::parse(server)?.origin().ascii_serialization()` of the URL the
  client actually connects to, for example `https://kanso.antonlabs.cc`: lowercase,
  punycode, default port omitted, path dropped. The server never supplies it. The owner's server
  answers to later names too (`https://sync.kansoapp.app`); the client maps those to its first
  origin, `OWNER_ORIGIN`, before deriving, so accounts made under the first name open under any.

### Password and KDF

- **Password.** Normalized to Unicode NFC (`unicode-normalization`), then used as UTF-8. Kanso
  requires 12–256 characters (Unicode scalar values) and refuses a password equal to the username,
  ignoring case. Only the client checks this.
- **Salt input.** `salt_input = "kanso/v1/salt" || lp(origin) || lp(username) || salt16`. Here
  `salt16` is 16 random bytes per account from the server's prelogin answer, and `username` is
  normalized. A second server that echoes the first server's salt still gets an auth key bound to
  its own origin, which is useless on the first server. Every password change and recovery replaces
  `salt16`.
- **Master key.** `master = Argon2id(password, salt_input, m = 1,048,576 KiB, t = 3, p = 4, version
  0x13, 32 bytes)`.
- **Timing.** 0.36 s warm and 0.45–0.56 s cold on the owner's M4 Pro, with `argon2 = 0.6` and its
  `parallel` feature (rayon, already in `Cargo.lock`). Without `parallel` it takes about 3.6 times
  as long. Expect about 1–1.5 s on an 8 GB M1.
- **Floor and ceilings.** Each account stores its parameters. The production floor is `m_kib ≥
  1,048,576`, `t ≥ 3`, `1 ≤ p`. The ceilings are `m_kib ≤ 2,097,152`, `t ≤ 8`, `p ≤ 8`, which caps
  a hostile server's demand at about 2 s and 2 GiB on the M4 Pro.
- **Where they come from.** The client takes both from its backend: `Backend::kdf_floor()` and
  `kdf_default()`. `HttpBackend` returns the production constants. `MemoryServer`, the test server,
  returns `m = 64 KiB, t = 1, p = 1`. There is no cargo feature that lowers the floor, so no build
  of the app can. The Worker enforces the production floor and ceilings regardless.

### Keys

All derived keys are 32 bytes unless stated. "HKDF" means HKDF-SHA256; without a stated salt it
uses the default all-zero salt (RFC 5869).

| Key | Derivation | Use |
|---|---|---|
| `master` | Argon2id as above | Input only; zeroized right after HKDF |
| `auth_key` | HKDF(`master`, info `"kanso/v1/auth"`) | Sent at sign-up, sign-in, password change, new recovery key and deletion |
| `kek` | HKDF(`master`, info `"kanso/v1/kek"`) | Wraps the data key; never leaves the Mac |
| `recovery_secret` (16) | Random | Shown as the recovery key; replaced on every use and every password change |
| `recovery_auth_key` | HKDF(ikm `recovery_secret`, salt `"kanso/v1/recovery" \|\| lp(origin) \|\| lp(username)`, info `"kanso/v1/recovery-auth"`) | Sent at sign-up, recovery, new recovery key and password change |
| `recovery_kek` | HKDF(same ikm and salt, info `"kanso/v1/recovery-kek"`) | Wraps the data key |
| `data_key` | Random at sign-up (rekey, deferred, would replace it) | Root of the content keys; kept in the Keychain |
| `doc_key` | HKDF(`data_key`, info `"kanso/v1/doc"`) | Seals documents |
| `file_key` | HKDF(`data_key`, info `"kanso/v1/file"`) | Seals file bytes |
| `id_key` | HKDF(`data_key`, info `"kanso/v1/wire-id"`) | HMAC key for wire IDs |
| `rev_key` | HKDF(`data_key`, info `"kanso/v1/wire-rev"`) | HMAC key for wire revisions |
| `device_name_key` | HKDF(`data_key`, info `"kanso/v1/device-name"`) | Seals device names |
| `key_id` (8) | HKDF(`data_key`, info `"kanso/v1/key-id"`, 8 bytes), as 16 hex digits | Told to the server; the client compares it on sign-in and on every changes page |

Every secret type wraps `Zeroizing` and has a redacted `Debug`.

### Wrapped data key

`wrapped = 0x01 || nonce(24) || XChaCha20-Poly1305(key, nonce, aad, data_key)`. That is 73 bytes,
or 98 base64url characters.

- **AAD.** `"kanso-wrap" || 0x01 || purpose || lp(account_id)`. The purpose byte is `0x50` (`P`) for
  the password wrap, `0x52` (`R`) for the recovery wrap and `0x4B` (`K`) for a passkey wrap. Each
  wrap opens only under its own KEK and for its own account, so a server can't pass one account's
  wrap off as another's (security re-check 3).
- **Rewrapping.** A password change rewraps the same data key, so notes aren't re-encrypted.
  Recovery unwraps with `recovery_kek`, then wraps under the new password's KEK and under a new
  recovery key. Passkey wraps are untouched by either, because the data key is the same; recovery
  removes them entirely (see "Rotations"). Adding or removing a passkey changes only that
  credential's wrap.
- **Checks after unwrapping.** A failed unwrap is `KeyMismatch`. The client also compares
  `key_id(data_key)` with the server's `key_id`; a difference is `KeyMismatch` too.

### Recovery key

The recovery key is 16 random bytes (128 bits), shown as 28 Crockford base32 characters in 7 groups
of 4, for example `0K7M-QX9P-3J2H-ZD8W-5TNB-1RGV-E6CA` (layout only; its checksum isn't real).

- **Encoding.** Read the 16 bytes as a big-endian integer `V`. The first 26 characters are `V` in
  base32, most significant digit first. 130 bits hold 128, so the first character is `0`–`7`.
- **Checksum.** The last 2 characters are `V mod 1021` (1021 is prime and below 32²), as two base32
  digits, most significant first. Every single-character substitution, and every adjacent
  transposition within the 26 data characters, changes `V mod 1021`, so the checksum always catches
  them. A script check over 20,000 random keys found no exception. Other typos get through with a
  probability of about 1 in 1021, and are then refused by the server and the unwrap.
- **Alphabet.** `0123456789ABCDEFGHJKMNPQRSTVWXYZ`.
- **Parsing.** Drop spaces and hyphens and uppercase. Read `O` as `0` and `I`/`L` as `1`. Reject
  `U` and every other character. Then require exactly 28 characters, a first character below `8`, a
  check value below 1021, and a matching checksum.
- **Rotation.** Recovery replaces the key: the used key stops working and the new one is shown. "New
  recovery key…" replaces it while signed in, and so does every password change. Recovery also
  removes every passkey (see "Sign-in methods › Rotations").

### Sealed documents

A document travels as `sealed = base64url(0x01 || nonce(24) || XChaCha20-Poly1305(doc_key, nonce,
aad, plaintext))`. The nonce is 24 random bytes from `getrandom`, new for every seal.

- **Plaintext.** UTF-8 JSON `{"id": <local id>, "rev": <local rev>, "fields": <object>}`.
  Tombstones have `"fields": {}`. Unknown top-level keys are ignored, for forward compatibility.
- **AAD.** `"kanso-doc" || 0x01 || lp(account_id) || lp(wire_id) || lp(wire_rev) || deleted`, where
  `deleted` is the byte `0x00` or `0x01`.
- **Opening** gives one of three results:
  1. *Too new*: the first byte is greater than `0x01`. Pulling stops at this document and the Mac
     says "Update Kanso". It never overwrites such a document.
  2. *Undecryptable*: the first byte is `0x01` but one of these fails: the AEAD; the JSON shape;
     `wire_id(id)` and `wire_rev(id, rev)` equalling the wire values; the local ID's kind equalling
     the wire kind; a tombstone having empty fields.
  3. *Opened.*
- **Size.** Sealed strings are 55 to 1,900,000 characters, so a D1 row stays under 2 MB. The client
  refuses to push a document whose `fields` JSON is over 1,400,000 bytes; it lands in the report's
  `too_large`. The largest real note is 253 KB.

### Opaque IDs and revisions

Local revisions are `{generation}-{MD5 of canonical JSON}`, and API callers may choose local IDs.
Neither goes on the wire:

- `wire_id(id) = kind + ":" + base64url(HMAC-SHA256(id_key, id)[0..16])`. The HMAC input is the
  full local ID including its `kind:` prefix, for example `note:Q2hhbmdlTWVQbGVhc2UxMg` from
  `note:07HmFZwo`.
- `wire_rev(id, rev) = hex(HMAC-SHA256(rev_key, lp(id) || rev)[0..16])`, 32 lowercase hex digits.
- `base_rev` on the wire is `wire_rev(id, synced_rev)`.

### Revision generations only grow (M2)

This is the invariant: every revision the server accepts for a document has a higher generation
than the one it replaces. Local writes already use `Rev::next(current)`. The PouchDB library importer only
writes when the incoming generation is higher (`Store::import_doc`).

Two store rules keep it, in `apply_remote`:

- **When the local version wins a conflict,** it gets a new revision: generation `max(local,
  remote) + 1`, with the same content hash. That revision is pushed with `base_rev =` the remote
  revision.
- **A remote revision is refused as `Stale`** when its generation is at or below the document's
  replay floor, `synced_gen`, and it isn't the revision the Mac holds or last synced. `synced_gen`
  is the highest generation this Mac has seen on this account for the document. A server-reported
  loss and an epoch change never lower it (see "Sync with encryption › What a Mac remembers per
  document"). A replayed older version or tombstone therefore can't revert a Mac that has seen
  something newer, including a note re-created under the same ID. That holds even after the server
  claims it lost the document or rotates its epoch. The one exception is a live version this Mac
  never held, arriving over a local deletion: an edit always beats a deletion, so it wins (rule 4
  in "Sync with encryption › In order" has the trade-off).

The report lists stale documents; the cursor moves past them.

### Sealed files

A file travels as `sealed = 0x01 || nonce(24) || XChaCha20-Poly1305(file_key, nonce, aad, bytes)`,
sent raw.

- **AAD.** `"kanso-file" || 0x01 || lp(account_id) || lp(wire_id)`.
- **Size.** The plaintext limit is 10 MiB (`MAX_ATTACHMENT_SIZE`), and the container allowance is 4,096 bytes, so a sealed file is 41 to 10,489,897 bytes.
- **Content type.** It stays inside the sealed file document. R2 objects are always `application/octet-stream`.
- **Matching bytes to a version.** The opened bytes must match the file document's `md5digest`. A
  mismatch counts as missing, and a later ordinary sync fetches them again ("Sync with encryption ›
  Files").

### Sealed device names

The name is the device's own: the ComputerName on macOS (`scutil --get ComputerName`), `COMPUTERNAME` on
Windows and the host name on Linux, with control characters removed, trimmed, and cut to 64 Unicode
scalar values. When that leaves nothing it is "Mac", "Windows PC" or "Linux computer".

`sealed_name = base64url(0x01 || nonce(24) || XChaCha20-Poly1305(device_name_key, nonce,
"kanso-device" || 0x01 || lp(account_id) || lp(device_id), name))`. That is 55–400 characters.

The server stores and returns it opaquely. A device that has no name yet (just signed in) shows as
"A Mac" until it sets one.

### Device identity (per library)

- **Creating it.** The first account action on a library creates, in its `meta`:
  - `sync.v2.library_id`: `l_` + 32 lowercase hex characters from `getrandom`; it names the Keychain items.
  - `sync.v2.device_secret`: 32 random bytes as hex.
  - `sync.v2.hardware`: `hex(SHA-256("kanso-hw" || IOPlatformUUID))[0..32]`. The UUID comes from
    `ioreg -rd1 -c IOPlatformExpertDevice`. If `ioreg` gives no UUID, the value is empty and the check below
    is skipped.
- **The device ID for an account** is `d_ + hex(HMAC-SHA256(device_secret, account_id)[0..16])`, so the server can't link one Mac's sessions across accounts.
- **The device proof for an account** is `HKDF(ikm device_secret, info "kanso/v1/device-proof" ||
  lp(origin) || lp(account_id))`, 32 bytes, sent as 43 base64url characters as `proof` in every
  `device` object (sign-up, sign-in, recovery). The server keeps only its SHA-256 in the session row
  (`proof_hash`). A sign-in skips `login-user` only when the proof matches the active session row of
  that device, so knowing a device ID (every Mac of the account sees them) isn't enough (security
  re-check 5). A signed-out row, whatever signed it out, exempts nothing: an old copy of the library
  holds the device secret, and the device list doesn't show the row (security review 2, finding 4).
- **A copied library.** When the stored `sync.v2.hardware` differs from this Mac's, the library was
  copied (Migration Assistant, restoring a backup to new hardware). The library ID and device secret
  are regenerated, which leaves the copy signed out: its Keychain item names change. Its binding and
  cursor stay, so signing in resumes as a new device instead of fighting the original Mac over one
  session.
- **Sign-out and unbind keep all three values.**

### Server-side hashes and the pepper

- `auth_hash = hex(SHA-256(auth_key))` and `recovery_hash = hex(SHA-256(recovery_auth_key))`. The
  inputs are 256-bit derived keys, so a plain hash can't be inverted. A pepper would add nothing:
  the wrapped data key in the same row allows the same offline password guessing, and the pepper's
  only effect would be locking everyone out if it were lost. The Worker compares the 32-byte
  digests with `crypto.subtle.timingSafeEqual`. Unknown usernames still cost one hash.
- **Session tokens** are 256-bit random values, stored as SHA-256 hex. **Device proofs** are stored
  the same way, as `proof_hash`.
- **`AUTH_PEPPER`** is a 32-byte base64url secret, used only through HKDF subkeys (WebCrypto `deriveBits`):
  - **Fake prelogin answers.** `fake_key = HKDF(pepper, info "kanso/v1/fake")`. The fake salt is
    `HMAC(fake_key, "salt" || 0x00 || username)[0..16]`. The fake account ID is `a_` +
    `hex(HMAC(fake_key, "account" || 0x00 || username)[0..16])`.
  - **Rate-limit keys.** `limit_key = HKDF(pepper, info "kanso/v1/limit")`. For IP subjects the key
    is `rule + ":" + hex(HMAC(limit_key, rule || 0x00 || subject)[0..16])`. For account subjects it
    is `rule + ":" + account_id`, which isn't secret, so the admin can read it. `login-user` uses the
    account ID of an existing username, and one shared key, `login-user:unknown`, for every name
    without an account.
- **A missing or malformed pepper** answers 503 `misconfigured` on the routes that use it (health,
  prelogin, sign-up, login, recovery, and the sign-in method flows that seal a device code or a
  result). Session routes keep working.

### Rust crates

Workspace dependencies, all on the `digest 0.11` generation already in `Cargo.lock`:

- `argon2 = { version = "0.6", default-features = false, features = ["alloc", "parallel", "zeroize"] }`
- `chacha20poly1305 = { version = "0.11", default-features = false, features = ["alloc", "zeroize"] }`
- `hkdf = "0.13"`, `hmac = "0.13"`, `sha2 = "0.11"`, `zeroize = "1.9"`, `getrandom = "0.4"`, `unicode-normalization = "0.1"`

The existing `base64` (0.22, `URL_SAFE_NO_PAD`) and `url` are reused. There are no custom
primitives.

## Sign-in methods

A sync account can be reached with a username and password, a platform passkey, an external security
key on Linux, Sign in with Apple, Google or GitHub. All of them create the same kind of session. The
difference is only how the person proves who they are, and how the new device gets the data key: a
password derives it, a passkey with PRF or a security key with `hmac-secret` derives it, and
everything else asks an already signed-in device or the recovery key.

The server still never holds a key that opens the library. An OAuth token proves identity to the
Worker; it is not and can never be an encryption key, because the provider and the Worker can both
obtain one. So the data key keeps its own wraps, and the new paths below add wraps or hand the key
over end-to-end encrypted.

A deployment advertises which methods it has. The app shows only those, and a passkey button only
where the app can run the ceremony itself: the native platform sheet on macOS and Windows, or a
direct CTAP2 ceremony on Linux. Health cannot see a USB key, so the Linux button also requires the
app to have enumerated a compatible authenticator. It always shows the username form. Old clients
ignore the advertisement and the new routes.

A **platform passkey** is a credential the operating system or a credential manager creates and
syncs. A **security key** is an external FIDO2 authenticator. They share the server's passkey
routes, public-key checks and wrap construction. They differ in who runs the ceremony: the platform
on macOS and Windows, or the app's libfido2 client on Linux. "Passkey" below means either
credential unless the text names the platform.

### Account identity

Every account keeps its `accounts` row and its username. A social or passkey sign-up suggests a
username and lets the user change it before the account exists (see "Usernames"). There is no
username-less account: invites (`GET /v2/users/<username>`), `kanso-identity-bind`, recovery,
`AccountRef` and the frozen `accounts.username NOT NULL UNIQUE` all keep working unchanged. This is
a deliberate choice over a nullable username, which would touch every one of those and need a table
rebuild.

A provider identity is one row:

    account_identities (account_id, provider, subject, created_at)
    PRIMARY KEY (provider, subject)

- `provider` is a lowercase ASCII name: `^[a-z][a-z0-9_]{1,31}$`. `apple`, `google` and `github` are
  the ones this revision defines; `microsoft` or another needs a module and its secrets, not a
  migration.
- `subject` is the provider's stable user identifier, 1–255 printable ASCII characters: Apple's
  `sub`, Google's `sub`, GitHub's numeric `id` in decimal. A provider's email, name, avatar, login
  name or profile is never read into a column, a log line or an error message. GitHub's `login` can
  change, so it isn't used.
- `UNIQUE (provider, subject)` means one identity belongs to one account. Two accounts can't claim
  the same GitHub user, even in a race: one insert loses with 409 `identity_taken`.
- An index on `account_id` lists an account's linked providers.

Signing in with an identity that isn't linked does **not** look for an account by email. The app
offers to create a new account instead, with the username step. An identity is only ever bound by an
explicit sign-up or link.

**Linking and unlinking.** A signed-in account can link any configured provider, and add any
number of passkeys, with a password or passkey proof (a provider-only account sets a password
first). Unlinking is refused with 409 `last_method` when it would leave the account with no
password, no provider identity and no passkey, unless the request also proves the recovery key:
`"recovery_auth_key"` in the body, checked against `recovery_hash`. Proving it is what "unless a
recovery path exists" means in practice. The app warns first: "This is your last way to sign in. If
you remove it, you'll need your recovery key." A recovery-only account is allowed; `recover/finish`
sets a password, so it isn't stuck. Only ways in this server offers count (security review of PR #67,
finding 5): an identity of a provider the deployment no longer serves or configures, or a passkey on
a deployment without `PASSKEY_RP_ID`, can't sign in, so it never keeps another method from being
the last. The rule is checked inside the deleting statement, with the offered providers bound to it.

**Changing a way in needs a fresh proof.** A session can sync; it can't change how the account is
entered. Linking or unlinking a provider, adding or removing a passkey, setting a first password,
replacing the recovery key and deleting the account each carry a `proof`:

```json
"proof": {"kind": "password", "auth_key": "<43>"}
       | {"kind": "passkey", "flow": "<43>",
          "assertion": {"credential_id", "authenticator_data", "client_data_json", "signature"}}
       | {"kind": "provider", "flow": "<43>"}
```

- A password proof is the current `auth_key`, derived as today.
- A passkey proof is an assertion over a challenge from `POST /v2/account/sign-in/proof/start`
  (5 minutes, single use, `userVerification: "required"`), checked against one of the account's
  credentials. It proves presence, not a new sign-in, so it doesn't create a session. A platform
  without a native passkey ceremony doesn't offer passkeys at all (see "Where the ceremony
  runs").
- A provider proof is a flow started by `POST /v2/account/sign-in/identities/<provider>/start` with
  `"intent": "proof"` (the session is enough; the provider sign-in is the proof) and completed like
  a link; the Worker checks that its subject is already linked to this account. A link's start
  instead carries a password or passkey `proof`, so a session alone can't add an identity.
- The app offers whichever the account has, preferring a passkey, then the password, then a
  provider. A locked device (no data key) can't make a proof for another device's action: the
  action is started by the device that holds the session, and the proof is made on that device. An
  approval request is not a proof.
- A provider proof can only unlink that same provider. It can't add a method, remove a passkey or
  unlink another provider: those need a password or passkey proof. Otherwise someone who took over
  the person's Google account could strip their passkeys and other providers. Setting a password, replacing the recovery key or
  deleting the account with a provider proof also needs the recovery key
  (`current_recovery_auth_key`). A compromised provider therefore can't add a second way in, take
  the break-glass path or destroy the server copy without a secret it doesn't have.

Passwords are not required. An account created with a passkey or a provider has `has_password = 0`;
`POST /v2/account/sign-in/password` sets the first password (with a `proof`), rewraps the data key
and rotates the recovery key; `POST /v2/account/password` still changes an existing one. Setting or
changing a password revokes every other session (`password_changed`), as today.

### Keys for passkeys and security keys

The WebAuthn PRF extension gives a credential a deterministic 32-byte output for an input the
relying party chooses. The input is not secret; the output is, and only that credential can produce
it.

- **The PRF input is one fixed value.** `prf_input = SHA-256("kanso/v1/passkey-prf" || 0x01)`,
  32 bytes, the same for every Kanso passkey. It is passed as `eval.first`; the WebAuthn client
  hashes it again with `"WebAuthn PRF" || 0x00` before the authenticator sees it (WebAuthn §10.1.4).
  A per-credential salt would be stronger domain separation, but it cannot be used for username-less
  sign-in: the client must choose `eval.first` before the authenticator tells it which credential
  the person picked, and `evalByCredential` is only allowed when `allowCredentials` is non-empty
  (WebAuthn §10.1.4). A second assertion just to learn the credential would ask the person twice.
  The separation that matters is kept in the KEK's `info`, which names the credential.
- **The output.** The ceremony returns `prf.enabled` and, when it is true, `prf.results.first`, a
  32-byte `ArrayBuffer`, read in the app. No other output is used, and no web page ever handles it
  (see "Where the ceremony runs").
- **The CTAP2 mapping on Linux.** CTAP2's `hmac-secret` extension returns the same
  credential-scoped secret. The app computes
  `ctap_salt = SHA-256("WebAuthn PRF" || 0x00 || prf_input)`—the same salt WebAuthn computes—and
  passes it as `salt1` in `authenticatorGetAssertion`, requiring user verification. The
  authenticator returns `HMAC-SHA256(credential_secret, ctap_salt)`, which is `prf_out` exactly;
  it is not hashed again. `passkey_kek` below is unchanged. libfido2 sets and reads the salt and
  result (`fido_assert_set_hmac_salt`, `fido_assert_hmac_secret_ptr`).
- **The passkey KEK.**

      prf_out     = PRF(credential, prf_input)                       (32 bytes)
      passkey_kek = HKDF(ikm  = prf_out,
                         salt = "kanso/v1/passkey" || 0x01 || lp(origin),
                         info = "kanso/v1/passkey-kek" || 0x01 || lp(account_id)
                                || lp(credential_id))

  `origin` is the account's origin as today, `credential_id` is the raw credential ID in base64url.
  Binding the credential ID means a wrap moved to another credential derives a different key and
  fails to open. `prf_input` is a protocol constant: changing it makes every passkey wrap
  unopenable, so the golden vectors pin it.
- **The wrap.** `wrapped = 0x01 || nonce(24) || XChaCha20-Poly1305(passkey_kek, nonce, aad,
  data_key)`, with the existing AAD scheme and a new purpose byte `0x4B` (`K`):
  `aad = "kanso-wrap" || 0x01 || 0x4B || lp(account_id)`. It is 73 bytes, 98 base64url characters,
  and is stored on the credential's row.
- **One wrap per credential.** A credential is registered on one device but may be used on any
  device the platform syncs it to (iCloud Keychain, Google Password Manager, 1Password, a security
  key). The same credential gives the same PRF output there, so the same wrap opens. A credential
  registered without PRF has `wrapped_key` NULL; adding PRF later means registering the credential
  again (a new credential ID) and removing the old one.
- **The key is checked.** After unwrapping, the client compares `key_id(data_key)` with the
  server's `key_id`; a difference is `KeyMismatch`, as for a password.

A passkey that has a wrap both signs in and unlocks. A passkey without one signs in; the device then
needs approval or the recovery key. The app says which at registration ("This passkey can unlock
Kanso on a new device" / "This passkey signs in; a new device will need approval").

#### Rotations

| Event | Data key | Wraps | Sessions | Passkeys |
|---|---|---|---|---|
| Set the first password | same | add the password wrap; rotate the recovery key | every other session revoked (`password_changed`) | kept |
| Change the password | same | replace the password wrap; rotate the recovery key | every other session revoked (`password_changed`) | kept, and listed on the success screen |
| Recover with the recovery key | same | replace the password wrap and the recovery wrap (the used key stops working) | every session revoked (`recovered`), then this device's new session | **all removed**, with their wraps |
| Add a passkey | same | add that credential's PRF wrap, when PRF was available | none | add |
| Remove a passkey | same | delete that credential's wrap | none (a device that already has the key keeps it) | remove |
| Link or unlink a provider | same | none | none | kept |
| Approve a new device | same | none on the server; the new device stores the key | none | kept |
| Rekey (deferred) | new | replace every wrap | all revoked (`rekeyed`) | all removed |

- **Why recovery removes passkeys.** A recovery is the break-glass path: it is what someone uses
  when they no longer trust the normal ways in. A passkey added by someone who once had the
  password must not survive it. The recovering device holds the data key, so re-adding a passkey is
  one ceremony away; the screen lists how many will go and says so before the recovery finishes.
- **Why a password change keeps them.** A password change already revokes every other session, and
  a passkey is a stronger credential than a password. Removing them would lock out a user who
  changes a password on a borrowed device. The success screen lists every passkey with when it was
  added and last used, and Settings lets the user remove any they don't recognize. A person who
  believes a passkey was added by someone else should recover, which removes them all.
- **Unlinking never touches keys.** A provider is only an authentication path; the data key's wraps
  are the password, the recovery key and passkeys.
- **A future rekey** replaces the data key, so every wrap and every passkey's wrap is replaced and
  every session is revoked. It is still deferred (see "Deferred").

### Passkeys

The Worker is the relying party. `PASSKEY_RP_ID` is the exact host of the deployment's public sync
address (`sync.kansoapp.app` for the owner's server), and `PASSKEY_ORIGINS` lists the https origins
(or `http://localhost` in development) the Worker accepts in `clientDataJSON.origin`; each must be
the RP ID or a subdomain of it. The Worker checks `rpIdHash == SHA-256(rp_id)`, the origin, the
challenge, the user-presence and
user-verification flags, the credential's signature, and the credential's account.

- **Registration** (`POST /v2/account/sign-in/passkeys/start` then `/finish`, session + `proof`):
  `authenticatorSelection` requires a discoverable credential (`residentKey: "required"`) and
  `userVerification: "required"`, with no attachment preference, so a platform passkey or a security
  key both work. `pubKeyCredParams` are ES256 (`-7`), EdDSA (`-8`) and RS256 (`-257`).
  `excludeCredentials` lists the account's credentials, so the same authenticator isn't registered
  twice. Attestation is `"none"`; the Worker records the COSE public key, algorithm, AAGUID (for
  display only), transports and sign count, and never validates a certificate chain. The user
  handle is `SHA-256("kanso/v1/webauthn-user" || lp(account_id))[0..16]`, opaque to the
  authenticator; the account is found by credential ID.
- **PRF at registration** is requested with `prf: {eval: {first: prf_input}}`. If the output is
  absent or `enabled` is false, the credential is stored without a wrap and the app says so. If it
  is present, the app derives `passkey_kek` and sends the 98-character wrap with the attestation.
- **Sign-in** (`POST /v2/auth/passkey/start` then `/finish`, no session): the start is
  username-less and returns an empty `allowCredentials`, which lets the platform show its passkey
  picker; the assertion's credential ID names the account. If the user typed a username first, the
  start may include that account's credential IDs, which helps security keys without discoverable
  storage. The finish returns a session, `key_id`, `epoch`, the credential's wrap if it has one,
  and whether the account has a password, so the app can offer every unlock path. It does not
  return the password wrap (see "Unlocking a new device"). The client requests the PRF output in the same `get()` with `eval.first = prf_input`; it
  needs no server input to do so, which is why username-less sign-in can unlock.

Linux submits the same two WebAuthn-shaped messages, but libfido2 produces the CTAP2 half. The
Worker therefore needs no Linux-specific route, attestation format or credential column. The only
new passkey advertisement bit is `security_key` in health.

- **Sign count.** A stored count of 0 means the authenticator doesn't count; it is never enforced.
  When both counts are above 0, a new count at or below the stored one is refused as a possible
  clone (`invalid_credentials`), and the row's count is updated on every higher one.
- **Transports** are stored as the space-separated hints the client saw (`usb`, `nfc`, `ble`,
  `hybrid`, `internal`), used only to help `allowCredentials`.
- **Several passkeys** per account are normal. The list shows when each was added and last used,
  and whether it can unlock. Removing one deletes its row and its wrap; it does not rewrap the data
  key, and it does not sign other devices out (a device that already has the key keeps it until
  sign-out).
  Removing the last sign-in method follows the recovery-proof rule above.

#### Where the ceremony runs

- **macOS 13 and later:** native AuthenticationServices
  (`ASAuthorizationPlatformPublicKeyCredentialProvider`), which needs the Associated Domains
  entitlement (`webcredentials:sync.kansoapp.app`) and a provisioning profile. A Developer ID
  provisioning profile can carry Associated Domains; a Mac app that uses only unrestricted
  entitlements needs no profile, but this is a restricted one, so `Kanso.app/Contents/
  embedded.provisionprofile` is required. If the native call fails (no associated domain, a denied
  prompt), the method isn't available and the app says so; it never falls back to a page. The
  Worker serves `https://sync.kansoapp.app/.well-known/apple-app-site-association`:
  `{"webcredentials":{"apps":["<TEAMID>.app.kanso.Kanso"]}}`, `Content-Type: application/json`, no
  redirects, valid TLS. Apple's CDN fetches it; changes can take hours.
- **PRF on macOS** needs macOS 15 (`ASAuthorizationPublicKeyCredentialPRFRegistrationInput` and
  `...PRFAssertionInput`, and `prf` on the registration and assertion requests). The app is built
  for macOS 13, so it checks `#available(macOS 15, *)`; on 13 and 14 a passkey still registers and
  signs in, without a wrap. On the platform side PRF needs iCloud Keychain on macOS 15; the
  browsers that use it, and the third-party credential managers that support it, vary (see the
  secondary note under "Sources"), so the app detects PRF per ceremony and never assumes it.
- **Windows 10 and 11:** the Win32 WebAuthn API (`webauthn.h`: `WebAuthNAuthenticatorMakeCredential`,
  `WebAuthNAuthenticatorGetAssertion`) with the same RP ID, which reaches Windows Hello and
  security keys. Registration asks with `bEnablePrf` and, when the WebAuthn API is V8 or newer,
  supplies the evaluation input as `pPRFGlobalEval` and reads the result from the attestation's
  `pHmacSecret`; assertions use `pHmacSecretSaltValues`. Where the OS or the authenticator gives no
  PRF output, the credential registers without a wrap. If the API call fails, the method isn't
  available and the app says so; it never falls back to a page. Step 2 records the
  `clientDataJSON.origin` each native API produces and pins `PASSKEY_ORIGINS` to it.
- **Linux:** the app is the CTAP2 client. It uses a safe Rust wrapper around libfido2 1.12 or newer,
  which drives USB HID and NFC security keys. USB HID is the required transport; NFC works where libfido2
  and the Linux host expose a reader, and PC/SC-only readers are not promised because distro builds
  vary. No Worker page is involved. `fido_dev_info_manifest` enumerates the authenticators; a udev
  monitor updates the button while the sign-in page is open, with enumeration when the form opens as
  the fallback.

  Registration calls `authenticatorMakeCredential` with `rk=true`, user verification required and
  the same RP ID, user handle and COSE algorithms as the WebAuthn routes. The app builds the standard
  `clientDataJSON` and `attestationObject` and sends them to
  `/v2/account/sign-in/passkeys/finish`. Sign-in is username-first so the app can pass the account's
  credential IDs as `allowList`; it calls `authenticatorGetAssertion` with `hmac-secret`, maps the
  result as above, and sends the standard assertion to `/v2/auth/passkey/finish`. If the key has no
  `hmac-secret`, the credential registers without a wrap and the app says it can sign in but not
  unlock a new device.

  The chosen Rust libfido2 bindings expose `hmac-secret` on assertions but not the make-credential
  `hmac-secret-mc` output. For a key that advertises `hmac-secret`, registration therefore follows
  with one immediate local assertion over a challenge derived from the registration challenge,
  requires UV and verifies the assertion against the new public key before sending the wrap; the user
  touches the key twice. If that second ceremony fails, no wrap is saved and the app says so. The
  Worker receives the same WebAuthn-shaped registration attestation and sign-in assertion as from
  macOS and Windows; the local assertion never leaves the app.

  The app never loads a page from the Worker and never asks a browser to run the ceremony. The
  rejected page design is still rejected: server code could take the PRF output and the wrap. The
  PIN, when the key needs one, is held in zeroizing memory and passed to libfido2; it never leaves
  the process.
- **Self-hosted servers.** Native macOS passkeys need the RP ID in the app's associated-domains
  entitlement, which is fixed at build time; the official app's is `sync.kansoapp.app`, so it hides
  platform passkeys for another server's domain even if that server advertises them. Windows native
  passkeys may work for any RP ID (to be confirmed on Windows 11 in step 2); if they don't, the app
  hides them there too. Linux security keys do not use the associated-domains entitlement, so they
  work with a self-hosted server whose health advertises a valid RP ID; the app checks that the RP
  ID is the server host or a parent before it starts the ceremony. A self-hosted owner who wants
  platform passkeys builds their own app with their domain in the entitlement. A server on an IP
  address or plain http (except `localhost`) advertises no passkeys, because WebAuthn has no valid
  RP ID there.

Passkeys are phishing-resistant because the credential is scoped to the RP ID and the system
checks the origin. The server never sees the private key and never sees the PRF output; it
sees the public key, the credential ID and the signatures.

### Sign in with Apple, Google and GitHub

All three follow the same shape: the app asks the Worker to start a flow, the person authenticates
with the provider in the system browser, and the Worker — not the app — completes the token
exchange and verifies the result. The app never receives an ID token, an access token or a client
secret. The Worker stores the provider's stable subject and discards everything else.

**Common rules.**

- The flow ID is 32 random bytes. Every start stores the device ID and SHA-256 of the device proof;
  the finish or poll must send the same device and proof, or 400 `bad_request`. This is what stops
  a flow ID captured from a callback URL from being claimed by another app.
- Every finish or poll is idempotent: repeating it never creates a second account, identity or
  device row. After a lost answer, the retry returns a fresh token for the same device, as the
  existing sign-up and login retries do; the new-account ticket stays the same until it is used.
- A lost `social/signup` or `passkey/signup/finish` answer is retried with the same body: the Worker
  finds the account by its account ID and username, checks the same identity or credential, recovery
  hash and key ID, and returns 201 with a fresh token instead of a second account.
- `state` is 32 random bytes, stored as SHA-256, compared in constant time, single use. `nonce` is
  32 random bytes, stored as SHA-256 in the flow and compared with the ID token's `nonce`; a
  replayed token has a consumed flow and is refused (`nonce_reused`).
- The only redirect URI is the one the flow started with: Apple's fixed `APPLE_REDIRECT_URI`,
  Google's loopback URI checked by shape. There is no open redirect, and no `return` parameter.
- The Worker fetches the token endpoint itself over https, with a 10 s timeout and no redirects. It
  never logs a token, an authorization code, an ID token or a subject.
- Each provider's JWKS URL is fixed in code, fetched with no redirects, cached for an hour per
  Worker isolate, and refetched once on an unknown `kid`; a second unknown `kid` is refused. The
  token's `alg` must be the one expected (RS256 for Google and Apple's ID token, ES256 only for
  Apple's client secret); `none`, HS256 and any other algorithm are refused.
- An identity that isn't linked returns `{"kind": "new_account", "ticket": "<flow id>",
  "suggested_username": "<name>"}` instead of a session. `POST /v2/auth/social/signup` consumes the
  ticket, creates the account with the chosen username, links the identity, inserts the session, and
  returns the same shape as a login. The ticket is single use and expires with the flow.
- A new account created this way has no password: `has_password = 0`, a random `auth_hash`, a
  random `kdf` salt and a `wrapped_key` under a random KEK that is zeroized immediately. The row
  satisfies every existing constraint, looks like any other row to the server's storage, and no
  password can open it. `GET /v2/account/sign-in` tells the client `"password": false`.
- Sign-up counts `signup-ip` and goes through the same `accounts` insert, so the `signup` setting,
  `max_accounts` and `signups_per_day` triggers apply. Sign-in counts `login-ip` and `login-user`
  (with the known-device exemption), whichever method it used.

**Sign in with Apple (web flow).** Apple's capability table lists Sign in with Apple under the
Apple Developer Program but not under Developer ID. A Developer ID build therefore cannot use the
native `ASAuthorizationAppleIDProvider`; the design uses the web flow on every platform, and a future
App Store build can add the native flow without changing the server.

The owner's Apple resources are configured as of **2026-10-23**:

- Team ID: `4973Y692JU`.
- The release App ID `app.kanso.Kanso` has Associated Domains.
- Primary App ID `app.kanso.signin` (`4973Y692JU.app.kanso.signin`) has Sign in with Apple.
- Services ID `app.kanso.Kanso.web` is grouped under that primary App ID, with domain
  `sync.kansoapp.app` and return URL `https://sync.kansoapp.app/v2/auth/apple/callback`.
- The Sign in with Apple key is named "Kanso Sign in with Apple", Key ID **`MS6Y3S5ZL7`**. Its `.p8`
  was uploaded to the `kanso-sync` Worker secret `APPLE_PRIVATE_KEY`, and the downloaded file was
  deleted; no key material is in this document. The Worker vars are `APPLE_CLIENT_ID=app.kanso.Kanso.web`,
  `APPLE_TEAM_ID=4973Y692JU`, `APPLE_KEY_ID=MS6Y3S5ZL7` and
  `APPLE_REDIRECT_URI=https://sync.kansoapp.app/v2/auth/apple/callback`.

Re-opening the Services ID in the live portal showed both website values persisted. Apple's current
web setup accepts the domain and return URL in the Services ID and needs no file on the server.
Apple also allowed this Services ID to be created without an App Store app, even though its help
page says the primary app must be on the App Store; that observed result is specific to this account
and date. None of this proves a real Apple sign-in: the routes are not implemented or deployed, and
the Worker vars are still to be set.

- `POST /v2/auth/apple/start` returns `{flow_id, auth_url}`. `auth_url` is
  `https://appleid.apple.com/auth/authorize` with the Services ID as `client_id`, the registered
  `redirect_uri`, `response_type=code`, `response_mode=form_post`, `scope=` (nothing: no name, no
  email, so the private relay never enters the picture), a fresh `state` and `nonce`.
- The app opens it in `ASWebAuthenticationSession` (macOS) or the system browser, with the callback
  scheme `kanso`.
- Apple posts `application/x-www-form-urlencoded` to
  `https://sync.kansoapp.app/v2/auth/apple/callback`. This route is the one exception to the
  `Origin` rule: a browser form post carries an `Origin` Apple's domain, so the route is public and
  relies on `state`, the code's five-minute life and the exact redirect URI instead. It checks
  `state`, exchanges the code at `https://appleid.apple.com/auth/token` with
  `client_id`/`client_secret`/`code`/`grant_type=authorization_code`/`redirect_uri`, and verifies
  the ID token: ES256/RS256 signature against `https://appleid.apple.com/auth/keys`, `iss` exactly
  `https://appleid.apple.com`, `aud` the Services ID, `exp` (120 s skew) and `nonce`. The client
  secret is an ES256 JWT with header `{alg:"ES256", kid:<Key ID>}` and claims
  `{iss:<Team ID>, iat, exp, aud:"https://appleid.apple.com", sub:<Services ID>}`, minted on demand
  and cached for at most a day (Apple's limit is six months).
- The callback stores the subject and the SHA-256 of a fresh secret on the flow, and answers an
  HTML page that redirects to `kanso://auth/apple?flow=<flow_id>&s=<secret>` (ASWebAuthenticationSession
  on macOS, the app's `kanso://` scheme handler on Windows and Linux); the app's session completes
  and it polls with `s`
  `POST /v2/auth/apple/poll` with its device every 4 s (the `AUTH_LIMITER` allows 20 a
  minute). The flow ID alone grants nothing: the poll carries the device's proof and is the only
  thing that creates a session.
- The name and email scopes are not requested. If an ID token ever carries `email`,
  `email_verified` or `is_private_email`, the Worker ignores them; a private relay address is never
  stored, so there is nothing to forward or delete.

**Google (installed app, loopback and PKCE).**

- The deployment has a Google Cloud OAuth client of type **Desktop app**: `GOOGLE_CLIENT_ID` and
  `GOOGLE_CLIENT_SECRET`. Desktop clients are public clients and the secret is not confidential;
  keeping it in the Worker means the app binary holds no provider secret and the exchange happens
  where the verification does.
- `POST /v2/auth/google/start` takes `{redirect_uri}`, checks it is exactly
  `http://127.0.0.1:<port>` (loopback only, no path, query or fragment, port 1024–65535), and
  returns `{flow_id, auth_url, nonce, state}` with `code_challenge` = base64url(SHA-256(verifier)),
  `code_challenge_method=S256`, `scope=openid`, `response_type=code` and
  `access_type=online`. The app keeps the verifier.
- The app opens the URL in the system browser and listens on `127.0.0.1:<port>` for one request
  (`ASWebAuthenticationSession` takes a custom scheme, not an http loopback redirect, so it isn't
  used here). It checks `state`, takes `code`, answers the browser tab with "Return to Kanso",
  closes the listener, and calls `POST /v2/auth/google/finish` with `{flow_id, code, code_verifier,
  device}`.
- The Worker checks the verifier against the stored challenge (constant time), exchanges the code
  at `https://oauth2.googleapis.com/token` with the client ID and secret, and verifies the ID token:
  RS256 against `https://www.googleapis.com/oauth2/v3/certs`, `iss` `https://accounts.google.com` or
  `accounts.google.com`, `aud` the client ID, `exp`/`iat` with 120 s skew, and `nonce`. It reads only
  `sub`. `email`, `name` and `picture` are ignored. No Google API is called and no access token is
  kept.
- The PKCE verifier never leaves the app until the finish request, which is over TLS to the
  Worker; the Worker uses it once.

**GitHub (device flow).** GitHub's OAuth apps support PKCE in the web flow, but the device flow is
better for a desktop app: no loopback listener, no client secret, and a code the user can read.

- `POST /v2/auth/github/start` calls `https://github.com/login/device/code` with `client_id` and no
  scope, and returns `{flow_id, user_code, verification_uri, expires_in, interval}`. The OAuth app
  must have Device flow enabled in its settings.
- The app shows `user_code`, opens `verification_uri` (`https://github.com/login/device`) in the
  system browser, and polls `POST /v2/auth/github/poll` with `{flow_id, device}` no faster than
  `interval`. The Worker enforces the interval (it stores `next_poll_at`), calls
  `https://github.com/login/oauth/access_token` with
  `grant_type=urn:ietf:params:oauth:grant-type:device_code`, and answers `{"status":"pending"}`
  until GitHub returns a token. `slow_down` adds five seconds; `expired_token` is 410
  `flow_expired`; `access_denied` is 403 `provider_denied`.
- With a token, the Worker calls `GET https://api.github.com/user` with a
  `User-Agent: kanso-sync/<version>` (GitHub requires one), reads `id` (the stable numeric user ID)
  as a decimal string, and discards the token at once — no column, no log, no retry. The
  login name, email and avatar are ignored. GitHub issues no ID token, so there is no nonce to
  check; the device flow's `device_code` and the Worker's `client_id` are the binding, and the
  token exchange happens server-side.
- There is no web + PKCE fallback (an owner decision): GitHub is the device flow only, and the
  Worker holds no GitHub client secret.

**Adding another provider** means a module that starts a flow, exchanges a code or polls a device
endpoint, and returns a stable subject; a row in `account_identities`; its secrets; and its name in
the health advertisement. The schema, the session code, the approval protocol and the app's list
are unchanged.

### Unlocking a new device

A device that signed in with Apple, Google, GitHub, or a passkey without PRF has a session but not
the data key. It cannot sync and the server can't help beyond ciphertext. Three paths unlock it, in
the order the app offers them:

1. **The password**, if the account has one. The app asks for the password and signs in again
   with it (`POST /v2/auth/login`, same device), which returns the password wrap only after the
   Worker has checked the auth key, under the per-account sign-in limit. A passkey or provider
   sign-in never returns the password wrap: anyone who controls a person's Google or GitHub
   account, or a synced passkey without PRF, could otherwise take the wrap away and guess the
   password offline at their own speed.
2. **An already signed-in device** (below).
3. **The recovery key**, through `recover/start`, which also sets a new password.

The app doesn't bind the library or start a sync until one succeeds. A session in this state is
"locked": it can list devices, request approval and sign out, but not push, pull or read the
account's documents; the Worker can't tell a locked session from an unlocked one, and doesn't need
to.

#### The protocol

`N` is the new device, `A` an already signed-in, unlocked device. X25519 keys are ephemeral and used
once. All binary values are base64url; `lp` is as in "Encodings".

1. **N asks, committing to its key.** It generates `(esk_N, epk_N)` and a stable, canonical
   32-byte base64url `request_id`, then calls `POST /v2/account/sign-in/approvals` with
   `{request_id, commit: c_N}`, where `c_N = SHA-256("kanso/v1/approve-commit" || 0x01 || epk_N)`.
   It does not send `epk_N` yet. The Worker records `request_id` as the request's ID, sets
   `expires_at = now + 5 minutes`, and answers `{id, expires_at, other_devices}` where
   `other_devices` counts the account's active sessions other than N's. Repeating the same
   `(request_id, account, device, commit)` returns that request without creating another row or
   counting `approval-create`; reusing the ID for a different tuple is 409
   `approval_request_id_conflict`. At most three pending requests exist per account; a fourth
   cancels the oldest. A new request counts `approval-create` (10 an hour).

   **Why N commits first.** The code is short, so whoever picks the last public key must not be
   able to try keys until the codes match. Without the commitment a malicious server could give A
   a key of its own, read `epk_A`, then try about a million keys toward N until N's code equals
   A's: seconds of work. With it, each side's key is fixed before the other's is known. N commits
   before it sees `epk_A`. The server must hand N its stand-in for `epk_A` before N reveals `epk_N`.
   It must commit toward A before A posts `epk_A`. So a substituted key matches the codes with
   probability one in a million per attempt, and every attempt is a visible request the user
   can cancel. This is the commit-then-reveal shape of Bluetooth numeric comparison and ZRTP.
2. **A sees it.** While unlocked and running, A holds the account event socket. The Worker
   publishes `approval.created` after the row commits, so A fetches
   `GET /v2/account/sign-in/approvals` once (the same GET runs at launch, when Settings › Account
   opens, and after every socket reconnect) and sees
   `{id, device_id, commit, created_at, expires_at}`. It shows "A device wants to join *ngoni*". It
   generates `(esk_A, epk_A)` and calls
   `POST /v2/account/sign-in/approvals/<id>/challenge` with `{epk: epk_A}`. A can't compute a code
   yet: it has only N's commitment.
3. **The code, once both keys are known.** N computes it after step 4's challenge arrives; A
   computes it in step 5, after N has revealed `epk_N` and A has checked it against `c_N`. With
   `dh = X25519(esk_A, epk_N)` (N computes `X25519(esk_N, epk_A)`; both refuse an all-zero
   result):

       approval_key = HKDF(ikm  = dh,
                           salt = "kanso/v1/approve" || 0x01 || lp(account_id),
                           info = "kanso/v1/approve-key" || 0x01 || lp(request_id)
                                  || lp(epk_N) || lp(epk_A))
       code_bytes   = HKDF(ikm  = dh,
                           salt = "kanso/v1/approve-code" || 0x01 || lp(account_id),
                           info = "kanso/v1/approve-code" || 0x01 || lp(request_id)
                                  || lp(epk_N) || lp(epk_A), 4 bytes)
       code         = (code_bytes as a big-endian u32) mod 1_000_000, six digits, shown "123 456"

   The Worker can compute neither: it has no private key. A server that substitutes either public
   key ends up with two different shared secrets, and because of the commitment in step 1 it can't
   choose its stand-in keys to make the two codes agree; it matches with probability 10⁻⁶ per
   request.
4. **N names itself.** N polls
   `GET /v2/account/sign-in/approvals/<id>` and receives `{status: "challenged", epk: epk_A,
   approver_device_id}`.
   It derives `approval_key` and the code, shows the code, and posts
   `POST /v2/account/sign-in/approvals/<id>/name` with `{epk: epk_N, sealed_name}`, revealing its
   key for the first time, with its device name sealed:

       sealed_name = 0x01 || nonce(24) ||
                     XChaCha20-Poly1305(approval_key, nonce, aad, name)
       aad         = "kanso-approve-name" || 0x01 || lp(account_id) || lp(request_id)
                     || lp(epk_N) || lp(epk_A)

   The name is the device-name rule already in "Sealed device names"; it is 55–400 characters.
   Posting it after the challenge is what keeps it sealed from the Worker.
5. **A checks and approves.** A receives `approval.named`, fetches
   `GET /v2/account/sign-in/approvals` once and gets `epk_N`. It checks
   `SHA-256("kanso/v1/approve-commit" || 0x01 || epk_N) == c_N` and stops with "This request
   changed after it was made. Don't approve." if not. Then it derives `approval_key` and the code,
   opens `sealed_name`, and shows the device name, the approximate time and the code:
   "A device wants to join *ngoni*: *Alex's MacBook Air*, asked at 14:02. Does it show 123 456?
   Only approve if it does." On approval it computes

       wrapped = 0x01 || nonce(24) || XChaCha20-Poly1305(approval_key, nonce, aad, data_key)
       aad     = "kanso-approve" || 0x01 || lp(account_id) || lp(request_id) || lp(epk_N)
                 || lp(epk_A) || lp(new_device_id) || lp(approver_device_id)

   (73 bytes, 98 characters) and calls
   `POST /v2/account/sign-in/approvals/<id>/approve` with `{wrapped_key}`. The Worker refuses a
   request from N's own device (`self_approval`, 403) and a request from an account other than the
   session's (404).
6. **N opens it.** N's poll returns `{status: "approved", epk_A, approver_device_id,
   wrapped_key}`. It checks the AAD
   (its device ID, A's device ID, both ephemeral keys, the request ID), unwraps, compares `key_id`
   with the server's, stores the data key in the Keychain, deletes the pending item, and calls
   `PUT /v2/account/device` with its sealed name. Then it binds the library and syncs.

If the codes differ, N shows "That code doesn't match. Don't approve. Cancel and try again." and
offers [Cancel]; the user must not approve on A. If A never answers, N offers the recovery key after
two minutes, and immediately if `other_devices` was 0. Either device can cancel with
`DELETE /v2/account/sign-in/approvals/<id>`; the Worker publishes `approval.cancelled`, N's poll sees
`status: "cancelled"`, and both stop.

**Events are hints, never state.** A frame carries only the event type and the approval ID. It never
carries the device name, the six-digit code, an ephemeral key, the wrapped key or note content. If a
frame is late, lost, or the socket is down, the next launch, Settings open, foreground or reconnect
`GET` catches up. The Durable Object keeps no event log; the D1 approval row is authoritative.

**Expiry, single use and retries.** A request lives five minutes from creation. A may answer it
once; a second `/approve` on an answered request is 409 `approval_used`. N may read the approved
result until it deletes the request or it expires, so a lost response is retried by polling again.
The Worker deletes expired and completed-and-deleted rows in the cron, and marks `expires_at` in
every answer. A request is bound to the account, both device IDs, N's commitment and both ephemeral keys; none of
those can be changed once set. The server can't replay an old approval into a new request
(the AAD's `request_id` and both keys differ), can't read the data key, can't read the name and
can't compute the code.

**Zero other devices.** If `other_devices` is 0 the app skips the request and shows "No other device
is signed in. Use your recovery key." If the count was stale and no device answers, the waiting
screen says the same after two minutes. The recovery key is always available: every sign-up creates
one, including a passwordless one.

**What the approval trusts.** Any unlocked session of the account can approve, because any of them
already holds the data key. A stolen session alone (without the device's Keychain item) cannot
approve; a stolen unlocked device can, like it can read the library. The prompt's name and code are
what let the person spot a request they didn't make.

### Prior art

Signal links a new device by showing a QR code on it that an existing device scans, after which the
existing device sends the account's keys over an ephemeral-key channel. The QR is the
out-of-band channel; there is no separately compared code, and the protocol details aren't public
(unverified). Bitwarden's "Log in with device" sends the request to a signed-in app, shows a
fingerprint phrase on both, and makes the approving device confirm the phrase matches before it
grants access; how the phrase is derived isn't public (unverified). Apple's two-step sign-in shows
a code on a trusted device and asks for it on the new one, which proves the person holds both, not
that two devices derived the same keys. 1Password uses the PRF extension to encrypt data with a
passkey, which is the construction this design borrows for passkeys.

This design takes the ephemeral-key transfer from Signal and the compared phrase from Bitwarden,
but derives a six-digit code from the DH secret and shows it on **both** screens. A one-tap
approval, or a code shown on one side only, lets a malicious relay pass; a compared code derived
from both keys does not.

### Sources

Verified against the linked page unless marked secondary:

- WebAuthn Level 3, the PRF extension (salt hashing, `enabled`, `results`, 32-byte output):
  <https://www.w3.org/TR/webauthn-3/#prf-extension>. The CTAP salt is
  `SHA-256(UTF8("WebAuthn PRF") || 0x00 || eval.first)`.
- Sign in with Apple for the web (domains and return URLs on the Services ID; no server file):
  <https://developer.apple.com/help/account/capabilities/configure-sign-in-with-apple-for-the-web/>.
- libfido2 (USB and NFC security keys on Linux; `hmac-secret` salt/secret API):
  <https://developers.yubico.com/libfido2/> and
  <https://developers.yubico.com/libfido2/Manuals/fido_assert_set_authdata.3/>.
- Durable Objects WebSocket Hibernation (accept, handlers, auto-response, hibernated sockets):
  <https://developers.cloudflare.com/durable-objects/best-practices/websockets/>; duration billing
  pauses while hibernated: <https://developers.cloudflare.com/durable-objects/platform/pricing/>.
- Apple passkeys and the associated-domain requirement:
  <https://developer.apple.com/documentation/authenticationservices/supporting-passkeys>.
- Apple's PRF inputs and outputs, macOS 15 / iOS 18:
  <https://developer.apple.com/documentation/authenticationservices/asauthorizationpublickeycredentialprfregistrationinput-swift.struct>
  and
  <https://developer.apple.com/documentation/authenticationservices/asauthorizationpublickeycredentialprfassertioninput-swift.struct>.
- Apple associated domains (AASA, entitlement, Developer ID):
  <https://developer.apple.com/documentation/xcode/supporting-associated-domains> and
  <https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.associated-domains>.
- Apple's macOS capability table: Associated domains is available under Developer ID; Sign in with
  Apple is listed only under the Apple Developer Program:
  <https://developer.apple.com/help/account/reference/supported-capabilities-macos/>.
- Provisioning profiles and restricted entitlements (TN3125):
  <https://developer.apple.com/documentation/technotes/tn3125-inside-code-signing-provisioning-profiles>.
- Sign in with Apple: environment and Services ID,
  <https://developer.apple.com/documentation/signinwithapple/configuring-your-environment-for-sign-in-with-apple>;
  the web flow, `form_post` and its parameters,
  <https://developer.apple.com/documentation/signinwithapple/incorporating-sign-in-with-apple-into-other-platforms>;
  identity-token validation (`iss`, `aud`, `exp`, `nonce`),
  <https://developer.apple.com/documentation/signinwithapple/verifying-a-user>; the client-secret
  JWT (`ES256`, `kid`, `iss`, `sub`, `aud`),
  <https://developer.apple.com/documentation/accountorganizationaldatasharing/creating-a-client-secret>;
  code validation, <https://developer.apple.com/documentation/signinwithapplerestapi/generate-and-validate-tokens>;
  the private relay, <https://developer.apple.com/documentation/signinwithapple/communicating-using-the-private-email-relay-service>.
- Google OAuth for installed apps (system browser, loopback, public clients, PKCE):
  <https://developers.google.com/identity/protocols/oauth2/native-app>.
- Google ID-token verification (JWKS, `iss`, `aud`, `exp`, `nonce`; `sub` is the identifier):
  <https://developers.google.com/identity/gsi/web/guides/verify-google-id-token> and
  <https://developers.google.com/identity/protocols/oauth2/openid-connect>.
- GitHub OAuth (web flow with PKCE; device flow), and the stable numeric `id` from `GET /user`:
  <https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps> and
  <https://docs.github.com/en/rest/users/users#get-the-authenticated-user>.
- Windows Win32 WebAuthn (`webauthn.h`), including `pPRFGlobalEval` and `pHmacSecretSaltValues`:
  <https://learn.microsoft.com/en-us/windows/win32/api/webauthn/> and
  <https://learn.microsoft.com/en-us/windows/win32/api/webauthn/ns-webauthn-webauthn_authenticator_make_credential_options>.
- 1Password's PRF support for passkeys it stores:
  <https://1password.com/blog/encrypt-data-saved-passkeys>.
- CTAP2 `hmac-secret`, which PRF is modelled on:
  <https://fidoalliance.org/specs/fido-v2.1-ps-20210615/fido-client-to-authenticator-protocol-v2.1-ps-errata-20210615.html#sctn-hmac-secret-extension>.

- Bitwarden's log-in-with-device flow and its compared fingerprint phrase:
  <https://bitwarden.com/help/log-in-with-device/>.

Secondary, not independently verified: the current mix of PRF support across platforms and
credential managers (Apple Passwords and Google Password Manager report the most consistent
success, third-party managers vary; Windows Hello returns PRF on Windows 11 24H2+ from the February
2026 update; current Chrome, Edge, Firefox and Safari expose it in their own passkey UIs):
<https://www.corbado.com/blog/passkeys-prf-webauthn>. Signal's linked-device protocol and
Bitwarden's phrase derivation are not public; only their user-facing behaviour is claimed above.

## Usernames

- **Normalizing,** on every route that takes a username and on the client. All three steps happen
  before any lookup, rate-limit key or fake answer:
  1. Strip leading and trailing U+0009, U+000A, U+000C, U+000D and U+0020. These five only: not JS
     `trim()`, which strips more.
  2. Reject (`invalid_username`) any remaining character outside U+0021–U+007E. This comes before
     lowercasing, because JS `toLowerCase` maps U+212A KELVIN SIGN to `k`.
  3. ASCII-lowercase.
- **Valid.** `^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$`: 3–32 characters, starting and ending with a
  letter or digit. A `CHECK` repeats this in D1.
- **Unique** on the normalized form.
- **Generated at a social or passkey sign-up.** The form shows a suggestion made from a small
  wordlist and four digits (for example `quiet-otter-4821`, which fits the pattern), checked against
  the reserved list and `UNIQUE`; the user may type another before the account is created. Every
  account therefore has a username, and none of the identity, invite, recovery or `AccountRef` code
  needs a nullable one. The user can't rename it in v1 (shares and `kanso-identity-bind` embed it);
  pick a good one at sign-up.
- **Reserved,** at sign-up only (`reserved_username`): `abuse admin administrator anonymous api
  everyone guest help hostmaster kanso me null owner postmaster root security server support sync
  system undefined webmaster www`.
- **Shared fixture.** `cloud/test/fixtures/usernames.json` (owner: the Worker builder) lists
  `{input, normalized, error}` cases. `normalized` is filled for reserved names too, and `error` is
  `invalid_username`, `reserved_username` or null. It includes U+00A0, U+2028, U+000B, U+212A,
  U+FEFF and each of the five trimmed characters. `cloud/src/usernames.ts` and
  `crates/kanso-sync/src/username.rs` both run it.
- **Enumeration.** Sign-up must say `username_taken`, so names can be enumerated there. That route
  is rate-limited per IP and capped per day. Prelogin answers an unknown name with a deterministic
  fake salt, fake account ID and default parameters, and sign-in fails the same way for an unknown
  name as for a wrong password.

## HTTP protocol (v2)

### Conventions

- **Base URL.** The server URL (default `https://sync.kansoapp.app`), checked by
  `check_endpoint`: https, or http to loopback only. A host ending in a dot is refused, since it would
  give the same server a second origin.
- **Browser requests (M7).** Any request with an `Origin` header gets 403 `forbidden_origin`,
  checked first; the native client never sends one. One route is an exception:
  `POST /v2/auth/apple/callback` accepts Apple's form post (origin `https://appleid.apple.com`, or
  none) and relies on `state` and the exact redirect URI. Requests with a body must have the right
  `Content-Type`, or they get 415 `unsupported_media_type`: `application/json` (with an optional
  `charset=utf-8`) for JSON, `application/octet-stream` for file uploads, and the form type on the
  Apple callback.
- **JSON bodies.** Required fields must be present with the right type. Unknown keys are ignored in
  requests and responses. Auth and account request bodies are limited to 8 KiB.
- **Sessions.** Session routes take `Authorization: Bearer kanso_<43 base64url>`. A header of any
  other shape gets 401 without touching D1. The one WebSocket route also takes
  `X-Kanso-Device` and `X-Kanso-Device-Proof`; its upgrade is authenticated exactly once.
- **Errors** are `{"error": "<code>", "message": "<English, for logs>", ...extras}`. The client maps
  codes to its own words and never shows `message`.
- **Unknown paths.** `/v1/*` and every other unknown path answer 404 `not_found`.
- **Client.** Blocking `ureq`, with `max_redirects(0)` so a redirect can never re-send a proof.
  Response size caps:

  | Response | Cap |
  |---|---|
  | Health, auth and account routes | 64 KiB |
  | Push and changes | 12 MiB |
  | Files | 10,489,897 bytes |

| Status | Code | Extras | When |
|---|---|---|---|
| 400 | `bad_request` | | Malformed JSON, wrong types, bad ID, revision, length, encoding or device ID, sealed under 55 characters, file body under 41 bytes |
| 400 | `invalid_username` | | Fails the rules |
| 400 | `reserved_username` | | Sign-up only |
| 400 | `unsupported_kdf` | | `kdf.alg` isn't `argon2id` or `kdf.version` isn't 19 |
| 400 | `weak_kdf` | | Parameters outside the floor or ceilings |
| 400 | `username_mismatch` | | `account/delete` with a username that isn't the account's |
| 400 | `checksum_mismatch` | | File body doesn't match `x-kanso-sha256` |
| 400 | `flow_expired` | | A passkey, proof, provider or approval flow is unknown, consumed or past its time |
| 400 | `nonce_reused` | | An ID token whose nonce the flow already used |
| 400 | `unsupported_method` | | A method this deployment doesn't offer |
| 400 | `provider_state` | | A provider callback with a missing or mismatched `state` |
| 401 | `unauthorized` | `reason`: `unknown`, `signed_out`, `revoked`, `password_changed`, `recovered`, `evicted`, `expired` or `rekeyed` | Session missing, malformed, unknown, revoked or expired |
| 401 | `invalid_credentials` | | Wrong proof, an unknown username, or a failed passkey assertion |
| 401 | `reauth_required` | `reason`: `reauth` | A sensitive change without a usable `proof` |
| 403 | `forbidden_origin` | | An `Origin` header, except on the Apple callback |
| 403 | `account_disabled` | | Disabled by the owner. Checked after credentials. Logout still answers 204. |
| 403 | `signup_closed` | `reason`: `closed`, `full` or `busy` | Switched off, `max_accounts` reached, or `signups_per_day` reached |
| 403 | `provider_disabled` | | A provider that isn't configured on this server |
| 403 | `provider_denied` | | The person declined at the provider |
| 403 | `self_approval` | | A device trying to approve its own request |
| 404 | `not_found` | | Unknown route, file, well-formed but unknown device, identity, passkey or approval |
| 405 | `method_not_allowed` | `Allow` header | |
| 409 | `username_taken` | | |
| 409 | `account_id_taken` | | Sign-up with an existing account ID (client retries with a new one) |
| 409 | `identity_taken` | `provider` | The provider identity belongs to another account |
| 409 | `last_method` | | The change would leave no password, identity or passkey this server offers, without a recovery proof |
| 409 | `already_has_password` | | `sign-in/password` on an account that has one |
| 409 | `no_password` | | `account/password` on an account that has none |
| 409 | `credential_exists` | | A passkey credential ID already registered |
| 409 | `approval_used` / `approval_expired` | | A second approval, or an expired request |
| 413 | `too_large` | | More than 200 documents, a push body over 4 MiB, sealed over 1,900,000 characters, a file over 10,489,897 bytes, an auth body over 8 KiB |
| 413 | `quota_exceeded` | `quota`: `docs`, `doc_bytes`, `files`, `file_bytes` or `server`; `used` (usage before this request); `limit` | When several apply, the first in that order |
| 415 | `unsupported_media_type` | | Wrong `Content-Type` |
| 429 | `rate_limited` | `retry_after` (seconds to the end of the window, rounded up), `Retry-After` header, `limit`: the rule name, `binding` or `session-account` (the bindings), `provider-poll`, or `writes-global` or `uploads-global` | A rate-limit rule or binding, or a server-wide daily cap |
| 500 | `internal` | | |
| 503 | `misconfigured` | | `AUTH_PEPPER` missing or malformed |
| 503 | `provider_unavailable` | `provider` | The provider's endpoint failed, timed out or answered with a 5xx |

### Rate limits

- **First layer, the bindings.** Two Workers rate-limit bindings, which write nothing to D1. Over
  either, the answer is 429 with `retry_after: 60`. If a binding is missing (unit tests), its layer
  is skipped. Like every Workers rate limit they count per Cloudflare location and settle
  eventually, so they are a filter in front of D1, not an exact cap.
  - `AUTH_LIMITER` (namespace `"1001"`, 20 requests per 60 s) is keyed by the caller's IP group and
    covers every `/v2/auth/*` and `/v2/health` request (`limit` `binding`).
  - `SESSION_LIMITER` (namespace `"1002"`, 120 requests per 60 s) is keyed by the account ID and
    covers every session route except logout, right after the session lookup (`limit`
    `session-account`). One signed-in account can't keep the shared database busy, from however
    many IPs it uses (security review 2, finding 2). Logout stays behind `AUTH_LIMITER` only.
- **Second layer, D1.** D1 is the authority, in fixed windows, and counts first (H1):

  ```sql
  INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, ?3)
  ON CONFLICT (key) DO UPDATE SET
    count = CASE WHEN window_start >= excluded.window_start THEN count + excluded.count ELSE excluded.count END,
    window_start = max(window_start, excluded.window_start)
  RETURNING count
  ```

  Here `?2 = floor(now / window) * window`, and `?3` is the amount (1, or the number of documents in
  a push). The upsert runs before credentials are checked or any work is done. If the returned
  count is over the maximum, the request is refused with 429, so parallel requests can't pass
  together. A request stamped with an earlier window than the row's (begun just before the
  boundary, or from a Worker whose clock is a little behind) counts against the newer window
  rather than resetting it, as the `daily` counters do (security review 2, finding 8).
- **IP groups.** IPv4 counts per address and IPv6 per /48, in D1 and in the binding alike. Flow IDs in `provider-poll` are stored as keyed hashes, like IPs. A /48
  is the smallest block a home or small office is commonly given, so per /56 one subscriber could
  rotate through 256 groups (security review 2, finding 9). The value comes from `CF-Connecting-IP`;
  `unknown` stands in when it's absent (local dev).
- **Expiry.** A rule's rows are deleted once their window has ended (by the cron; the counting
  upsert also overwrites a stale window).

| Rule | Subject | Window | Max | Counts |
|---|---|---|---|---|
| `signup-ip` | IP group (/48) | 1 h | 10 | Every sign-up request |
| `login-user` | account ID; one shared row for names without an account | 15 min | 10 | Every sign-in, unless its device ID has an active session row (`revoked_at IS NULL`) for that account and SHA-256 of the request's device proof equals that row's `proof_hash` (M8, security re-check 5, security review 2) |
| `login-ip` | IP group (/48) | 15 min | 30 | Every sign-in |
| `recover-ip` | IP group (/48) | 1 h | 10 | Every `recover/start` and `recover/finish` |
| `password-account` | account ID and device (the device ID's first 64 bits) | 15 min | 5 | Every current-password proof (password change, set, new recovery key, delete, linking, unlinking, passkeys) |
| `password-total` | account ID | 15 min | 15 | The same requests, counted only after the device's `password-account` passes |
| `proof-account` | account ID and device | 1 h | 20 | Every passkey or provider proof for a sensitive change (`sign-in/proof/start`, link and proof flows) |
| `proof-total` | account ID | 1 h | 60 | The same requests, counted only after the device's `proof-account` passes |
| `passkey-challenge` | IP group (/48) | 15 min | 30 | Every passkey sign-in start (`passkey/signup/start` counts `signup-ip`) |
| `passkey-manage` | account ID | 1 h | 20 | Every passkey registration and removal |
| `approval-create` | account ID | 1 h | 10 | Every new device-approval request |
| `approval-poll` | account ID | 1 h | 900 | Only the requester's `GET /v2/account/sign-in/approvals/<id>` is counted; at most three requests, each polled every 2 s for its five-minute life, plus retry headroom. The approver's list GETs are not counted; it uses the event socket plus launch, Settings and reconnect GETs |
| `approval-respond` | account ID | 1 h | 30 | Every challenge, name and approval answer |
| `provider-poll` | flow ID | 15 min | 300 | Every GitHub device-flow poll and Apple callback poll; 20 a minute, enough for GitHub's 5 s interval; the Worker also enforces that interval |
| `writes-account` | account ID | 1 day | the account's `quota_writes_per_day`, else `settings.writes_per_day` | Documents in each push, plus 1 per file upload, plus 1 per push refused as malformed or too large (so failed pushes aren't free) |
| `uploads-account` | account ID | 1 day | `settings.upload_bytes_per_day` | Each upload's `Content-Length` (capped at the largest file plus one byte; the largest file when it's missing), counted before the body is read, after `writes-account` passes; a body longer than it declared counts the rest |

The proof rules count per device first (security review of PR #67, finding 3). One session sees the
same `password-account` and `proof-account` answers it always did, but it can use up at most a third
of the account's total and none of another device's own allowance. A stolen session therefore can't
lock a passkey-only owner out of proofs. Deleting the account deletes all of these rows.

There is no per-username recovery rule, so nobody can lock someone else out of recovery (M8). A
sign-up past `signups_per_day` (server-wide per UTC day, counted by a trigger in `daily`; deleting
an account doesn't give one back) gets 403 `signup_closed` with reason `busy`. Two more server-wide
daily caps are counted by triggers in `daily`, not here: stored document writes
(`global_writes_per_day`) and uploaded file bytes (`global_upload_bytes_per_day`). Past either, a
push or upload gets 429 with `limit` `writes-global` or `uploads-global` and `retry_after` up to the
next UTC midnight (security re-check 2), but only for an account above its fair share.

**Fair shares** (security review 2, finding 3). First come, first served, five accounts at their
own upload budgets, or fifteen at their write budgets, could use a cap up by noon and pause every
other Mac until midnight. So each cap has a fair share per account: the cap ÷ `max_accounts`, in
SQLite's integer division (6,000 writes and 100,000,000 upload bytes by default). Past the cap, the
trigger refuses a write only when the account's own count today is above its fair share:
`writes-account` for documents, `uploads-account` for upload bytes, both counted by the Worker
before the batch, this request included. An account at or below its fair share is never refused
by a server-wide cap. Fail closed: a missing setting, `max_accounts` 0, or no row for today in
`limits` counts as above the fair share. The price is the bound: once the cap is reached, every
account may still use its fair share, so a day's total stays under about twice the cap (the cap,
plus `max_accounts` fair shares, plus the request that crossed it, plus a fair share for each
account created that day).

Login keys `login-user` by account, so attempts on many unknown names share one row. That tells a
prober whether a name exists, which sign-up's `username_taken` already does.

**Floods.** A `workers.dev` address can't have WAF rules. Surviving a real flood needs a custom
domain with a Cloudflare WAF rate-limiting rule in front of the Worker. Expensive requests from a
valid session, which an IP-keyed WAF rule can't stop, are bounded by `SESSION_LIMITER` per account. Accounts are bound to their
server's origin (M5), so the owner must choose the final URL before the first account is created.

### Routes

| Method | Path | Session | Purpose |
|---|---|---|---|
| GET | `/v2/health` | no | Liveness, protocol version, whether sign-ups are open, and the sign-in methods this server offers |
| POST | `/v2/auth/prelogin` | no | KDF parameters, salt and account ID for a username |
| POST | `/v2/auth/signup` | no | Create an account and this device's session |
| POST | `/v2/auth/login` | no | Sign in a device |
| POST | `/v2/auth/recover/start` | no | Prove the recovery key; get the recovery-wrapped data key |
| POST | `/v2/auth/recover/finish` | no | Set a new password and a new recovery key |
| POST | `/v2/auth/passkey/start` / `/finish` | no | Sign in with a passkey; `/finish` returns a session, a wrap and a key ID |
| POST | `/v2/auth/passkey/signup/start` / `/finish` | no | Create an account with a passkey |
| POST | `/v2/auth/apple/start` / `/poll` | no | Sign in or sign up with Apple; the web flow's callback is below |
| GET/POST | `/v2/auth/apple/callback` | no | Apple's `form_post`; the one public route that accepts an `Origin` |
| POST | `/v2/auth/google/start` / `/finish` | no | Sign in or sign up with Google (loopback + PKCE) |
| POST | `/v2/auth/github/start` / `/poll` | no | Sign in or sign up with GitHub (device flow) |
| POST | `/v2/auth/social/signup` | no | Create the account a provider flow returned a ticket for |
| GET | `/v2/account/sign-in` | yes | Passwords, identities, passkeys and the recovery path |
| POST | `/v2/account/sign-in/proof/start` | yes | A challenge for a sensitive change's passkey proof |
| POST | `/v2/account/sign-in/password` | yes | Set the first password (and rotate the recovery key) |
| POST | `/v2/account/sign-in/passkeys/start` / `/finish` | yes | Register a passkey (and wrap the data key when PRF is available) |
| POST | `/v2/account/sign-in/passkeys/<credential_id>/remove` | yes | Remove a passkey |
| POST | `/v2/account/sign-in/identities/<provider>/start` | yes | Link a provider (or get a proof flow) |
| POST | `/v2/account/sign-in/identities/<provider>/unlink` | yes | Unlink a provider |
| GET | `/v2/account/events` | yes | WebSocket: live approval event hints for this account (session token + device proof) |
| POST | `/v2/account/sign-in/approvals` | yes | Ask another device to approve this one |
| GET | `/v2/account/sign-in/approvals` | yes | Pending approvals for launch, Settings and reconnect; not a timer |
| GET | `/v2/account/sign-in/approvals/<id>` | yes | The requester's poll |
| POST | `/v2/account/sign-in/approvals/<id>/challenge` | yes | The approver's ephemeral key |
| POST | `/v2/account/sign-in/approvals/<id>/name` | yes | The requester's sealed device name |
| POST | `/v2/account/sign-in/approvals/<id>/approve` | yes | The approver's sealed data key |
| DELETE | `/v2/account/sign-in/approvals/<id>` | yes | Cancel or decline an approval |
| POST | `/v2/auth/logout` | yes | Sign out this device |
| GET | `/v2/account` | yes | Account, usage and quotas |
| POST | `/v2/account/password` | yes | Change the password |
| POST | `/v2/account/recovery-key` | yes | Replace the recovery key |
| POST | `/v2/account/delete` | yes | Delete the account and its server data |
| PUT | `/v2/account/device` | yes | Set this device's sealed name |
| GET | `/v2/account/devices` | yes | List active devices |
| DELETE | `/v2/account/devices/<device_id>` | yes | Sign out one device |
| PUT | `/v2/account/devices/<device_id>/push` | yes | Set this device's push token |
| DELETE | `/v2/account/devices/<device_id>/push` | yes | Turn this device's pushes off |
| POST | `/v2/push` | yes | Write sealed documents |
| GET | `/v2/changes?since=&limit=` | yes | The account's change feed |
| PUT | `/v2/files/<wire_id>` | yes | Store sealed file bytes |
| GET | `/v2/files/<wire_id>` | yes | Fetch sealed file bytes |
| POST | `/v2/account/rekey` | yes | Deferred: specified under "Deferred", not built in v1. It answers 404 until then. |

Shared shapes:

```json
"kdf":    {"alg": "argon2id", "version": 19, "m_kib": 1048576, "t": 3, "p": 4}
"device": {"id": "d_<32 hex>", "sealed_name": "<55–400 base64url>" | null, "proof": "<43>"}
```

#### GET /v2/health

200 `{"ok": true, "protocol": 2, "signup": "open" | "closed" | "full", "auth": {…},
"events": {…}}`. A setting of `only:<name>` reports `closed`; `full` means the switch is open and
`max_accounts` is reached. The daily cap isn't reported. 503 `misconfigured` without a pepper.

`auth` is the deployment's advertisement, and its absence means "username and password only":

```json
"auth": {"password": true,
         "passkeys": {"rp_id": "sync.kansoapp.app",
                      "origins": ["https://sync.kansoapp.app"],
                      "security_key": true} | null,
         "providers": ["apple", "google", "github"]},
"events": {"account": "/v2/account/events"}
```

`passkeys` is null when `PASSKEY_RP_ID` is unset, an IP address, or plain http other than
`localhost`. `passkeys.security_key` is true when the passkey routes are configured and the
deployment accepts security-key ceremonies; it says nothing about whether the client can run CTAP2
or whether a key is attached, so the Linux app checks its own client and enumerates. `events.account`
advertises the live approval socket; when it is absent the app uses the GET fallbacks only. `providers` lists only the ones whose client IDs and secrets are configured. An
old server omits `auth` or `events`; an old client ignores them. The app asks health each time the
signed-out form opens, and shows a method only when it is advertised there.

#### POST /v2/auth/prelogin

Request `{"username"}`. 200 `{"kdf", "salt": "<22>", "account_id": "a_<32 hex>"}`. Unknown names
get the fake salt, the fake account ID and the default parameters. 400 `invalid_username`. Only the
binding limits it: it reads, and writes nothing.

#### POST /v2/auth/signup

```json
{"username", "account_id": "a_<32 hex, client-generated>", "kdf", "salt": "<22, client-generated>",
 "auth_key": "<43>", "wrapped_key": "<98>", "recovery_auth_key": "<43>", "recovery_wrapped_key": "<98>",
 "key_id": "<16 hex>", "device": {...}}
```

201 `{"account_id", "username", "token": "kanso_<43>", "epoch": "<16 hex>"}`.

Steps:

1. Count `signup-ip`.
2. Validate the body.
3. If an account with this username exists and this is the same request that created it (the same
   account ID, KDF parameters, salt, wrapped keys and key ID, its `auth_hash` equal to SHA-256 of
   `auth_key` and its `recovery_hash` to SHA-256 of `recovery_auth_key`, both hashes always
   compared), this sign-up already happened and its answer was lost: upsert this device's session
   and answer 201 with that account's ID, username and epoch and the new token, whatever the
   sign-up settings say. It doesn't count towards `signups_per_day`. A disabled account gets 403
   `account_disabled`. The auth key alone isn't enough: that would make sign-up a password oracle
   outside the sign-in limits.
4. Otherwise insert the account and the session in one batch. Triggers refuse it when sign-ups are
   closed, the server is full, or the day's limit is reached (403 `signup_closed` with `closed`,
   `full` or `busy`), before any `UNIQUE` check. `UNIQUE` failures give 409 `account_id_taken`
   (checked first) or `username_taken`. The Worker generates the epoch.

Retrying (security re-check 7): after a lost response the client sends the same request again (the
same account ID, salt and keys) and gets 201 for the account it created; it checks that the
answer's `account_id` is its own. A 409 `username_taken` means someone else holds the name.

#### POST /v2/auth/login

Request `{"username", "auth_key", "device": {"id", "proof"}}` (no name yet: the client doesn't hold
the data key). 200:

```json
{"account_id", "username", "token", "wrapped_key", "key_id", "epoch",
 "evicted_device": null | {"id", "sealed_name"}}
```

Steps:

1. Count `login-ip`, and `login-user` unless the device is known: its ID has a session row for the
   account that is active (`revoked_at IS NULL`) and whose `proof_hash` is SHA-256 of `device.proof`.
2. Check the credentials: 401 `invalid_credentials` if wrong.
3. Check that the account is enabled: 403 if disabled.
4. One batch:
   1. If 10 other sessions are active, revoke the one with the oldest `last_seen_at` (`evicted`).
   2. Upsert this device's session as active, with a new token hash, the proof's hash, and
      `sealed_name` kept from its earlier row, if it has one.
   3. Delete the account's revoked rows beyond the 90 most recently revoked, so sign-ins can't
      grow `sessions` without bound.

Because it's one batch, concurrent sign-ins can't exceed 10 active devices. After unwrapping, the
client sends `PUT /v2/account/device` with its sealed name.

#### POST /v2/auth/recover/start

Request `{"username", "recovery_auth_key"}`. 200 `{"account_id", "recovery_wrapped_key",
"key_id"}`. Counts `recover-ip`. 401 `invalid_credentials`; 403 if disabled (after the proof).

#### POST /v2/auth/recover/finish

```json
{"username", "recovery_auth_key", "kdf", "salt", "auth_key", "wrapped_key",
 "recovery_auth_key_new", "recovery_wrapped_key_new", "device": {...}}
```

200 `{"account_id", "username", "token", "epoch"}`.

Counts `recover-ip` and checks `recovery_hash`. One batch then:

1. Replaces the KDF parameters, salt, `auth_hash` and `wrapped_key`, and sets `has_password = 1`.
2. Replaces `recovery_hash` and `recovery_wrapped_key` with the new ones, so the used recovery key
   stops working (H3).
3. Revokes every session (`recovered`) and deletes every passkey and its wrap.
4. Upserts this device's session.

Retrying: if the old recovery hash no longer matches, but the hash of `recovery_auth_key_new` equals
the stored recovery hash and the hash of `auth_key` equals `auth_hash`, the change is already in
place, and the Worker only upserts the session.

Credential updates are compare-and-swap: here, in password changes and in new recovery keys, the
`UPDATE accounts` carries `WHERE` the hash the Worker just checked (`recovery_hash` here,
`auth_hash` in the others), and counts as done only if it changed the row. Of two concurrent
changes one wins; the other gets 401 `invalid_credentials`, or the retry answer when it asked for
the same values.

#### Sign-in routes

All new bodies are JSON and capped at 8 KiB. A `flow_id` is 32 random bytes (43 base64url
characters). A provider flow expires in 10 minutes; a passkey or proof challenge in 5. A flow is consumed once; a second use, or an expired one, is 400 `flow_expired`.
Every route that creates a session answers with the same shape:

```json
{"kind": "session", "account_id", "username", "token", "key_id", "epoch", "has_password",
 "passkey_wrapped_key": "<98>" | null,
 "evicted_device": null | {"id", "sealed_name"}}
```

There is no password wrap here: it comes only from `POST /v2/auth/login`, after the auth key is
checked, so a passkey or provider sign-in can't be used to collect it for offline guessing.
`passkey_wrapped_key` is the credential's wrap, sent only by a passkey sign-in. A provider flow for an identity that isn't
linked answers instead:

```json
{"kind": "new_account", "ticket": "<flow_id>", "suggested_username": "quiet-otter-4821"}
```

**POST /v2/auth/passkey/start** — `{"username"?: string, "device": {...}}`. 200
`{"flow_id", "challenge", "rp_id", "allow_credentials": [{"id", "transports"}],
"user_verification": "required", "timeout": 300000}`. Without a username `allow_credentials` is
empty (a discoverable credential); with one it lists that account's credentials, in the order added.
An unknown username gets an empty list, so the route says no more than sign-up already does.
Counts `passkey-challenge`. 400 `unsupported_method` when the deployment has no `PASSKEY_RP_ID`.

**POST /v2/auth/passkey/finish** — `{"flow_id", "credential_id", "authenticator_data",
"client_data_json", "signature", "user_handle"?, "device": {...}}`. 200 the session shape. The Worker checks
the challenge, the origin, `SHA-256(rp_id)`, the `UP` and `UV` flags, the signature over
`authenticator_data || SHA-256(client_data_json)`, the credential, the account's enabled flag and the
sign count (a stored count of 0 is exempt; a non-zero count that doesn't grow is
`invalid_credentials`). Counts `login-ip`, and `login-user` keyed by the credential's account unless
the device proof matches that account's active session. 401 `invalid_credentials` for every
credential failure, without saying which part failed; 403 `account_disabled`.

**POST /v2/auth/passkey/signup/start** — `{"username", "account_id", "device"}`. 200
`{"flow_id", "challenge", "rp_id", "user": {"id", "name", "displayName"},
"pub_key_cred_params": [-7, -8, -257],
"authenticator_selection": {"residentKey": "required", "userVerification": "required"},
"exclude_credentials": []}`. 400 `invalid_username`, 400 `reserved_username`, 409
`username_taken`, 403 `signup_closed` (with its reason), 400 `unsupported_method`. Counts
`signup-ip`. The username isn't reserved; the finish repeats the check.

**POST /v2/auth/passkey/signup/finish** — `{"flow_id", "credential_id", "attestation_object",
"client_data_json", "transports", "passkey_wrapped_key"?, "username", "account_id",
"recovery_auth_key", "recovery_wrapped_key", "key_id", "device": {...}}`. The username and account
ID are checked against the flow. 201 the session shape.
One batch inserts the account (`has_password = 0`, random KDF parameters and salt, a random auth
hash and a password wrap under a KEK that is zeroized), the passkey, the recovery material, the
identity-free session and the
`signup-ip`/`signups_per_day` counts, exactly as `POST /v2/auth/signup` does. 409 `username_taken`
or `account_id_taken`, 403 `signup_closed`.

**POST /v2/auth/apple/start**, **/google/start**, **/github/start** — for sign-in and sign-up:
`{"device": {...}}` for Apple and GitHub, `{"device": {...}, "redirect_uri"}` for Google. Each
returns the provider's start shape:
`{flow_id, auth_url, expires_at}` (Apple), `{flow_id, auth_url, nonce, state, expires_at}` (Google),
`{flow_id, user_code, verification_uri, interval, expires_in}` (GitHub). These public routes only
sign a device in or create an account; linking and proofs start with the
session route below. Every start binds the flow to one device and proof; a poll or finish from another device
is 400 `bad_request`. 403 `provider_disabled` when the provider isn't configured.

**POST /v2/account/sign-in/identities/<provider>/start** — `{"intent": "link" | "proof",
"proof"?: <proof>, "device": {...}}`. 200 the same start shape as that provider's public route.
`link` requires a password or passkey `proof`; `proof` needs only the session, because the provider
sign-in is the proof. The completion is the provider's usual finish or poll route, and its answer
is `{"kind": "linked", "provider", "created_at"}` or `{"kind": "proof", "provider"}` instead of a
session. 401 `reauth_required` when a link has no usable proof; 409 `identity_taken` when the
subject belongs to another account.

**POST /v2/auth/apple/poll**, **/google/finish**, **/github/poll** — the app's completion:
`{flow_id, device}` for Apple and GitHub, `{flow_id, code, code_verifier, device}` for Google. 200
the session shape, the new-account shape, or `{"status": "pending"}` (Apple and GitHub). GitHub's
poll is refused with 429 `rate_limited` (`limit` `provider-poll`) when the Worker's next poll is
later than the request. 400 `flow_expired`, 400 `nonce_reused`, 403 `provider_denied`, 409
`identity_taken` (a link to an account that already owns the identity), 503 `provider_unavailable`
(the provider's endpoint failed or timed out).

**POST /v2/auth/apple/callback** — the `form_post` route (POST only). It takes
`application/x-www-form-urlencoded`, not JSON, capped at 8 KiB, and is the one public route that
ignores an `Origin` header: a browser form post from Apple carries one. It checks `state`,
exchanges the code and
verifies the token (see "Sign in with Apple"), stores the subject on the flow, and answers an HTML
page that redirects to `kanso://auth/apple?flow=<flow_id>&s=<secret>` and says "Return to Kanso." A bad
`state`, a missing code or a failed verification answers the same page with a short error sentence
and no flow. It never answers JSON and never creates a session.

**POST /v2/auth/social/signup** — `{"ticket", "username", "account_id", "recovery_auth_key",
"recovery_wrapped_key", "key_id", "device": {...}}`. 201 the session shape. The device and proof
must be the flow's. The ticket is a provider flow whose subject was verified and not yet linked;
the batch creates the account with
`has_password = 0`, random KDF parameters and salt, links the identity, inserts the session and
consumes the ticket. 409
`username_taken`, 409 `account_id_taken`, 409 `identity_taken` (linked between the flow and now),
403 `signup_closed`.

**GET /v2/account/sign-in** — 200:

```json
{"username", "has_password",
 "identities": [{"provider", "created_at"}],
 "passkeys": [{"credential_id", "created_at", "last_used_at", "prf", "transports"}],
 "recovery": true}
```

`last_used_at` is to the hour. `prf` is true when the credential carries a wrap. The subject and
the public key aren't returned; the client doesn't need them. `recovery` is always true.

**POST /v2/account/sign-in/proof/start** — `{"credential_id"?: string}`. 200
`{"flow_id", "challenge", "allow_credentials", "timeout": 300000}`. Counts `proof-account`. The
assertion is checked against that credential, with `userVerification: "required"`; the flow is
single use and expires in 5 minutes. A `proof` object (see "Changing a way in needs a fresh proof")
is accepted by every sensitive route below; the assertion is sent with the action.

**POST /v2/account/sign-in/password** — `{"proof", "current_recovery_auth_key"?, "kdf", "salt",
"auth_key", "wrapped_key", "recovery_auth_key", "recovery_wrapped_key"}`. 200
`{"signed_out_devices"}`. Sets the first password, wraps the data key under its KEK, sets
`has_password = 1`, replaces the recovery key in the same compare-and-swap, and revokes every other
session (`password_changed`). Counts `password-account`. A `provider` proof also requires
`current_recovery_auth_key`, checked against `recovery_hash`, so a provider takeover can't set the
first password. 409 `already_has_password` when the account has one; use
`POST /v2/account/password`. 401 `invalid_credentials` when the proof or the recovery key fails.

**POST /v2/account/sign-in/passkeys/start** then **/finish** —
`{"proof"}` (a password or passkey proof) then `{"flow_id", "credential_id",
"attestation_object", "client_data_json", "transports", "passkey_wrapped_key"?}`. 200 the
registration options (as the sign-up start, minus the username), then 201 `{"credential_id",
"prf", "created_at"}`. The finish stores the credential
for the session's account, with the wrap when PRF gave one. Counts `passkey-manage`. 409
`credential_exists` for a credential ID already registered, by this account or another (the latter
is never overwritten).

**POST /v2/account/sign-in/passkeys/<credential_id>/remove** — `{"proof"}`. 204. Deletes the
credential and its wrap. 404 for an unknown credential of this account; 409 `last_method` when the
rule applies and no recovery proof is present. Counts `passkey-manage`.

**POST /v2/account/sign-in/identities/<provider>/unlink** — `{"proof", "recovery_auth_key"?}`.
204. 404 when the provider isn't linked; 409 `last_method` with the rule above. The provider's
`subject` row goes; nothing else changes.

**Approvals.** The requester and approver are both sessions of the account, and the requesting
device can't approve itself.

- **POST /v2/account/sign-in/approvals** — `{"request_id": "<43>", "commit": "<43>"}`. 200
  `{"id", "expires_at", "other_devices"}`. `request_id` is the caller's stable canonical 32-byte
  ID; `commit` is SHA-256 of the requester's ephemeral X25519 public key with its label (see "The
  protocol"). Neither can be changed. Repeating the same `(request_id, account, device, commit)`
  returns the original answer without another insert, cap update, event or `approval-create`
  count; reusing an ID for a different tuple is 409 `approval_request_id_conflict`.
  At most three pending requests per account; a fourth cancels the oldest. A new request counts
  `approval-create`; publishes `approval.created` after the insert commits.
- **GET /v2/account/sign-in/approvals** — 200 `{"approvals": [{"id", "device_id", "commit",
  "epk": "<43>" | null, "created_at", "expires_at", "approver_epk": "<43>" | null,
  "sealed_name": "<...>" | null, "status"}]}`, oldest first, only pending requests for the
  session's account and never the caller's own. `epk` and `sealed_name` appear after the requester
  posts them.
- **GET /v2/account/sign-in/approvals/<id>** — the requester's poll: 200 `{"status":
  "waiting" | "challenged" | "approved" | "cancelled" | "expired", "approver_epk"?: "<43>",
  "approver_device_id"?: "d_<32 hex>", "sealed_name"?: "<...>", "wrapped_key"?: "<98>"}`. Counts
  `approval-poll`; 404 for another account's request or an unknown ID.
- **POST /v2/account/sign-in/approvals/<id>/challenge** — `{"epk": "<43>"}`. 200 `{"ok": true}`.
  Only a different device; stores the approver's ephemeral public key once. The same key again is
  200; another device's or another key is 409 `approval_used`. The first approver can still answer,
  or the request can be cancelled and made again.
- **POST /v2/account/sign-in/approvals/<id>/name** — `{"epk": "<43>", "sealed_name": "<55–400>"}`.
  200 `{"ok": true}`. Only the requesting device, only after a challenge; stores both once, then
  publishes `approval.named`. The Worker checks `epk` against the stored commitment and answers 400
  `bad_request` on a mismatch (a well-behaved server's check; the approver checks again, because it
  doesn't trust the server).
- **POST /v2/account/sign-in/approvals/<id>/approve** — `{"wrapped_key": "<98>"}`. 200
  `{"ok": true}`. Only the device that challenged, after its challenge; the same `wrapped_key`
  again is 200 (a retry); a different one, or a second device, is 409 `approval_used`; 409
  `approval_expired` past the time. Counts `approval-respond`.
- **DELETE /v2/account/sign-in/approvals/<id>** — 204, by the requester, or by the approver once
  it has challenged (a decline); 404 for any other device. Cancels it and publishes
  `approval.cancelled`; the other side's poll says `cancelled`.

#### GET /v2/account/events

Upgrade to a WebSocket with `Authorization: Bearer kanso_<43>`,
`X-Kanso-Device: d_<32 hex>` and `X-Kanso-Device-Proof: <43>`. This is a session route: it rejects an
`Origin`, checks the token, the device ID and SHA-256 of the proof against the active session's
`proof_hash` in constant time, then forwards `{account_id, device_id}` to `ACCOUNT_ROOM` and returns
101. A missing or wrong proof is 401 `invalid_credentials`; no socket is accepted.

`ACCOUNT_ROOM` is one Durable Object per account, keyed by `idFromName(account_id)`, and has no
SQLite or storage. It accepts with `ctx.acceptWebSocket(server, [device_id])`, replaces an older
socket for the same device (close code
4000, "replaced"), and attaches only the account and device to the socket. It does not keep a log.
`setWebSocketAutoResponse(new WebSocketRequestResponsePair("ping", "pong"))` and Cloudflare's
protocol ping handling keep idle heartbeats from waking it. After an event handler returns the
object can hibernate; accepted sockets stay connected and duration billing pauses. When an event
wakes it, `getWebSockets()` returns the hibernated sockets.

Server-to-app frames are text:

```json
{"type":"hello"}
{"type":"approval.created","id":"<id>"}
{"type":"approval.named","id":"<id>"}
{"type":"approval.cancelled","id":"<id>"}
```

No frame carries note content, ciphertext, the device name, ephemeral keys, the code or a wrapped
key; the app fetches the row over HTTPS after an event. The app sends only pings. Unknown event types
are ignored and a malformed client frame closes the socket with code 1008. The Worker publishes after
the D1 commit and never fails the HTTP route if the room cannot be reached.

The DO's socket count is bounded by the account's active devices: one socket per device replaces the
old one. The upgrade consumes one `SESSION_LIMITER` token; frames do not touch D1 or a Worker rate
limit. The app reconnects with bounded exponential backoff and does one approvals `GET` after every
connect, launch, Settings open and foreground. There is no 30-second approval timer.

**Changed existing routes.** `POST /v2/account/recovery-key` and `POST /v2/account/delete` accept a
`proof` in place of `current_auth_key` / `auth_key`, so an account without a password can rotate its
recovery key and delete itself. `POST /v2/account/password` keeps `current_auth_key` and answers 409
`no_password` for a passwordless account; `POST /v2/account/sign-in/password` is its first password.
The delete batch gains the account's `account_identities`, `account_passkeys` and
`device_approvals` rows, and its `auth_flows` rows last, after the account row; the cron deletes any
left over.

**Which session ends close event sockets.** Every one: sign-out and revocation (that device), a
password change, a first password and a recovery (the account's other devices, or all of them),
eviction by the ten-device limit on every route that makes a session (the evicted device), account
deletion (all), and the 180-day expiry (that device, on its next request or in the cron's sweep).
The admin script's `disable` and `delete` close them through the cron within 15 minutes
(`socket_closes`).

#### POST /v2/auth/logout

Revokes this session (`signed_out`) and answers 204, for disabled accounts too. The client treats
401 as done.

#### GET /v2/account

200:

```json
{"account_id", "username", "created_at",
 "usage": {"docs", "doc_bytes", "files", "file_bytes", "writes_today"},
 "quota": {"docs", "doc_bytes", "files", "file_bytes", "writes_per_day"}}
```

#### POST /v2/account/password

Request `{"current_auth_key", "kdf", "salt", "auth_key", "wrapped_key", "recovery_auth_key",
"recovery_wrapped_key"}`. 200 `{"signed_out_devices": n}`. 409 `no_password` for an account without
one; it uses `POST /v2/account/sign-in/password` instead.

The two recovery fields are a new recovery key, in the formats `POST /v2/account/recovery-key`
takes and checked the same way (400 `bad_request` when missing or malformed). The client makes a
fresh recovery secret, derives them as it does for "New recovery key…", and shows the new key once
the change succeeds (security review 2, finding 1).

Counts `password-account`. 401 `invalid_credentials` on a wrong current key. On success one
compare-and-swap `UPDATE accounts … WHERE auth_hash = <the checked hash>` replaces the password
material, `recovery_hash` (SHA-256 of `recovery_auth_key`) and `recovery_wrapped_key` together. A
password change therefore always ends every older recovery key, including one set by someone who
once knew the password. Every other active session is revoked (`password_changed`).

Retrying: if the current key fails but `auth_hash`, `wrapped_key`, `recovery_hash` and
`recovery_wrapped_key` already equal the request's new values, the answer is 200 with 0.

#### POST /v2/account/recovery-key

Request `{"current_auth_key", "recovery_auth_key", "recovery_wrapped_key"}`, or the same with a
`proof` in place of `current_auth_key`, which is how a passwordless account replaces its key. A
`provider` proof also takes `current_recovery_auth_key`, checked against `recovery_hash`. 200 `{}`.
Counts `password-account` or `proof-account`, then replaces the recovery hash and wrap. Sessions
stay.

Retrying: if the current key is right and the stored values already equal the new ones, the answer
is 200.

#### POST /v2/account/delete

Request `{"proof", "username"}`, or `{"auth_key", "username"}` for a password account. A
`provider` proof also takes `current_recovery_auth_key`, checked against `recovery_hash`.
200 `{"deleted": true}`.

Counts `password-account` for a password and `proof-account` for a proof, and checks it. Then
`username_mismatch` unless the username normalizes to the account's. One batch then:

1. Delete the account's rows from `account_docs`.
2. Delete its rows from `account_files`.
3. Delete its rows from `sessions`.
4. Delete its rows from `account_identities`, `account_passkeys`, `device_approvals` and
   `auth_flows`.
5. Delete its account-keyed `limits` rows.
6. Delete the `accounts` row.
7. Insert `purges('accounts/<id>/')`.

Then up to 20 purge rounds inline. Retrying: a retry after a lost response gets 401 `unknown` (the
session row is gone), which the client treats as success. Any other 401 reason means the session
ended before the deletion, so the client reports that instead.

#### PUT /v2/account/device, GET /v2/account/devices, DELETE /v2/account/devices/<device_id>

- **PUT** `{"sealed_name"}` sets the current session's name and answers 204.
- **GET** `{"devices": [{"id", "sealed_name", "created_at", "last_seen_at", "current"}]}` lists
  active sessions only, most recently seen first.
- **DELETE** runs `UPDATE sessions SET revoked_at = coalesce(revoked_at, ?3), revoked_reason =
  'revoked' WHERE account_id = ?1 AND device_id = ?2`. A row already signed out, for any reason, is
  marked `revoked` too and keeps its time, so the owner can always remove a Mac they no longer
  trust. It answers 204, or 404 when the account has no row for that device (an unknown ID, or
  another account's device), or 400 for a malformed ID. Deleting the current device is a logout
  (reason `signed_out`).

#### PUT and DELETE /v2/account/devices/<device_id>/push

Push notifications are silent wake-ups: a device that has a token is nudged when another device of
the account (or a member of a shared note) changes something, then pulls over HTTPS as usual. The
push says nothing about what changed.

- **PUT** `{"platform", "token", "environment"}` sets the caller's own token and answers 204.
  `platform` is `apns` or `fcm`; `environment` is `production` or `development` (`fcm` has only
  `production`). An APNs token is lowercase hex, an even 32–200 characters; an FCM token is 100–4096
  characters of `A-Z a-z 0-9 _ : -`. The same shapes are `CHECK`s on `sessions`. Registering again
  replaces the token. A token another session holds is cleared from it in the same batch, so a
  token is on one session at most.
- **DELETE** clears the token and answers 204 whether or not one was set.
- 400 for a bad body or a malformed device ID; 404 for any device but the session's own (another
  device of the account, another account's, an unknown one), as `DELETE /v2/account/devices/<id>`
  answers a device that isn't the caller's; 401 for a session that has ended.

**What wakes a device.** A `POST /v2/push` with at least one `ok` result wakes the account's other
active devices that have a token; a shared note's update or comment (`POST
/v2/collab/shares/<id>/log`) wakes the active devices of every member account of the share. Never
the sending device, never a revoked session, never a disabled account's. A push that is all
conflicts wakes nobody. Waking costs the route one D1 `UPDATE` and one Durable Object call, and a
failure of either is logged (`push`, `wake_failed` or `schedule_failed`) and never fails the route.

**Coalescing.** A device is sent at most one push per 30 seconds. The `UPDATE` sets `push_due_at =
coalesce(push_due_at, max(now, push_last_sent_at + 30000))` on the matching rows and returns the
earliest; the route tells the account's `AccountRoom` (`schedule`), which arms its alarm for that
time unless an earlier one is set. The alarm claims the rows whose due time has come and whose last
push is 30 seconds old (`UPDATE ... SET push_last_sent_at = now, push_due_at = NULL ... RETURNING`),
sends to each (four at a time), clears a token the provider reports gone (only if the row still
holds the token that was sent), and arms itself again for the earliest row still waiting. A burst of
changes therefore ends in one trailing push, never in none. A send that fails is logged
(`push`, `refused` or `send_failed`, with the platform and status) and not retried until the next
change.

**Cleared tokens.** A trigger (migration `0011`) clears the token, platform, environment, update
time and due time when a session's `revoked_at` goes from `NULL` to a time (sign-out, revoke,
expiry, eviction, password change, recovery, rekey) and when its `token_hash` changes (the device
signed in again; it registers again). `push_last_sent_at` stays, so a device that signs in again is
still owed its 30 seconds. Account deletion deletes the rows.

**Providers.**

- *APNs* (token auth): an ES256 JWT `{alg, kid: APNS_KEY_ID}` with `{iss: APNS_TEAM_ID, iat}`,
  signed with the `.p8` key in `APNS_KEY_P8` and cached in the Worker for 40 minutes. `POST
  https://api.push.apple.com/3/device/<token>` (`api.sandbox.push.apple.com` for `development`)
  with `authorization: bearer <jwt>`, `apns-push-type: background`, `apns-priority: 5`,
  `apns-topic: <APNS_TOPIC>`, `apns-expiration: <now + 1 hour, in seconds>`, `apns-collapse-id:
  sync`, and the body `{"aps":{"content-available":1}}`. A token is gone on 410, or on 400
  `BadDeviceToken`. A 403 `ExpiredProviderToken` drops the cached JWT.
- *FCM* (HTTP v1): `FCM_SERVICE_ACCOUNT` is the service account's JSON key. An RS256 assertion
  (`scope` `https://www.googleapis.com/auth/firebase.messaging`, `aud` the key's `token_uri`,
  one hour) is exchanged for an access token, cached until five minutes before it expires. `POST
  https://fcm.googleapis.com/v1/projects/<project_id>/messages:send` with the body `{"message":
  {"token": "<token>", "android": {"priority": "HIGH", "ttl": "3600s", "collapse_key": "sync"}}}`
  and no `data` or `notification`. A token is gone when `error.details[]` has `errorCode`
  `UNREGISTERED`. A 401 drops the cached access token.
- A platform whose secrets are missing is skipped with one log line (`push`, `unconfigured`, the
  platform) per alarm; its tokens are still stored.

#### POST /v2/push

Request `{"docs": [{"id", "rev", "deleted", "sealed", "base_rev"}]}`. All five fields are required:
`deleted` is a boolean, and `base_rev` is a string or null. Patterns:

- `id`: `^(note|book|tag|file):[A-Za-z0-9_-]{22}$`
- `rev`, `base_rev`: `^[0-9a-f]{32}$`
- `sealed`: 55–1,900,000 characters of base64url. A `CHECK` enforces the alphabet, so the Worker
  doesn't scan it.

200 `{"results": [...]}`, one per document, in order:

| Result | Meaning |
|---|---|
| `{"id", "status": "ok", "seq"}` | Stored under `seq`. |
| `{"id", "status": "conflict", "current": {"id", "rev", "deleted", "sealed"}}` | `base_rev` isn't the current revision; here is the server's version. |
| `{"id", "status": "conflict", "current": null}` | The client sent a `base_rev`, but the server has no such document: it lost it. |
| `{"id", "status": "behind"}` | A conflict whose current version didn't fit the response's 8 MiB budget of sealed text. The client pulls before retrying. |

Semantics are v1's, scoped to the account: a document is accepted iff its `base_rev` is the
account's current revision of it, or the account has none and `base_rev` is null. Documents apply
in order. The client sends at most 200 documents and 4 MiB of body per push. Steps:

1. Validate the body (400, or 413 for size). A refused body counts 1 against `writes-account`
   first, and the answer is 429 instead if that is over the day's budget. An empty `docs` answers
   200 `{"results": []}` and counts nothing. Otherwise count `writes-account` by the number of
   documents. Over the day's budget → 429. The sealed alphabet is checked only by the `CHECK`,
   after counting.
2. One batch, three statements per document, as in v1 and b4e2733, with `ROW_COST = 512`:

```sql
-- ?1 account, ?2 wire id, ?3 base_rev, ?4 rev, ?5 deleted (0/1), ?6 sealed, ?7 length(sealed), ?8 now
-- Stored lengths are read with octet_length(), from the record header; length() on TEXT reads
-- the whole value. Sealed text is base64url, so bytes equal characters.
ACCEPT  = ((?3 IS NULL AND NOT EXISTS (SELECT 1 FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2))
           OR EXISTS (SELECT 1 FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2 AND d.rev = ?3))
BUMP    = UPDATE accounts SET
            last_seq = last_seq + 1,
            docs = docs + (1 - ?5)
                 - coalesce((SELECT 1 - d.deleted FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2), 0),
            doc_bytes = doc_bytes + ?7 + 512
                 - coalesce((SELECT octet_length(d.sealed) + 512 FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2), 0)
          WHERE accounts.id = ?1 AND ACCEPT RETURNING last_seq AS value
CURRENT = SELECT id, rev, deleted, sealed FROM account_docs WHERE account_id = ?1 AND id = ?2 AND NOT ACCEPT
WRITE   = INSERT INTO account_docs (account_id, id, rev, seq, deleted, sealed, updated_at)
            SELECT ?1, ?2, ?4, last_seq, ?5, ?6, ?8 FROM accounts WHERE accounts.id = ?1 AND ACCEPT
          ON CONFLICT (account_id, id) DO UPDATE SET rev = excluded.rev, seq = excluded.seq,
            deleted = excluded.deleted, sealed = excluded.sealed, updated_at = excluded.updated_at
```

   The quota triggers on `accounts` refuse any `BUMP` that grows usage past a quota or cap, and the
   whole batch rolls back. The Worker recognises the refusal by the `quota_exceeded:<kind>`
   (or, at sign-up, `signup_closed:<reason>`) text in D1's error message. It then reads the
   account's usage and answers 413
   `quota_exceeded` with `used` and `limit`. Shrinking writes always pass, so an account over quota
   can still delete. There is no check-then-act anywhere (H1). Every stored write (new or updated)
   also counts towards `global_writes_per_day`; past it, for an account above its fair share (see
   "Rate limits"), a trigger raises `rate_limited:writes-global`, the batch rolls back, and the
   answer is 429 (security re-check 2).
3. **File tombstones.** For accepted file tombstones, delete their R2 objects (one call). Then one
   batch: subtract their `account_files` sizes (plus 1,024 each) and count from the account, and
   delete those rows.

Retrying: a lost response is recovered through the feed. Right before each push the client records
each document's `sent_rev` in a guarded write. The next sync pulls the document back. If the echo is
the Mac's current revision, the store marks it synced. If the note was edited since, the echo equals
`sent_rev`: it becomes `synced_rev`, the newer edit stays pending on top of it, and nothing counts as a
conflict, whatever `updatedAt` says.

#### GET /v2/changes?since=&limit=

200:

```json
{"docs": [{"id", "rev", "deleted", "sealed", "seq"}], "last_seq": n, "more": bool,
 "head_seq": n, "epoch": "<16 hex>", "key_id": "<16 hex>"}
```

- **Parameters.** `limit` defaults to 100 and must be 1–500; `since` defaults to 0 and must not be
  negative. Anything else, a non-integer included, is 400.
- **Two reads keep the page small.** First the sequence numbers and lengths, as
  `octet_length(sealed)`, which SQLite answers without reading the text (`length()` would read
  every candidate row in full, overflow pages included; security review 2, finding 2). Then the
  rows that fit in 8 MiB of sealed text, always at least one.
- **`more`** is true when that prefix was cut, or when the first read returned `limit` rows.
- **`last_seq`** is the last document's seq, or `since` when the page is empty.
- **`head_seq`** is the account's `last_seq`.
- **Splicing.** Each string goes into the response through `JSON.stringify`.

#### PUT and GET /v2/files/<wire_id>

PUT takes the sealed bytes with `Content-Type: application/octet-stream`, `Content-Length` and
`x-kanso-sha256: <64 lowercase hex>`. The client always sends the hash; anything but 64 lowercase
hex is 400. Steps:

1. Check the path's wire ID and the headers (400 or 415). Then count `writes-account` by 1.
2. Read the body: under 41 bytes is 400; over 10,489,897 bytes is 413.
3. Hash it with `crypto.subtle.digest("SHA-256")` and compare: 400 `checksum_mismatch` if different.
4. One batch:
   1. `SELECT size` of the row this replaces.
   2. Update `accounts`: `files` grows by `NOT EXISTS(row)`, and `file_bytes` by `size + 1024 −
      coalesce(old size + 1024, 0)`, computed in SQL. The statement must affect the account row;
      if it doesn't, the account is gone, and the answer is 401.
   3. Upsert `account_files` with `uploaded_at` = now. A trigger counts the size towards
      `global_upload_bytes_per_day`, replacements included, and refuses past it, for an account
      above its fair share, with `rate_limited:uploads-global` (429; security re-check 2).

   Triggers refuse growth past quotas with 413.
5. `FILES.put(key, body, {sha256})`. If R2 fails, a compensating batch restores the old row and
   counters, if the row still holds this upload's size, and the answer is 500. Restoring a larger
   old size sets `recounted_at`, so the quota triggers let the correction through.
6. 200 `{"ok": true, "size": n}`.

Two concurrent uploads of the same ID can leave the counter off by their size difference. `recount`
repairs it. The engine never uploads one file twice at once.

GET answers 200 with the bytes (`application/octet-stream`, `Content-Length`) only when the account
has an `account_files` row for the ID, and 404 otherwise, even if R2 still holds an object (security
re-check 8). The cron reconciles the two (below).

## Storage

### Migration `cloud/migrations/0002_accounts.sql`

This migration only adds things. It never reads or changes `docs` or `counter`. Quotas, caps and
sign-up rules live in triggers, so D1 enforces them atomically. Wrangler's statement splitter tracks
`CREATE TRIGGER … END` the way SQLite does (checked in `wrangler 4.151.1`, `splitSqlQuery`), and
step 1 applies this file with both `wrangler d1 migrations apply --local` and vitest's
`applyD1Migrations`. D1 enforces foreign keys, so deletions run children first.

```sql
-- Accounts with end-to-end encrypted libraries (docs/ACCOUNTS.md). Additive only: the
-- single-user tables `docs` and `counter` stay as they are until the owner runs
-- `npm run -s admin -- purge-legacy` with a verified manifest. Quotas, caps and the
-- sign-up switch are enforced here, so concurrent requests can't race past them.
-- Every setting fails closed: a missing value refuses (coalesce to 0 or 'closed'), and
-- rows of `settings` and `daily` can't be deleted or renamed.

CREATE TABLE settings (
    name TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    CHECK ((name = 'signup' AND (value IN ('open', 'closed')
                                 OR (value GLOB 'only:*' AND length(value) BETWEEN 8 AND 37)))
           OR (name != 'signup' AND length(value) BETWEEN 1 AND 15 AND value NOT GLOB '*[^0-9]*'))
);
INSERT INTO settings (name, value) VALUES
    ('signup', 'closed'),
    ('max_accounts', '50'),
    ('signups_per_day', '5'),
    ('quota_docs', '50000'),
    ('quota_doc_bytes', '104857600'),
    ('quota_files', '10000'),
    ('quota_file_bytes', '524288000'),
    ('writes_per_day', '20000'),
    ('upload_bytes_per_day', '1073741824'),
    ('global_doc_bytes', '6000000000'),
    ('global_file_bytes', '50000000000'),
    ('global_writes_per_day', '300000'),
    ('global_upload_bytes_per_day', '5000000000');

CREATE TRIGGER settings_keep BEFORE DELETE ON settings
BEGIN SELECT RAISE(ABORT, 'settings_required'); END;

CREATE TRIGGER settings_names_fixed BEFORE UPDATE OF name ON settings
BEGIN SELECT RAISE(ABORT, 'settings_required'); END;

-- The numeric settings as one row, for the triggers. A missing setting reads as NULL,
-- which every trigger turns into a refusal.
CREATE VIEW defaults AS SELECT
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'max_accounts') AS max_accounts,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'signups_per_day') AS signups_per_day,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'quota_docs') AS docs,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'quota_doc_bytes') AS doc_bytes,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'quota_files') AS files,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'quota_file_bytes') AS file_bytes,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'global_doc_bytes') AS global_doc_bytes,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'global_file_bytes') AS global_file_bytes,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'global_writes_per_day') AS global_writes_per_day,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'global_upload_bytes_per_day')
        AS global_upload_bytes_per_day;

-- Server-wide counters for one UTC day (`day` = floor(ms / 86400000)): accounts created,
-- documents stored and file bytes uploaded. Only triggers write them, and deleting an
-- account doesn't lower them. A write dated before `day` counts towards `day`.
CREATE TABLE daily (
    name TEXT PRIMARY KEY NOT NULL CHECK (name IN ('signups', 'writes', 'upload_bytes')),
    day INTEGER NOT NULL CHECK (day >= 0),
    count INTEGER NOT NULL CHECK (count >= 0)
);
INSERT INTO daily (name, day, count) VALUES ('signups', 0, 0), ('writes', 0, 0), ('upload_bytes', 0, 0);

CREATE TRIGGER daily_keep BEFORE DELETE ON daily
BEGIN SELECT RAISE(ABORT, 'daily_required'); END;

CREATE TRIGGER daily_names_fixed BEFORE UPDATE OF name ON daily
BEGIN SELECT RAISE(ABORT, 'daily_required'); END;

-- `docs` counts live documents; `doc_bytes` is the sum of length(sealed) + 512 over all of
-- the account's rows, tombstones included; `files` and `file_bytes` count account_files rows
-- and their size + 1024. NULL quotas use `defaults`; a NULL `quota_writes_per_day` uses the
-- `writes_per_day` setting (the Worker reads it; no trigger does). The quota triggers exempt
-- an update that changes `recounted_at`, which only two corrections do: the admin recount, and
-- an upload's compensation when R2 fails and the row's old, larger size comes back (sync.ts).
-- The cron's size reconciliation doesn't, so a growing correction meets the quotas.
CREATE TABLE accounts (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (length(id) = 34 AND substr(id, 1, 2) = 'a_' AND substr(id, 3) NOT GLOB '*[^0-9a-f]*'),
    username TEXT NOT NULL UNIQUE
        CHECK (length(username) BETWEEN 3 AND 32 AND username NOT GLOB '*[^a-z0-9_-]*'
               AND username GLOB '[a-z0-9]*' AND username GLOB '*[a-z0-9]'),
    kdf_m_kib INTEGER NOT NULL CHECK (kdf_m_kib BETWEEN 1048576 AND 2097152),
    kdf_t INTEGER NOT NULL CHECK (kdf_t BETWEEN 3 AND 8),
    kdf_p INTEGER NOT NULL CHECK (kdf_p BETWEEN 1 AND 8),
    salt TEXT NOT NULL CHECK (length(salt) = 22 AND salt NOT GLOB '*[^A-Za-z0-9_-]*'),
    auth_hash TEXT NOT NULL CHECK (length(auth_hash) = 64 AND auth_hash NOT GLOB '*[^0-9a-f]*'),
    wrapped_key TEXT NOT NULL CHECK (length(wrapped_key) = 98 AND wrapped_key NOT GLOB '*[^A-Za-z0-9_-]*'),
    recovery_hash TEXT NOT NULL CHECK (length(recovery_hash) = 64 AND recovery_hash NOT GLOB '*[^0-9a-f]*'),
    recovery_wrapped_key TEXT NOT NULL
        CHECK (length(recovery_wrapped_key) = 98 AND recovery_wrapped_key NOT GLOB '*[^A-Za-z0-9_-]*'),
    key_id TEXT NOT NULL CHECK (length(key_id) = 16 AND key_id NOT GLOB '*[^0-9a-f]*'),
    epoch TEXT NOT NULL CHECK (length(epoch) = 16 AND epoch NOT GLOB '*[^0-9a-f]*'),
    last_seq INTEGER NOT NULL DEFAULT 0 CHECK (last_seq >= 0),
    docs INTEGER NOT NULL DEFAULT 0 CHECK (docs >= 0),
    doc_bytes INTEGER NOT NULL DEFAULT 0 CHECK (doc_bytes >= 0),
    files INTEGER NOT NULL DEFAULT 0 CHECK (files >= 0),
    file_bytes INTEGER NOT NULL DEFAULT 0 CHECK (file_bytes >= 0),
    quota_docs INTEGER CHECK (quota_docs > 0),
    quota_doc_bytes INTEGER CHECK (quota_doc_bytes > 0),
    quota_files INTEGER CHECK (quota_files > 0),
    quota_file_bytes INTEGER CHECK (quota_file_bytes > 0),
    quota_writes_per_day INTEGER CHECK (quota_writes_per_day > 0),
    recounted_at INTEGER,
    created_at INTEGER NOT NULL,
    disabled_at INTEGER
);
CREATE INDEX accounts_created ON accounts (created_at);

-- Sign-ups: the switch (or a reservation for one name), then max_accounts, then the daily
-- cap, counted in `daily`. The WHEN clauses exclude each other, so one refusal names one
-- reason.
CREATE TRIGGER signup_closed BEFORE INSERT ON accounts
WHEN coalesce((SELECT value FROM settings WHERE name = 'signup'), 'closed') NOT IN ('open', 'only:' || NEW.username)
BEGIN SELECT RAISE(ABORT, 'signup_closed:closed'); END;

CREATE TRIGGER signup_full BEFORE INSERT ON accounts
WHEN coalesce((SELECT value FROM settings WHERE name = 'signup'), 'closed') IN ('open', 'only:' || NEW.username)
 AND (SELECT count(*) FROM accounts) >= coalesce((SELECT max_accounts FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'signup_closed:full'); END;

CREATE TRIGGER signup_busy BEFORE INSERT ON accounts
WHEN coalesce((SELECT value FROM settings WHERE name = 'signup'), 'closed') IN ('open', 'only:' || NEW.username)
 AND (SELECT count(*) FROM accounts) < coalesce((SELECT max_accounts FROM defaults), 0)
 AND coalesce((SELECT CASE WHEN day >= NEW.created_at / 86400000 THEN count ELSE 0 END
               FROM daily WHERE name = 'signups'), 9223372036854775807)
     >= coalesce((SELECT signups_per_day FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'signup_closed:busy'); END;

CREATE TRIGGER signup_count AFTER INSERT ON accounts
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.created_at / 86400000 THEN count + 1 ELSE 1 END,
                     day = max(day, NEW.created_at / 86400000)
    WHERE name = 'signups';
END;

-- Quotas only stop growth. One refusal names one quota, in the order docs, doc_bytes,
-- server (global document bytes); files, file_bytes, server (global file bytes).
CREATE TRIGGER quota_docs BEFORE UPDATE OF docs ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.docs > OLD.docs
 AND NEW.docs > coalesce(NEW.quota_docs, (SELECT docs FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:docs'); END;

CREATE TRIGGER quota_doc_bytes BEFORE UPDATE OF doc_bytes ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.doc_bytes > OLD.doc_bytes
 AND NOT (NEW.docs > OLD.docs AND NEW.docs > coalesce(NEW.quota_docs, (SELECT docs FROM defaults), 0))
 AND NEW.doc_bytes > coalesce(NEW.quota_doc_bytes, (SELECT doc_bytes FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:doc_bytes'); END;

CREATE TRIGGER quota_server_docs BEFORE UPDATE OF doc_bytes ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.doc_bytes > OLD.doc_bytes
 AND NOT (NEW.docs > OLD.docs AND NEW.docs > coalesce(NEW.quota_docs, (SELECT docs FROM defaults), 0))
 AND NEW.doc_bytes <= coalesce(NEW.quota_doc_bytes, (SELECT doc_bytes FROM defaults), 0)
 AND (SELECT sum(doc_bytes) FROM accounts) - OLD.doc_bytes + NEW.doc_bytes
     > coalesce((SELECT global_doc_bytes FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:server'); END;

CREATE TRIGGER quota_files BEFORE UPDATE OF files ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.files > OLD.files
 AND NEW.files > coalesce(NEW.quota_files, (SELECT files FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:files'); END;

CREATE TRIGGER quota_file_bytes BEFORE UPDATE OF file_bytes ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.file_bytes > OLD.file_bytes
 AND NOT (NEW.files > OLD.files AND NEW.files > coalesce(NEW.quota_files, (SELECT files FROM defaults), 0))
 AND NEW.file_bytes > coalesce(NEW.quota_file_bytes, (SELECT file_bytes FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:file_bytes'); END;

CREATE TRIGGER quota_server_files BEFORE UPDATE OF file_bytes ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.file_bytes > OLD.file_bytes
 AND NOT (NEW.files > OLD.files AND NEW.files > coalesce(NEW.quota_files, (SELECT files FROM defaults), 0))
 AND NEW.file_bytes <= coalesce(NEW.quota_file_bytes, (SELECT file_bytes FROM defaults), 0)
 AND (SELECT sum(file_bytes) FROM accounts) - OLD.file_bytes + NEW.file_bytes
     > coalesce((SELECT global_file_bytes FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:server'); END;

-- One row per device per account. Revoked rows stay 90 days (to explain a 401 and to keep a
-- returning device's name); active rows expire after 180 days unseen. `proof_hash` is SHA-256
-- of the device's proof, which only that device can derive; only an active row's exempts a
-- sign-in from `login-user`.
CREATE TABLE sessions (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    device_id TEXT NOT NULL
        CHECK (length(device_id) = 34 AND substr(device_id, 1, 2) = 'd_' AND substr(device_id, 3) NOT GLOB '*[^0-9a-f]*'),
    token_hash TEXT NOT NULL UNIQUE CHECK (length(token_hash) = 64 AND token_hash NOT GLOB '*[^0-9a-f]*'),
    proof_hash TEXT NOT NULL CHECK (length(proof_hash) = 64 AND proof_hash NOT GLOB '*[^0-9a-f]*'),
    sealed_name TEXT
        CHECK (sealed_name IS NULL OR (length(sealed_name) BETWEEN 55 AND 400 AND sealed_name NOT GLOB '*[^A-Za-z0-9_-]*')),
    created_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    revoked_at INTEGER,
    revoked_reason TEXT CHECK (revoked_reason IN
        ('signed_out', 'revoked', 'password_changed', 'recovered', 'evicted', 'expired', 'rekeyed')),
    CHECK ((revoked_at IS NULL) = (revoked_reason IS NULL)),
    PRIMARY KEY (account_id, device_id)
);
CREATE INDEX sessions_by_account ON sessions (account_id, revoked_at, last_seen_at);

-- Sealed documents, keyed by (account, wire id). Only base64url can be stored.
CREATE TABLE account_docs (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    id TEXT NOT NULL CHECK (
        ((substr(id, 1, 5) IN ('note:', 'book:', 'file:')) AND length(id) = 27
             AND substr(id, 6) NOT GLOB '*[^A-Za-z0-9_-]*')
        OR (substr(id, 1, 4) = 'tag:' AND length(id) = 26 AND substr(id, 5) NOT GLOB '*[^A-Za-z0-9_-]*')),
    rev TEXT NOT NULL CHECK (length(rev) = 32 AND rev NOT GLOB '*[^0-9a-f]*'),
    seq INTEGER NOT NULL CHECK (seq > 0),
    deleted INTEGER NOT NULL CHECK (deleted IN (0, 1)),
    sealed TEXT NOT NULL CHECK (length(sealed) BETWEEN 55 AND 1900000 AND sealed NOT GLOB '*[^A-Za-z0-9_-]*'),
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, id),
    UNIQUE (account_id, seq)
);

-- Server-wide document writes per UTC day (global_writes_per_day), shared fairly: once the
-- day's count reaches the cap, a write is refused only for an account above its fair share
-- (global_writes_per_day / max_accounts), judged by its own `writes-account` count today, which
-- the Worker takes before the batch (this push included). An account at or below its fair share
-- is never refused here, so a few accounts at their own budgets can't pause everyone; a day's
-- total stays under about twice the cap. Fails closed: a missing setting, max_accounts 0, or no
-- `writes-account` row for today counts as above the fair share. The push's upsert fires BEFORE
-- INSERT on both of its paths, then AFTER INSERT or AFTER UPDATE, so every stored write is
-- checked and counted once.
CREATE TRIGGER writes_global BEFORE INSERT ON account_docs
WHEN coalesce((SELECT CASE WHEN day >= NEW.updated_at / 86400000 THEN count ELSE 0 END
               FROM daily WHERE name = 'writes'), 9223372036854775807)
     >= coalesce((SELECT global_writes_per_day FROM defaults), 0)
 AND coalesce((SELECT CASE WHEN window_start = NEW.updated_at / 86400000 * 86400000 THEN count END
               FROM limits WHERE key = 'writes-account:' || NEW.account_id), 9223372036854775807)
     > coalesce((SELECT global_writes_per_day / max_accounts FROM defaults WHERE max_accounts > 0), -1)
BEGIN SELECT RAISE(ABORT, 'rate_limited:writes-global'); END;

CREATE TRIGGER writes_count_insert AFTER INSERT ON account_docs
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.updated_at / 86400000 THEN count + 1 ELSE 1 END,
                     day = max(day, NEW.updated_at / 86400000)
    WHERE name = 'writes';
END;

CREATE TRIGGER writes_count_update AFTER UPDATE OF rev ON account_docs
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.updated_at / 86400000 THEN count + 1 ELSE 1 END,
                     day = max(day, NEW.updated_at / 86400000)
    WHERE name = 'writes';
END;

-- Every stored R2 object and its size, for quotas and for deleting without listing.
-- `uploaded_at` is the time of the upload that stored this size; corrections (a failed R2
-- put's compensation, the cron's reconciliation) change `size` but never `uploaded_at`.
CREATE TABLE account_files (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    id TEXT NOT NULL CHECK (substr(id, 1, 5) = 'file:' AND length(id) = 27 AND substr(id, 6) NOT GLOB '*[^A-Za-z0-9_-]*'),
    size INTEGER NOT NULL CHECK (size BETWEEN 41 AND 10489897),
    uploaded_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, id)
);

-- Server-wide upload bytes per UTC day (global_upload_bytes_per_day), weighted by size and
-- counted on every upload, replacements included. Shared fairly as document writes are: past
-- the cap, only an account whose own `uploads-account` count today (this upload included) is
-- above global_upload_bytes_per_day / max_accounts is refused, and the same things fail closed.
CREATE TRIGGER uploads_global BEFORE INSERT ON account_files
WHEN coalesce((SELECT CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count ELSE 0 END
               FROM daily WHERE name = 'upload_bytes'), 9223372036854775807)
     > coalesce((SELECT global_upload_bytes_per_day FROM defaults), 0) - NEW.size
 AND coalesce((SELECT CASE WHEN window_start = NEW.uploaded_at / 86400000 * 86400000 THEN count END
               FROM limits WHERE key = 'uploads-account:' || NEW.account_id), 9223372036854775807)
     > coalesce((SELECT global_upload_bytes_per_day / max_accounts FROM defaults WHERE max_accounts > 0), -1)
BEGIN SELECT RAISE(ABORT, 'rate_limited:uploads-global'); END;

CREATE TRIGGER uploads_count_insert AFTER INSERT ON account_files
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count + NEW.size ELSE NEW.size END,
                     day = max(day, NEW.uploaded_at / 86400000)
    WHERE name = 'upload_bytes';
END;

CREATE TRIGGER uploads_count_update AFTER UPDATE OF uploaded_at ON account_files
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count + NEW.size ELSE NEW.size END,
                     day = max(day, NEW.uploaded_at / 86400000)
    WHERE name = 'upload_bytes';
END;

-- Fixed-window counters. Keys hold HMACs of IPs and usernames, never the values.
CREATE TABLE limits (
    key TEXT PRIMARY KEY NOT NULL CHECK (length(key) <= 80),
    window_start INTEGER NOT NULL,
    count INTEGER NOT NULL CHECK (count >= 0)
);

-- R2 prefixes still to empty (deleted accounts, legacy files/).
CREATE TABLE purges (
    prefix TEXT PRIMARY KEY NOT NULL
        CHECK (prefix = 'files/' OR (prefix GLOB 'accounts/a_*/' AND length(prefix) = 44)),
    requested_at INTEGER NOT NULL
);
```

Before applying it remotely, the deployed database must have only `0001_init` applied. b4e2733's
`0002_users` must never have run there.

### Migration `cloud/migrations/0006_sign_in.sql`

This migration only adds: one column on `accounts` and four tables. It touches no existing row,
constraint, index or trigger. It must run before the Worker that reads it.

```sql
-- Passkeys and third-party sign-in (docs/ACCOUNTS.md › Sign-in methods). Additive:
-- 0002's rows, triggers and quotas are untouched. `has_password` defaults to 1, so every
-- existing account is a password account. A provider row holds the stable subject only;
-- emails, names, profile data and provider tokens are never stored.

ALTER TABLE accounts ADD COLUMN has_password INTEGER NOT NULL DEFAULT 1;

CREATE TABLE account_identities (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    provider TEXT NOT NULL
        CHECK (length(provider) BETWEEN 2 AND 32 AND provider GLOB '[a-z]*'
               AND provider NOT GLOB '*[^a-z0-9_]*'),
    subject TEXT NOT NULL
        CHECK (length(subject) BETWEEN 1 AND 255 AND subject NOT GLOB '*[^ -~]*'),
    created_at INTEGER NOT NULL,
    PRIMARY KEY (provider, subject)
);
CREATE INDEX account_identities_by_account ON account_identities (account_id, provider);

-- One row per passkey. `wrapped_key` is the data key under the PRF-derived KEK (98 base64url
-- characters), or NULL when the credential registered without PRF. `aaguid` is for display.
CREATE TABLE account_passkeys (
    credential_id TEXT PRIMARY KEY NOT NULL
        CHECK (length(credential_id) BETWEEN 22 AND 1364
               AND credential_id NOT GLOB '*[^A-Za-z0-9_-]*'),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    public_key TEXT NOT NULL
        CHECK (length(public_key) BETWEEN 20 AND 2048 AND public_key NOT GLOB '*[^A-Za-z0-9_-]*'),
    alg INTEGER NOT NULL CHECK (alg IN (-8, -7, -257)),
    sign_count INTEGER NOT NULL CHECK (sign_count >= 0),
    transports TEXT NOT NULL DEFAULT ''
        CHECK (length(transports) <= 64 AND transports NOT GLOB '*[^a-z ]*'),
    wrapped_key TEXT CHECK (wrapped_key IS NULL
        OR (length(wrapped_key) = 98 AND wrapped_key NOT GLOB '*[^A-Za-z0-9_-]*')),
    aaguid TEXT CHECK (aaguid IS NULL
        OR (length(aaguid) = 32 AND aaguid NOT GLOB '*[^0-9a-f]*')),
    created_at INTEGER NOT NULL,
    last_used_at INTEGER,
    CHECK (last_used_at IS NULL OR last_used_at >= created_at)
);
CREATE INDEX account_passkeys_by_account ON account_passkeys (account_id, created_at);

-- One row per provider or passkey ceremony. `device_code` is GitHub's (sealed under the pepper's
-- flow key, because it can mint a token); `nonce_hash` and `state_hash` are SHA-256 hex. A row is
-- consumed once and deleted by the cron.
CREATE TABLE auth_flows (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (length(id) = 43 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    kind TEXT NOT NULL CHECK (length(kind) BETWEEN 2 AND 32 AND kind GLOB '[a-z]*'
        AND kind NOT GLOB '*[^a-z0-9_]*'),
    intent TEXT CHECK (intent IS NULL OR intent IN ('signin', 'signup', 'link', 'proof')),
    -- The account, or for a sign-up flow the client's proposed account ID, which has no row yet.
    account_id TEXT,
    device_id TEXT,
    device_proof_hash TEXT CHECK (device_proof_hash IS NULL OR (length(device_proof_hash) = 64
        AND device_proof_hash NOT GLOB '*[^0-9a-f]*')),
    challenge TEXT CHECK (challenge IS NULL OR (length(challenge) = 43
        AND challenge NOT GLOB '*[^A-Za-z0-9_-]*')),
    rp_id TEXT,
    nonce_hash TEXT CHECK (nonce_hash IS NULL OR (length(nonce_hash) = 64
        AND nonce_hash NOT GLOB '*[^0-9a-f]*')),
    state_hash TEXT CHECK (state_hash IS NULL OR (length(state_hash) = 64
        AND state_hash NOT GLOB '*[^0-9a-f]*')),
    pkce_hash TEXT CHECK (pkce_hash IS NULL OR (length(pkce_hash) = 64
        AND pkce_hash NOT GLOB '*[^0-9a-f]*')),
    redirect_uri TEXT CHECK (redirect_uri IS NULL OR length(redirect_uri) <= 255),
    provider TEXT,
    subject TEXT CHECK (subject IS NULL
        OR (length(subject) BETWEEN 1 AND 255 AND subject NOT GLOB '*[^ -~]*')),
    device_code TEXT CHECK (device_code IS NULL OR length(device_code) <= 512),
    next_poll_at INTEGER,
    -- Added by 0007_provider_completion.sql: GitHub's current polling interval.
    poll_interval_seconds INTEGER CHECK (poll_interval_seconds IS NULL OR poll_interval_seconds >= 5),
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    consumed_at INTEGER,
    -- Added by 0007_provider_completion.sql: the one completion attempt allowed
    -- to write a session, identity or account for this flow.
    completion_attempt_id TEXT CHECK (completion_attempt_id IS NULL OR (length(completion_attempt_id) = 43
        AND completion_attempt_id NOT GLOB '*[^A-Za-z0-9_-]*')),
    completion_kind TEXT CHECK (completion_kind IS NULL OR completion_kind IN ('signin', 'link', 'social_signup')),
    completion_account_id TEXT CHECK (completion_account_id IS NULL OR (length(completion_account_id) = 34
        AND substr(completion_account_id, 1, 2) = 'a_' AND substr(completion_account_id, 3) NOT GLOB '*[^0-9a-f]*')),
    completion_succeeded INTEGER NOT NULL DEFAULT 0 CHECK (completion_succeeded IN (0, 1)),
    CHECK (expires_at > created_at),
    -- Provider flows name their provider; passkey and proof flows don't. A new provider's kind is
    -- just its name, so it needs no migration.
    CHECK ((kind IN ('passkey_signin', 'passkey_signup', 'passkey_register', 'proof'))
           = (provider IS NULL))
);
CREATE INDEX auth_flows_expiry ON auth_flows (expires_at);
CREATE UNIQUE INDEX auth_flows_completion_attempt ON auth_flows (completion_attempt_id)
    WHERE completion_attempt_id IS NOT NULL;

-- One row per device-approval request. The account, requesting device and requesting ephemeral
-- key never change; the approver's key, the sealed name and the sealed data key are filled once.
CREATE TABLE device_approvals (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (length(id) = 43 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    device_id TEXT NOT NULL CHECK (length(device_id) = 34 AND substr(device_id, 1, 2) = 'd_'
        AND substr(device_id, 3) NOT GLOB '*[^0-9a-f]*'),
    commit_hash TEXT NOT NULL
        CHECK (length(commit_hash) = 43 AND commit_hash NOT GLOB '*[^A-Za-z0-9_-]*'),
    -- Revealed with the sealed name, after the challenge; checked against commit_hash.
    epk TEXT CHECK (epk IS NULL OR (length(epk) = 43 AND epk NOT GLOB '*[^A-Za-z0-9_-]*')),
    CHECK (epk IS NULL OR approver_epk IS NOT NULL),
    approver_device_id TEXT,
    approver_epk TEXT CHECK (approver_epk IS NULL
        OR (length(approver_epk) = 43 AND approver_epk NOT GLOB '*[^A-Za-z0-9_-]*')),
    sealed_name TEXT CHECK (sealed_name IS NULL
        OR (length(sealed_name) BETWEEN 55 AND 400 AND sealed_name NOT GLOB '*[^A-Za-z0-9_-]*')),
    wrapped_key TEXT CHECK (wrapped_key IS NULL
        OR (length(wrapped_key) = 98 AND wrapped_key NOT GLOB '*[^A-Za-z0-9_-]*')),
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    completed_at INTEGER,
    cancelled_at INTEGER,
    CHECK (expires_at > created_at),
    CHECK (cancelled_at IS NULL OR completed_at IS NULL)
);
CREATE INDEX device_approvals_by_account ON device_approvals (account_id, expires_at);

CREATE TRIGGER device_approvals_immutable
BEFORE UPDATE OF account_id, device_id, commit_hash, created_at, expires_at ON device_approvals
BEGIN SELECT RAISE(ABORT, 'approval_immutable'); END;

-- epk, approver_epk and sealed_name are each set once.
CREATE TRIGGER device_approvals_set_once
BEFORE UPDATE OF epk, approver_epk, sealed_name ON device_approvals
WHEN (OLD.epk IS NOT NULL AND NEW.epk IS NOT OLD.epk)
  OR (OLD.approver_epk IS NOT NULL AND NEW.approver_epk IS NOT OLD.approver_epk)
  OR (OLD.sealed_name IS NOT NULL AND NEW.sealed_name IS NOT OLD.sealed_name)
BEGIN SELECT RAISE(ABORT, 'approval_immutable'); END;
```

- `has_password` is written only by `social/signup` (0), `passkey/signup/finish` (0) and
  `sign-in/password` (1). `POST /v2/auth/signup` leaves the default 1. A passwordless account's
  `auth_hash`, `kdf_*`, `salt` and `wrapped_key` are random values that satisfy the existing
  `CHECK`s and that no client holds; its row is indistinguishable from a password account to the
  storage layer.
- A passwordless account's password wrap is opened by nobody; `has_password` is what tells the app
  to offer "Set a password" rather than "Change password". The server knows which accounts have a
  password, which is metadata it already has from the method list.
- A passkey proof is an `auth_flows` row with `kind = 'proof'`, the account, the challenge and a
  5-minute life; the assertion in the action names its credential. A provider proof is a provider
  flow with `intent = 'proof'` and the account; its subject is checked against that account's
  linked identity.
- The cron deletes `auth_flows` past `expires_at + 1 day` and `device_approvals` past
  `expires_at + 1 day`; a deletion of the account deletes its rows first (`account_identities`,
  `account_passkeys`, `device_approvals`, and `auth_flows` by `account_id`).
- D1 has no read replicas in this deployment, so a just-created flow is always read from the
  primary (L8).

### Migration `cloud/migrations/0007_provider_completion.sql`

The provider flows need two pieces of state the 0006 schema did not carry. Both are additive and
leave old rows valid:

- `poll_interval_seconds` keeps GitHub's current device-flow interval, so a `slow_down` raises the
  next wait cumulatively instead of resetting to a fixed five seconds.
- `completion_attempt_id`, `completion_kind`, `completion_account_id` and `completion_succeeded`
  claim a provider completion before its session, identity or account write. The unique attempt ID
  gates every write in the same D1 batch, so an expired or already-consumed flow cannot commit a
  session; a lost-answer retry can still replay the successful answer. The success flag is set only
  after the result exists in that transaction.

`0007` is applied after `0006` by the same `npm run migrate:remote` step. It is the only additive
migration this step introduces; no existing column, constraint, index or trigger changes.

### Migrations `0008_apple_app_secret.sql`, `0009_socket_closes.sql` and `0010_end_flows_with_sessions.sql`

`0008` adds `auth_flows.app_secret_hash` (SHA-256 of the secret the Apple callback hands the app
through `kanso://auth/apple?…&s=`) and `auth_flows.app_claimed_at` (set by the first poll that
sends it). `0009` adds `socket_closes (account_id PRIMARY KEY, requested_at)`: the admin script's
`disable` and `delete` insert the account, and the cron closes its event sockets and deletes the
row (at most 100 accounts a run). `0010` adds two triggers: when a session is revoked, for any
reason, its device's unconsumed `link` and `proof` flows (provider flows by intent, passkey proofs by
kind) and `passkey_register` flows are consumed in the same transaction; when an account is
disabled, all of the account's. All three are additive and applied by `npm run migrate:remote`.

### Migration `0011_push.sql`

Adds to `sessions`: `push_platform`, `push_token`, `push_environment`, `push_updated_at`,
`push_last_sent_at` and `push_due_at`, all nullable, with `CHECK`s on the token's shape for its
platform, on the three being set together, and on `push_due_at` existing only with a token; a unique
index on `push_token` where it is set; an index of an account's due rows; and the trigger
`sessions_push_clear` described under "PUT and DELETE /v2/account/devices/<device_id>/push".
Additive, applied by `npm run migrate:remote` before the Worker that uses it is deployed.

### Sessions in the Worker

Every session route authenticates with one read:

```sql
SELECT s.device_id, s.last_seen_at, s.revoked_reason, s.proof_hash, a.id, a.username,
       a.disabled_at, a.has_password
FROM sessions s JOIN accounts a ON a.id = s.account_id WHERE s.token_hash = ?1
```

- No row: 401 `unknown`.
- A revoked row: 401 with its reason.
- `last_seen_at` more than 180 days ago: revoke it (`expired`), then 401.
- Otherwise `last_seen_at` is updated when it is more than an hour old.

The query also selects `s.proof_hash`. Session routes do not need it, but the account-event upgrade
compares the supplied device proof with it before forwarding the socket to `ACCOUNT_ROOM`.

Auth and rate-limit reads always go to the primary. The Worker never uses D1's Sessions API or read
replicas (L8).

### Quotas and capacity

These are the D1 settings defaults, sized for Workers Paid. D1 there allows 10 GB per database; the
arithmetic uses 10,000,000,000 bytes. Paid includes about 50 million rows written a month, and R2
Paid has no size cap. Every trigger reads its setting with `coalesce(…, 0)` (`'closed'` for
`signup`), and rows of `settings` and `daily` can't be deleted or renamed, so a lost setting
refuses rather than allows (security re-check 1).

| Setting | Default | Meaning |
|---|---|---|
| `max_accounts` | 50 | |
| `signups_per_day` | 5 | Server-wide, per UTC day; deleting an account doesn't give one back |
| `quota_docs` | 50,000 | Live documents of every kind; deleting frees space |
| `quota_doc_bytes` | 100 MiB (104,857,600) | `length(sealed) + 512` per row, tombstones included. Real cost. |
| `quota_files` | 10,000 | Stored file objects |
| `quota_file_bytes` | 500 MiB (524,288,000) | `size + 1,024` per file |
| `writes_per_day` | 20,000 | Pushed documents plus file uploads (and refused push bodies), per account, unless the account's `quota_writes_per_day` says otherwise (`quota <user> --writes-per-day`). A fifteenth of `global_writes_per_day`, so no one account can spend the server's day. A library that needs more in one day, like the owner's first upload or a re-merge of a large library, pauses at the cap and finishes after UTC midnight; the owner raises their own account instead. |
| `upload_bytes_per_day` | 1 GiB (1,073,741,824) | Uploaded file bytes per account per UTC day, refused uploads included. Twice the default file bytes quota, under a fifth of `global_upload_bytes_per_day`. |
| `global_doc_bytes` | 6,000,000,000 | Sum over all accounts |
| `global_file_bytes` | 50,000,000,000 | Sum over all accounts |
| `global_writes_per_day` | 300,000 | Stored document writes (new and updated) per UTC day, all accounts together. Past it, only accounts above their fair share (÷ `max_accounts`: 6,000 writes) are refused, so a day's total stays under about 600,000. |
| `global_upload_bytes_per_day` | 5,000,000,000 | File bytes uploaded per UTC day, all accounts, replacements included. Past it, only accounts above their fair share (÷ `max_accounts`: 100,000,000 bytes) are refused, so a day's total stays under about 10,000,000,000. |

**The 512-byte row cost** covers the row's other columns and its entries in both indexes. A
tombstone measures about 300 B with indexes. `quota.test.ts` pins the figure: it inserts 10,000
random rows into a fresh database and checks that `PRAGMA page_count × page_size` grew by no more
than `Σ(length(sealed) + 512)`.

**D1 budget** (10,000 MB):

| Item | Arithmetic | Bytes |
|---|---|---|
| Documents | global cap; 50 × 104,857,600 = 5,242,880,000 fits inside, leaving 757,120,000 for owner overrides | 6,000,000,000 |
| `account_files` rows | 50 × 10,000 × 200 B | 100,000,000 |
| Sessions | 50 accounts × at most 100 rows (10 active plus 90 days of revoked) × 400 B, rounded up | 10,000,000 |
| Sign-in methods | 50 accounts × (20 identities + 20 passkeys + 20 flows and approvals) × 400 B | 1,200,000 |
| `limits` | the binding admits at most 20 auth requests per minute per IP group; rows go when their window ends; budgeted | 200,000,000 |
| Legacy `docs`/`counter` until the purge | 823 notes; `legacy-status` prints the real size | 100,000,000 |
| Total | | 6,411,200,000 |

That leaves 3.59 GB, or 36%, of headroom. The bytes quota is the real limit on documents: 50,000
live documents only reach 100 MiB at an average of about 1,585 bytes sealed (104,857,600 ÷ 50,000 − 512).

**R2:** 50 × 524,288,000 = 26,214,400,000 B of charged file bytes. That is under the 50 GB global
cap, which only overrides can approach, and costs at most 50 GB × $0.015 = $0.75 a month.

**Rows written:** an accepted document writes about 5 rows (the account row, the document row, its
two index entries and the `daily` counter). One account at the daily maximum writes 20,000 × 5 =
100,000 rows a day, which is 3,100,000 in a 31-day month, inside the included 50 million. It takes
seventeen accounts at the maximum all month to exceed it. With fair shares, `global_writes_per_day`
bounds the whole server at about twice 300,000 × 5, 3,000,000 rows a day, about 93 million in a
31-day month in the worst case (every account at its fair share after the heavy ones have used
the cap up, every day). Beyond the included rows it costs $1 per million, so that worst case is
about $43 a month; and the `accounts` command shows `writes_today` for each account. The owner's
first upload is about 909 documents, roughly 3,600 rows.

**Requests:** 50 accounts × 3 Macs × about 400 requests a day is about 1.8 million a month, against
10 million included.

**Admin overcommit rule:** `max-accounts`, `quota` and `default-quota` refuse, unless given
`--force`, any change after which `Σ effective quotas of existing accounts + (max_accounts −
accounts) × default` exceeds a global cap. The daily budgets have a per-account rule:
`writes-per-day`, `upload-bytes-per-day`, `global-writes-per-day`, `global-upload-bytes-per-day`
and `quota --writes-per-day` refuse, unless given `--force`, any change after which one account's
daily budget (the default, an override, or the upload budget) is over its server-wide cap. Each,
and `max-accounts`, prints each cap's fair share (cap ÷ `max_accounts`), how many accounts at their
whole budget reach the cap (after which only accounts above their fair share wait for midnight),
and the day's bound of about twice the cap. `accounts` shows the fair shares beside today's
counts.

### R2 layout, purges and the cron

- Sealed files: `accounts/<account_id>/<wire_file_id>`. The legacy plaintext sits at `files/<id>` until its purge.
- **A purge round** is `list({prefix, limit: 1000})` then `delete(keys)`. A purge row is deleted
  only when a list returns empty and the row is more than an hour old, so an upload still in flight
  can't leave an orphan (L6). Account deletion runs up to 20 rounds inline.
- **The cron** (`*/15 * * * *`, `scheduled()`) does five things:
  - runs up to 50 purge rounds;
  - deletes `limits` rows whose window has ended;
  - reconciles R2 with `account_files` (security re-check 8): it lists a bounded number of
    accounts' prefixes per run, taking accounts in turn, deletes objects that have no row and were
    uploaded more than an hour ago, and sets a row's `size` to its object's size where they differ,
    adjusting `file_bytes` in the same batch. It doesn't set `recounted_at`: a correction that grows
    `file_bytes` meets the quota triggers like an upload, because two racing uploads of one ID can
    leave a small row over a large object. If a quota refuses it, the row goes (refunded) and then
    the object;
  - revokes active sessions unseen for 180 days (`expired`);
  - deletes revoked sessions more than 90 days old.

## Worker configuration

Changes to `cloud/wrangler.jsonc`:

```jsonc
"observability": {
  "enabled": true,
  "logs": { "enabled": true, "invocation_logs": false },
  "traces": { "enabled": false }
},
"triggers": { "crons": ["*/15 * * * *"] },
"ratelimits": [
  { "name": "AUTH_LIMITER", "namespace_id": "1001", "simple": { "limit": 20, "period": 60 } },
  { "name": "SESSION_LIMITER", "namespace_id": "1002", "simple": { "limit": 120, "period": 60 } }
],
"durable_objects": {
  "bindings": [
    { "name": "NOTE_ROOM", "class_name": "NoteRoom" },
    { "name": "ACCOUNT_ROOM", "class_name": "AccountRoom" }
  ]
},
"migrations": [
  { "tag": "v1", "new_sqlite_classes": ["NoteRoom"] },
  { "tag": "v2", "new_classes": ["AccountRoom"] }
]
// AUTH_PEPPER (required): wrangler secret put AUTH_PEPPER. SYNC_TOKEN is no longer read.
```

Push notifications add the var `APNS_TOPIC` (`app.kanso.Kanso`) and four secrets, set with
`wrangler secret put`: `APNS_KEY_P8` (the `.p8` PEM), `APNS_KEY_ID`, `APNS_TEAM_ID` and
`FCM_SERVICE_ACCOUNT` (the service account's JSON key). The `AccountRoom` class keeps its v2
`new_classes` migration: alarms work on it, so no migration tag is added.

Merge the new `ACCOUNT_ROOM` binding and v2 migration with the existing `NOTE_ROOM` and v1
migration. Do not replace or renumber the deployed v1 migration; append v2.

The Worker's environment type is `Env { DB: D1Database; FILES: R2Bucket;
NOTE_ROOM: DurableObjectNamespace<NoteRoom>; ACCOUNT_ROOM: DurableObjectNamespace<AccountRoom>;
AUTH_PEPPER?: string; AUTH_LIMITER?: RateLimit; SESSION_LIMITER?: RateLimit;
PUBLIC_LIMITER?: RateLimit; REROUT_API_KEY?: string; REROUT_PROJECT_ID?: string;
PASSKEY_RP_ID?: string; PASSKEY_ORIGINS?: string; APP_ASSOC_APPS?: string;
APPLE_CLIENT_ID?: string; APPLE_TEAM_ID?: string; APPLE_KEY_ID?: string;
APPLE_PRIVATE_KEY?: string; APPLE_REDIRECT_URI?: string; GOOGLE_CLIENT_ID?: string;
GOOGLE_CLIENT_SECRET?: string; GITHUB_CLIENT_ID?: string }`.
`ACCOUNT_ROOM` uses `new_classes` rather than SQLite storage: the event room keeps no rows.
`REROUT_API_KEY` (optional, a secret) turns on published notes' short links:
`npx wrangler secret put REROUT_API_KEY` in `cloud/`, pasting a key of the Kanso project on Rerout;
without it there are none.

Sign-in methods are optional per deployment and are advertised only when configured:

| Name | Kind | What it is |
|---|---|---|
| `PASSKEY_RP_ID` | var | The exact public host passkeys are scoped to (`sync.kansoapp.app`). Unset, or an IP, means no passkeys. |
| `PASSKEY_ORIGINS` | var, JSON | `["https://sync.kansoapp.app", …]`; each must be the RP ID or a subdomain of it. |
| `APP_ASSOC_APPS` | var, JSON | `["<TEAMID>.app.kanso.Kanso"]`, served in the AASA file. |
| `APPLE_CLIENT_ID` | var | The Sign in with Apple Services ID. |
| `APPLE_TEAM_ID` | var | The 10-character Team ID; also the client secret's `iss`. |
| `APPLE_KEY_ID` | var | The 10-character Key ID of the Sign in with Apple `.p8` key. |
| `APPLE_PRIVATE_KEY` | secret | The `.p8` private key, PKCS#8 PEM, exactly as Apple issued it. |
| `APPLE_REDIRECT_URI` | var | `https://sync.kansoapp.app/v2/auth/apple/callback`, the Services ID's return URL. |
| `GOOGLE_CLIENT_ID` | var | The Desktop-app OAuth client ID. |
| `GOOGLE_CLIENT_SECRET` | secret | The Desktop-app client secret (not confidential for a desktop client; it stays out of the app). |
| `GITHUB_CLIENT_ID` | var | The OAuth app's client ID. |

A provider is advertised only when its client ID and (where it has one) its secret are present.
`AUTH_PEPPER` also seals GitHub's short-lived device code and the flow results in D1; rotating it
cancels flows in flight, which expire in minutes anyway. The Worker serves
`/.well-known/apple-app-site-association` itself, not from `public/`, with `application/json`, no
redirects and a short `Cache-Control`; it holds no secret. Apple's current web Sign in with Apple
setup stores the domain and return URL on the Services ID and needs no file on the server. Commands
run in `cloud/`, and the deploy token is only referenced by name:

```bash
export CLOUDFLARE_API_TOKEN="$IMNGONII_CLOUDFLARE_API_TOKEN"
pepper=$(openssl rand 32 | base64 | tr '+/' '-_' | tr -d '=\n')
printf %s "$pepper" | npx wrangler secret put AUTH_PEPPER
# Optional backup in a password manager: losing it changes fake prelogin answers, resets rate-limit
# counters, and cancels sign-in flows in flight. Then:
unset pepper
```

Tests and local dev use a fixed test pepper: in `vitest.config.ts` bindings, and as `--var
AUTH_PEPPER:<value>` on `wrangler dev`. `.dev.vars` and `cloud/.wrangler/` go in `.gitignore`.
Provider tests mock `fetch` and use a local JWKS; they never call Apple, Google or GitHub.

## Client: kanso-sync

### Modules

| Module | Contents |
|---|---|
| `crypto.rs` | `KdfParams` (`FLOOR`, `DEFAULT`, `TEST`, `CEILING`); derivation; HKDF; wrap and unwrap (`WrapPurpose::Password`, `Recovery` and the new `Passkey`); `seal_doc`/`open_doc` (→ opened, undecryptable or too new); `seal_file`/`open_file`; `seal_name`/`open_name`; `wire_id`, `wire_rev`, `key_id`; `PRF_INPUT`, `passkey_kek`; `RecoveryKey`, `Password`, zeroizing key types (`AuthKey`, `Kek`, `DataKey`, `DeviceProof`). Pure. |
| `passkey.rs` | `PasskeyRegistration`, `PasskeyAssertion`, `PasskeyInfo`, the WebAuthn wire shapes, base64url helpers and `passkey_wrapped_key`. The same shapes carry libfido2's Linux CTAP2 result. Pure. |
| `security_key.rs` | Linux only: pure helpers for the CTAP2 path — the CTAP salt mapping, `SecurityKeyRegistration` and `SecurityKeyAssertion`, and their conversion to `PasskeyRegistration`/`PasskeyAssertion`. Device enumeration and every libfido2 call stay in the app. |
| `approval.rs` | `ApprovalKeys`, `Approval`, `ApprovalRequest`, `Approved`: the X25519 exchange, the six-digit code, the sealed name and the data-key wrap. Pure. |
| `events.rs` | `AccountEvent` and `EventStream`: the authenticated account WebSocket over `tungstenite`/`rustls`, one blocking connection, typed `approval.created`/`named`/`cancelled` events and bounded reconnect. It never opens UI. |
| `provider.rs` | `Provider`, the flow and ticket shapes, `ProviderSignIn`, `NewAccount`, `AccountMethods`. Pure. |
| `username.rs` | `normalize_username`, reserved list. Pure. |
| `wire.rs` | `WireDoc`, `WirePushResult` (`Ok`, `Conflict`, `Lost`, `Behind`), `WirePage` (with `more`, `head_seq`, `epoch`, `key_id`), patterns, limits, the `WireRemote` trait, `ApiError` and its codes. |
| `remote.rs` | The engine's plaintext `Remote` trait. `PushResult` gains `Lost`, `Behind` and `Unreadable { newer_format }`. `ChangesPage` gains `more`, `head_seq`, `epoch: Option`, `key_id: Option`, `skipped`, `repairable` and `too_new`. `put_file(id, bytes)` and `get_file(id) -> Option<Vec<u8>>`. |
| `sealed.rs` | `Sealed<W: WireRemote>`: seals outgoing documents and opens incoming ones, maps local IDs and revisions to wire ones and back, and keeps the repair bases for documents that were undecryptable. `PlainView`, the decrypted test view (below), lives here. |
| `api.rs` | The `Api` trait (auth and account routes) and `Backend: Send + Sync` (`api(server)`, `remote(session)`, `kdf_floor()`, `kdf_default()`). |
| `http.rs` | `HttpBackend`: every v2 route over blocking `ureq`, with no redirects and the response caps. |
| `memory.rs` | `MemoryServer`: implements `Backend` with the Worker's sync, auth, session, quota, limit and sign-in rules, run against the shared fixtures. A `TestProvider` issues fixed ID tokens and subjects, and a `TestAuthenticator` is a software P-256 credential that signs assertions and returns a deterministic PRF output. Its auth hashes are SHA-256, as in the Worker. |
| `account.rs` | The public account functions, `Session`, `LockedSession`, `Step`. |
| `binding.rs` | Device identity (ID and proof), `meta_keys`, the guard pairs, `local_library`, `plan`, `bind`, `unbind`. |
| `keychain.rs` | Replaces `credentials.rs`: `SecretStore`, `Keychain`, `MemorySecrets`, the items below. |
| `engine.rs` | Pull and push as now, with cancellation, the guard, epoch and head checks, re-merge, repair, stale handling, quota and pause handling, and file MD5 binding. |
| `verify.rs` | `verify` (with an optional legacy manifest) and `estimate`. |
| `bin/kanso-sync.rs` | `status`, `sync`, `verify`, `estimate`, `unbind`. |

### Public API the app calls

```rust
pub const DEFAULT_SERVER: &str = "https://sync.kansoapp.app";

pub trait Backend: Send + Sync { /* api, remote, kdf_floor, kdf_default, retry_pause (1 s, then 4 s) */ }
pub struct HttpBackend;                 // the app
pub struct MemoryServer { /* … */ }     // tests and snapshots only; never constructed by main.rs

pub struct DeviceIdentity { pub library_id: String, pub name: String, /* device secret, private */ }
impl DeviceIdentity {
    pub fn device_id(&self, account_id: &str) -> String;
    pub fn device_proof(&self, origin: &str, account_id: &str) -> DeviceProof;
}
pub enum Step { StoppingSync, DerivingKeys, Contacting, Saving }
pub fn origin(server: &str) -> Result<String, EndpointError>;   // the origin the salt input binds
pub struct Password(/* NFC, Zeroizing */);
impl Password { pub fn new(text: &str, username: &str) -> Result<Password, PasswordError>; }
pub struct RecoveryKey(/* 16 bytes, Zeroizing */);
impl RecoveryKey { pub fn parse(text: &str) -> Result<RecoveryKey, RecoveryKeyError>; pub fn display(&self) -> String; }
pub fn normalize_username(input: &str) -> Result<String, UsernameError>;

/// A signed-in library: Send + Sync, shared as Arc<Session>. The token and data key are
/// private and zeroized on drop. `server` and `origin` come from the Keychain item.
pub struct Session { pub server: String, pub origin: String, pub account_id: String,
                     pub username: String, pub device_id: String, /* … */ }
pub struct DeviceInfo { pub id: String, pub name: Option<String>, pub created_at: i64, pub last_seen_at: i64, pub current: bool }
pub struct Usage { pub docs: u64, pub doc_bytes: u64, pub files: u64, pub file_bytes: u64, pub writes_today: u64, pub quota: Quota }
pub enum SignupState { Open, Closed, Full }
pub struct SignedUp { pub session: Session, pub recovery_key: RecoveryKey }
pub struct SignedIn { pub session: Session, pub evicted: Option<DeviceInfo> }
pub struct Recovered { pub session: Session, pub recovery_key: RecoveryKey }   // the new key
pub struct PasswordChanged { pub signed_out_devices: usize, pub recovery_key: RecoveryKey }
/// A call that sets a new recovery key failed. `recovery_key` is the unsaved key left in
/// `sync-recovery-key:<library>` whenever the server may hold it: show it, never drop it.
pub struct RotationError { pub error: AccountError, pub recovery_key: Option<Box<PendingRecoveryKey>> }

pub fn server_info(b: &dyn Backend, server: &str) -> Result<SignupState, AccountError>;
/// The methods a server advertises: passkeys (with whether Linux security keys are accepted), the
/// configured providers and the account-event path. An old server answers with none, which means
/// username and password only and no live approval channel.
pub struct PasskeyMethods { pub rp_id: String, pub security_key: bool }
pub struct ServerAuth { pub password: bool, pub passkeys: Option<PasskeyMethods>,
                        pub providers: Vec<Provider>, pub account_events: Option<String> }
pub fn server_auth(b: &dyn Backend, server: &str) -> Result<ServerAuth, AccountError>;
pub fn sign_up(b: &dyn Backend, secrets: &dyn SecretStore, server: &str, username: &str, password: &Password, device: &DeviceIdentity, progress: &dyn Fn(Step)) -> Result<SignedUp, RotationError>;
pub fn sign_in(b: &dyn Backend, server: &str, username: &str, password: &Password, device: &DeviceIdentity, progress: &dyn Fn(Step)) -> Result<SignedIn, AccountError>;
pub fn recover(b: &dyn Backend, secrets: &dyn SecretStore, server: &str, username: &str, key: &RecoveryKey, new_password: &Password, device: &DeviceIdentity, progress: &dyn Fn(Step)) -> Result<Recovered, RotationError>;
pub fn sign_out(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session) -> Result<(), AccountError>; // always tries the logout; queues it on failure
pub fn discard_session(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session) -> Result<(), AccountError>; // logout only: the library's items stay
pub fn sign_out_locked(b: &dyn Backend, secrets: &dyn SecretStore, locked: &LockedSession) -> Result<(), AccountError>; // logout and delete the pending item
pub fn flush_pending_logouts(b: &dyn Backend, secrets: &dyn SecretStore, library_id: &str);
pub fn change_password(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session, current: &Password, new: &Password, progress: &dyn Fn(Step)) -> Result<PasswordChanged, RotationError>;
pub fn new_recovery_key(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session, current: &Password, progress: &dyn Fn(Step)) -> Result<RecoveryKey, RotationError>;
pub fn new_recovery_key_with_proof(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session,
                                   proof: &Proof, recovery: Option<&RecoveryKey>, progress: &dyn Fn(Step))
                                   -> Result<RecoveryKey, RotationError>;
pub fn delete_account(b: &dyn Backend, session: &Session, password: &Password, typed_username: &str, progress: &dyn Fn(Step)) -> Result<(), AccountError>;
pub fn delete_account_with_proof(b: &dyn Backend, session: &Session, proof: &Proof,
                                 recovery: Option<&RecoveryKey>, typed_username: &str,
                                 progress: &dyn Fn(Step)) -> Result<(), AccountError>;
pub fn devices(b: &dyn Backend, session: &Session) -> Result<Vec<DeviceInfo>, AccountError>;
pub fn revoke_device(b: &dyn Backend, session: &Session, device_id: &str) -> Result<(), AccountError>;
pub fn usage(b: &dyn Backend, session: &Session) -> Result<Usage, AccountError>;

/// What the server advertises and what this account has. `rp_id` is None when the server offers no
/// passkeys; `providers` only the configured ones.
pub struct AccountMethods { pub has_password: bool, pub rp_id: Option<String>,
                            pub available: Vec<Provider>, pub linked: Vec<LinkedProvider>,
                            pub passkeys: Vec<PasskeyInfo> }
/// An open lowercase name (`apple`, `google`, `github`, later `microsoft`), not an enum: a new
/// provider must not need a client release to be stored, and an app that doesn't know one doesn't
/// show its button.
pub struct Provider(/* String */);
impl Provider { pub const APPLE: Provider; pub const GOOGLE: Provider; pub const GITHUB: Provider; }
pub struct LinkedProvider { pub provider: Provider, pub created_at: i64 }
pub struct PasskeyInfo { pub credential_id: Vec<u8>, pub created_at: i64,
                         pub last_used_at: Option<i64>, pub prf: bool, pub transports: Vec<String> }

/// A WebAuthn ceremony's result, as the platform returned it. The app base64url-decodes and
/// fills these; `kanso-sync` never calls a platform API. `prf` is the 32-byte output when the
/// extension was requested and answered.
pub struct PasskeyRegistration { pub credential_id: Vec<u8>, pub attestation_object: Vec<u8>,
                                 pub client_data_json: Vec<u8>, pub transports: Vec<String>,
                                 pub prf: Option<[u8; 32]> }
pub struct PasskeyAssertion { pub credential_id: Vec<u8>, pub authenticator_data: Vec<u8>,
                              pub client_data_json: Vec<u8>, pub signature: Vec<u8>,
                              pub user_handle: Option<Vec<u8>>, pub prf: Option<[u8; 32]> }

/// A device that has a session but not the data key. It can request approval, list methods and
/// sign out; it can't sync. `server`, `origin` and `device_id` come from the pending item.
pub struct LockedSession { pub server: String, pub origin: String, pub account_id: String,
                           pub username: String, pub device_id: String, pub key_id: String,
                           pub epoch: String, pub wrapped_key: Option<String>,
                           pub evicted: Option<DeviceInfo>, /* token, private */ }
pub enum PasskeySignIn { Unlocked(SignedIn), Locked(LockedSession) }
pub enum ProviderSignIn { Locked(LockedSession), NewAccount(NewAccount) }
pub struct NewAccount { pub ticket: String, pub suggested_username: String }

pub struct ApprovalKeys(/* local X25519 secret, zeroized */);
impl ApprovalKeys {
    pub fn generate() -> ApprovalKeys;
    pub fn epk(&self) -> [u8; 32];
    /// Both sides call this with the other side's public key. Returns the key, the six-digit
    /// code and the seal/open helpers below.
    pub fn complete(&self, other_epk: &[u8; 32], account_id: &str, request_id: &str)
        -> Result<Approval, CryptoError>;
}
pub struct Approval(/* approval_key, code */);
impl Approval {
    pub fn code(&self) -> String;                                  // "123 456"
    pub fn seal_name(&self, account_id: &str, request_id: &str, epk_new: &[u8; 32],
                     epk_approver: &[u8; 32], name: &str) -> String;
    pub fn open_name(&self, account_id: &str, request_id: &str, epk_new: &[u8; 32],
                     epk_approver: &[u8; 32], sealed: &str) -> Option<String>;
    pub fn wrap(&self, account_id: &str, request_id: &str, epk_new: &[u8; 32],
                epk_approver: &[u8; 32], new_device: &str, approver_device: &str,
                data_key: &DataKey) -> String;
    pub fn unwrap(&self, account_id: &str, request_id: &str, epk_new: &[u8; 32],
                  epk_approver: &[u8; 32], new_device: &str, approver_device: &str,
                  wrapped: &str) -> Result<DataKey, CryptoError>;
}
pub struct ApprovalRequest { pub id: String, pub epk: [u8; 32], pub expires_at: i64,
                             pub other_devices: usize }
pub struct Approved { pub approver_epk: [u8; 32], pub approver_device_id: String,
                      pub wrapped_key: String }

/// A fresh proof for a sensitive change (see "Changing a way in needs a fresh proof").
pub enum Proof<'a> { Password(&'a AuthKey),
                     Passkey { flow: String, assertion: PasskeyAssertion },
                     Provider { flow: String } }

pub fn methods(b: &dyn Backend, session: &Session) -> Result<AccountMethods, AccountError>;
pub fn sign_up_with_passkey(b: &dyn Backend, secrets: &dyn SecretStore, server: &str,
                            username: &str, device: &DeviceIdentity, registration: PasskeyRegistration,
                            progress: &dyn Fn(Step)) -> Result<SignedUp, RotationError>;
pub fn sign_in_with_passkey(b: &dyn Backend, server: &str, device: &DeviceIdentity,
                            assertion: PasskeyAssertion, progress: &dyn Fn(Step))
                            -> Result<PasskeySignIn, AccountError>;
pub fn sign_up_with_provider(b: &dyn Backend, secrets: &dyn SecretStore, server: &str,
                             provider: Provider, flow: &str, username: &str, device: &DeviceIdentity,
                             login_device: &LoginDevice,
                             progress: &dyn Fn(Step)) -> Result<SignedUp, RotationError>;
pub fn sign_in_with_provider(b: &dyn Backend, server: &str, provider: Provider, flow: &str,
                             device: &DeviceIdentity, progress: &dyn Fn(Step))
                             -> Result<ProviderSignIn, AccountError>;
/// Open a locked session with the library's saved data key, without a network request.
pub fn unlock_with_local_key(locked: &LockedSession, local: &Session) -> Result<SignedIn, AccountError>;
/// Unlock a locked session with its password: prelogin, derive, unwrap, compare the key ID,
/// and prove the persisted provider/passkey device on the same device ID.
pub fn unlock_with_password(b: &dyn Backend, locked: &LockedSession, password: &Password,
                            device: &DeviceIdentity, progress: &dyn Fn(Step))
                            -> Result<SignedIn, AccountError>;
/// Refresh `has_password` using only the locked session's token.
pub fn methods_locked(b: &dyn Backend, locked: &LockedSession) -> Result<bool, AccountError>;
pub fn link_identity(b: &dyn Backend, session: &Session, provider: Provider, flow: &str)
                     -> Result<(), AccountError>;
pub fn unlink_identity(b: &dyn Backend, session: &Session, provider: Provider, proof: &Proof,
                       recovery: Option<&RecoveryKey>) -> Result<(), AccountError>;
pub fn add_passkey(b: &dyn Backend, session: &Session, registration: PasskeyRegistration,
                   proof: &Proof) -> Result<PasskeyInfo, AccountError>;
pub fn remove_passkey(b: &dyn Backend, session: &Session, credential_id: &[u8], proof: &Proof,
                      recovery: Option<&RecoveryKey>) -> Result<(), AccountError>;
pub fn set_password(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session, new: &Password,
                    proof: &Proof, recovery: Option<&RecoveryKey>, progress: &dyn Fn(Step))
                    -> Result<PasswordChanged, RotationError>;
pub fn request_approval(b: &dyn Backend, session: &LockedSession, keys: &ApprovalKeys)
                         -> Result<ApprovalRequest, AccountError>;
/// Persist the stable request ID and requester secret before create, making a lost answer and
/// relaunch retry safe; reuse the same ID and keys until the request ends.
pub fn request_approval_resumable(b: &dyn Backend, secrets: &dyn SecretStore,
                                  locked: &LockedSession, request_id: &str,
                                  keys: &ApprovalKeys) -> Result<ApprovalRequest, AccountError>;
pub fn resume_approval(b: &dyn Backend, secrets: &dyn SecretStore, locked: &LockedSession)
                       -> Result<ApprovalRequest, AccountError>;
/// The approver: posts its challenge, reads the sealed name when it has arrived, and returns the
/// code and name for the prompt.
pub fn approve_request(b: &dyn Backend, session: &Session, request: &ApprovalRequest,
                       keys: &ApprovalKeys) -> Result<(Approval, Option<String>), AccountError>;
pub fn finish_approval(b: &dyn Backend, session: &Session, request: &ApprovalRequest,
                       approval: &Approval, data_key: &DataKey) -> Result<(), AccountError>;
pub fn wait_for_approval(b: &dyn Backend, session: &LockedSession, request: &ApprovalRequest,
                         keys: &ApprovalKeys) -> Result<Approved, AccountError>;

pub fn sync(store: &Store, b: &dyn Backend, session: &Session, options: &SyncOptions) -> Result<SyncReport, SyncError>;
pub fn verify(store: &Store, b: &dyn Backend, session: &Session, legacy: Option<&Path>, repair: bool) -> Result<VerifyReport, SyncError>;
pub fn estimate(store: &Store) -> Result<Estimate, StoreError>;   // charged bytes the library needs
pub fn problem(err: &SyncError) -> Option<SyncProblem>;             // None for Cancelled; BackOff for SyncError::BackOff
pub fn report_problem(report: &SyncReport) -> Option<SyncProblem>;  // problems in a sync that finished
```

`SyncOptions` gains two fields:

- `cancel: Option<Arc<AtomicBool>>`. The engine checks it between pages, batches and files, and
  returns `SyncError::Cancelled`.
- `cursor_key: &'static str`, default `"sync.v2.last_seq"`. Only the rollback test sets the legacy
  `"sync.last_seq"`, to run the old rules against `tests/common/legacy.rs`.
- `refetch_files: usize`, default 8: the most live files without bytes one sync fetches again
  (Sync with encryption › Files).

`SyncReport` gains `paused_limit: Option<String>` and `uploads_held_until: Option<i64>` (Sync with
encryption › Pauses). `SyncError` gains `BackOff { until: i64 }`, and `SyncProblem` gains
`ServerBusy { until }` and `BackOff { until }`, right after `Paused`.

In `remote.rs`, `ChangesPage.epoch` and `key_id` are `Option`. `Sealed` always fills them, because
`WirePage` requires both, so a server can't skip the checks by leaving them out. Only the legacy
test remote leaves them `None`.

`AccountError` has one variant per error code, plus:

- `Transport`
- `ServerOutdated`: 404 from `/v2/health`.
- `UnsafeKdf`: prelogin outside the backend's floor or ceilings.
- `KeyMismatch`: a failed unwrap or a different key ID.
- `Unauthorized { reason }`
- `Password`, `Username` and `RecoveryKey` validation.
- `Protocol` (an answer of the wrong shape), `Endpoint` (`check_endpoint`) and `Keychain`.
- `UnsavedRecoveryKey { username }`: the library's Keychain still holds another account's unsaved
  recovery key, which a new one would replace. Nothing was sent.

### Account flows

- **Sign up.**
  1. Normalize the username and check the password, offline.
  2. `server_info`, which must say `Open`.
  3. Generate the account ID, salt, data key and recovery secret.
  4. Derive the keys (`Step::DerivingKeys`) and wrap twice. Seal the device name.
  5. Save `sync-recovery-key:<library>`, unconfirmed (below), then `POST signup` (`Contacting`).
  6. Save the session's Keychain item, then bind (`Saving`).

  The salt input includes the username, so trying another name after `username_taken` derives again.
- **Sign in.**
  1. Prelogin, refusing parameters outside the floor or ceilings, to get the salt and account ID.
  2. Compute the device ID and proof, and derive.
  3. Log in, unwrap, and compare the key ID: a failure is `KeyMismatch` (the session is logged out,
     nothing is saved).
  4. Ask the binding question if needed, then bind and save.
  5. `PUT /v2/account/device` with the sealed name.
  6. If a device was evicted, notify "Signed out “*name*” to make room: an account can be signed
     in on 10 Macs at a time."
- **Recover.**
  1. Parse the key, then prelogin.
  2. `recover/start`; unwrap and compare the key ID.
  3. Make a new salt and a new recovery secret, derive, wrap, save the new key unconfirmed, and
     `recover/finish`. Recovery sets a password, so it is also the way into a passwordless account,
     and it removes every passkey (see "Rotations"); the screen says how many and that they can be
     added again from this device.
  4. Show the new recovery key screen.
- **Change password.** Prelogin for the salt, two derivations, and a new recovery secret: a
  recovery key that outlived the password would let whoever holds it take the account back. The
  new key is saved unconfirmed, then `account/password` carries `recovery_auth_key` and
  `recovery_wrapped_key` with the new password's values. Other Macs get 401 `password_changed`;
  this one shows the key screen over "Your password changed, and so did your recovery key: the one
  you saved no longer works."
- **New recovery key.** Derive the current `auth_key`, make a new recovery secret, wrap, save it
  unconfirmed, then `account/recovery-key`. Show the key screen.
- **Delete account.** Derive `auth_key` (or take the proof, plus the recovery key when the proof is
  a provider), then `account/delete`, then `unbind`.
- **Sign up with a passkey.**
  1. Choose a username (the form suggests one) and call `passkey/signup/start`; the Worker checks
     the name and returns the challenge and options.
  2. Generate the account ID, data key and recovery secret; save the recovery key unconfirmed.
  3. Register the credential with the platform, asking for PRF with `eval.first = PRF_INPUT`. If the
     platform answered, derive `passkey_kek` and wrap the data key; otherwise register without a
     wrap. Seal nothing else yet.
  4. `passkey/signup/finish` with the attestation, the recovery material, the key ID and the device;
     save the session, bind, sync.
  5. Show the recovery key screen. If PRF was absent, the screen says this passkey signs in but a
     new device will need approval or the recovery key.
- **Sign up with Apple, Google or GitHub.**
  1. Start the provider flow and open the system browser; finish it (Apple: poll; Google: finish;
     GitHub: poll). The answer is a new-account ticket and a suggested username.
  2. Choose the username; generate the account ID, data key and recovery secret; save the recovery
     key unconfirmed.
  3. `social/signup` with the ticket, username, recovery material, key ID and device; save the
     session, bind, sync.
  4. Show the recovery key screen; offer "Add a passkey" from Settings afterwards.
- **Sign in with a passkey.**
  1. `passkey/start` (username-less), platform assertion with `eval.first = PRF_INPUT`.
  2. `passkey/finish`. If this library already holds the data key for the account, compare `key_id`
     and use it; no approval. Otherwise, if `passkey_wrapped_key` is present, derive `passkey_kek`
     from the PRF output, unwrap and compare `key_id`; this is a full unlock, no approval needed.
  3. Otherwise the session is locked. If `wrapped_key` is present, offer "Unlock with your
     password"; otherwise request approval; the recovery key stays the last choice. The binding
     question and `bind` follow the unlock, as for a password sign-in.
  4. If the platform's authenticator has no PRF, the app doesn't offer that path and goes straight
     to the locked-session screen.
- **Sign in with a provider.** Complete the flow. If this library already holds the data key for
  the account, compare `key_id` and use it; no unlock. Otherwise the session is locked: offer the
  password when `has_password`, then approval, then recovery. Password unlock is a separate
  `/v2/auth/login` with the persisted provider device proof; the provider flow itself never carries
  a password wrap. Nothing is bound or synced until the user accepts Resume, Add or Replace after
  the unlock. A new-account answer is the sign-up flow above.
- **Link a provider.** Settings › Account › Sign-in methods › [Link] › choose the provider. The app
  asks for a `proof` first (a passkey assertion or the password; a provider-only account sets a
  password first), starts the flow with
  `intent: "link"`, completes it, and refreshes the list. A 409 `identity_taken` says the provider
  account is already linked elsewhere; it never says where.
- **Unlink a provider.** The row's [Unlink] asks for a `proof`. If it is the last method, the app
  asks whether the recovery key is saved, takes it if so, and sends `recovery_auth_key`; the Worker
  answers 204 (recovery only) or 409 `last_method`. The app never removes the last row silently.
- **Add a passkey.** Settings › Account › Sign-in methods › [Add a passkey…]. It takes a password
  or passkey `proof` (a provider-only account sets a password first), runs the same registration
  as sign-up, and shows whether the new passkey can unlock.
- **Remove a passkey.** The row's [Remove] takes a `proof`, and the recovery key if it is the last
  method. The list refreshes; other devices keep their data key.
- **Set a password.** Offered when `has_password` is false: [Set a password…] takes a `proof`, the
  new password and its confirmation, makes a new recovery secret, and calls `sign-in/password`;
  with no password or passkey yet, it also asks for the recovery key, because a provider proof
  alone must not be enough to add a password. It leads to the recovery key screen and signs other
  devices out. The row then reads "Change password…" and uses the existing route.
- **Approve a device** (an existing device).
  1. While unlocked, the app holds the account event socket and does one approvals `GET` on launch,
     when Settings › Account opens, on foreground and after every reconnect. There is no timer.
  2. `approval.created` fetches the list once and shows "A device wants to join *ngoni*". The sheet
     generates `ApprovalKeys`, posts the challenge, then opens the sealed name when
     `approval.named` fetches it.
  3. It shows the name, the time and the code: "Does it show 123 456? Only approve if it does."
     [Approve] calls `finish_approval`; [Cancel] deletes the request.
  4. The sheet never appears on a locked device (it can't wrap the key).
- **Wait to be approved** (the new device).
  1. The sign-in flow saved a pending item; the page shows "Approve this Mac on another device".
  2. It creates `ApprovalKeys`, posts the request, shows "Waiting for another device…", and polls
     every 2 s.
  3. On the challenge it shows the code and "It should match the code on your other device"; when
     the name arrives it posts its own sealed name.
  4. On approval it computes the code, compares it with what it shows, unwraps and checks
     `key_id`, and returns an unlocked candidate to the app's main flow. Only after Resume, Add or
     Replace consent does it save `sync-account`, delete the pending item, bind and sync; Cancel
     logs the new token out and leaves the library's existing notes, binding and Keychain items
     alone.
  5. If `other_devices` is 0, or two minutes pass, it offers [Use your recovery key] and [Cancel].
  6. The code never matches: it says so, offers Cancel, and never saves anything.
- **Sign out.** First cancel and join the running sync (fix 2). Then delete
  `sync-account:<library>`, any `sync-pending:<library>` and any `sync-recovery-key:<library>` (H3). Then try `logout` with a 5 s
  timeout, even when the Keychain refused a deletion (A14; that refusal is reported after). On
  failure, add `{server, token}` to `sync-logout:<library>`. `flush_pending_logouts` retries those
  at launch and before each sync until the server answers 204 or 401. Every read-modify-write of
  the queue holds one lock in the process, never across a request: a flush reads the queue, sends
  the logouts, then takes out only those that went, so a token queued meanwhile stays (sync 9).

**A new recovery key on its way** (sec-client 1). Sign-up, recovery, a password change and "New
recovery key" each choose the key before building the request and save it in
`sync-recovery-key:<library>` marked `unconfirmed` before the request leaves; nothing is sent if
the Keychain refuses. Then:

- The server says yes: the item is marked confirmed.
- Every attempt was refused (a 4xx) or never left the Mac: the item goes back to what it was.
- Any attempt's answer was lost, garbled or a 5xx: the item stays, and `RotationError.recovery_key`
  carries it, so the app shows it with a note that the server may not have taken it. This is
  sticky across the retries (review-verify 1): when the first attempt's answer is lost and a retry then
  never leaves the Mac (offline) or gets a 429, the first one may still have been carried out, so
  the key is kept, not dropped.
- Such a key is checked once (review-verify 6): `recovery_key_held` sends `recover/start` with the
  key's proof for the item's username. 200: the server holds it, so the item is marked confirmed.
  401 `invalid_credentials`: it doesn't, so the change never happened and the item is deleted
  (while it still holds that key and is still unconfirmed). Anything else (offline, a 429, a 5xx)
  leaves the item as it is. The app asks once per key shown, after the task that showed it and the
  launch check, and never again in that launch, since `recover/start` counts against `recover-ip`
  (10 an hour per address); a key still unconfirmed is asked about again at the next launch.

While the item holds an unsaved key for the same account, the next rotation sets that same key
again (except the key a recovery proves), so a retried or repeated change can't leave the server
with a key the Mac no longer has. Another account's unsaved key refuses with `UnsavedRecoveryKey`.

**Requests.** A request whose answer is lost is sent again with the identical body, only on routes
the server answers idempotently, after about 1 s and then 4 s (`Backend::retry_pause`). The HTTP
backend gives each request, connecting included, 30 s on the auth and account routes, 5 s on
`logout`, and 300 s on pushes, changes and files (A7). Request bodies and answers that hold keys,
proofs or tokens are zeroized when dropped, and their `Debug` shows `<redacted>`.

### Sync with encryption

The engine works on plaintext `SyncDoc`s through `Remote`, with `Sealed<W>` in between.

#### What a Mac remembers per document

The data-integrity review found one root problem. `synced_rev` was meant to mean "a version this
account's server actually had", but three paths broke that: binding with Add, a server-reported loss,
and an epoch re-merge. The M2 stale rule then turned real changes into silent overwrites or
deletions. Or the other way, a hostile server could clear the floor and bring deleted notes back. So
each document row now keeps separate facts, one per question, and no rule reads a fact meant for
another question:

| Column | Question it answers | Set by | Never changed by |
|---|---|---|---|
| `synced_rev` | Which revision does this account's server hold, as far as this Mac knows? It is the next push's `base_rev`; NULL means "push with no base". | A guarded write that learned it from the server: an accepted push (`mark_synced`), or a server version that was applied, unchanged, stale, an echo or a conflict (`apply_remote`). Cleared by `Lost` (`mark_unsynced`), the re-merge's missing pass (`mark_missing_unsynced`) and binding (`reset_sync`). | Local writes |
| `synced_gen` | The replay floor: the highest generation this Mac has seen on this account for the document. | Raised, never lowered, by every write that sets `synced_rev` from the server. Reset to 0 only when binding to an account (`reset_sync`); `replace_library` removes the rows. | `Lost`, `mark_missing_unsynced`, an epoch change, local writes |
| `sent_rev` | Which revision is in a push whose answer this Mac hasn't processed yet? | `mark_sent`, in one guarded write right before each push request. Cleared by every write that sets or clears `synced_rev`. | Local writes |

A document is pending when `synced_rev IS NULL OR synced_rev != rev`, as before.

A tombstone that `reset_sync` leaves behind (Add, a fresh bind, unbind) gets `synced_rev = rev` and
`synced_gen = 0`. That pair means "never on this account". The tombstone is never pushed, the floor
doesn't protect it from the account's versions, a re-merge doesn't count it as missing, `verify`
skips it, and a note re-created over it is pushed with no base.

`apply_remote` decides in this order:

1. No local row: apply. Same revision: mark it synced (and keep bytes that arrived with it).
   `synced_rev` already equals the remote revision: unchanged.
2. **Our own echo.** The remote revision equals `sent_rev`: the server took our push and the answer
   was lost. `synced_rev` becomes that revision, the newer local revision stays pending on top, and
   it is `Unchanged`. It is never a conflict, so an API merge that kept `updatedAt` isn't reverted by
   its own echo, and an editor save isn't falsely flagged.
3. **The floor (M2).** A remote revision whose generation is at or below `synced_gen` never reverts
   the Mac. It is `Stale`: the local version is rebased above it and pushed again. `Lost`, the
   re-merge's missing pass and an epoch change keep `synced_gen`. So a server that claims it lost a
   document, or rotates its epoch, and then answers with an old sealed version still gets `Stale`. It
   can't bring back a deleted note or roll back an edit, however many such tricks it plays in one sync.
4. **Nothing is discarded silently.** A stale remote version can be live, differ from the local
   content, and have a content hash this Mac doesn't hold (neither current nor in the document's
   history). Then it is kept as a snapshot and the document is listed in `conflicted`
   (`Stale { conflict: true }`, counted in `conflicts_local`). After a restore, generations from
   different Macs no longer compare: another Mac's newer offline edit can fall below this Mac's floor.
   It loses to the local version, as any conflict loser does, and is kept and flagged. A replayed
   version this Mac already holds adds nothing and flags nothing.

   **Except over a deletion** (review-verify 2). When the local side is a tombstone, a live stale version
   whose content this Mac never held isn't stale. It takes the normal "edit beats deletion" path:
   it is stored (a file's bytes with it), `synced_rev` becomes its revision, and it is `RemoteWon`,
   counted in `conflicts_remote` and flagged. So after a restore, a Mac's deletion never pushes a
   tombstone over another Mac's edit it never saw, and never deletes that file's bytes on the
   server or on the other Mac. The outcome no longer depends on which Mac syncs first, or on how
   many edits each made, and the flag always names a note that is there to open. A stale conflict
   is only ever raised over live local content.

   A deletion always puts the version it deletes into history, even from the UI right after an
   edit, whose snapshots are otherwise at most one per 5 minutes. That history is how a replay of
   content this Mac held, the version it deleted or any earlier one still in history, is told from
   another Mac's edit: a replay stays `Stale`, and the deletion stands.

   The trade-off: a hostile server can now bring a deleted note back on a Mac by replaying a version
   another Mac wrote that this Mac never pulled (for instance an intermediate edit it skipped). It
   is real content of the account, flagged where it lands, and of the same class as the existing
   "a Mac that never saw a newer version" risk ("Threat model › Residual risks", stale reads). The
   user-facing rule, "an edit always beats a deletion", wins over that.

   A residual stays on the other side, in the re-merge: a Mac whose clean edit the restore dropped
   takes a deletion that another Mac pushed after the restore without ever seeing that edit,
   because it can't tell that deletion from one made after seeing it. The edit is kept in the
   document's history and the document is flagged, but nothing in the app opens a deleted note's
   history; a backup has it. The Account page names and opens only a conflicted note that is still
   there.
5. Otherwise the conflict rules apply as before. A clean local copy takes the remote version. A
   changed one keeps the newer `updatedAt`, and an edit beats a deletion. The loser is kept as a
   snapshot and flagged.

**Add** (review-sync 1, sec-client 5). The library's live documents get `synced_rev = NULL` and
`synced_gen = 0`, and are pushed with no base. Its tombstones are never on this account. The first
sync pulls first:

- The account's live copy of a note deleted on this Mac comes back: an edit always beats a deletion,
  however many edits either side made. For a file, its bytes come back with it, on the server and on
  every Mac.
- A note deleted in the account but live on this Mac goes back to the account.
- A note changed on both sides keeps both versions and is flagged.

#### In order

- **Start of a sync** (first page):
  - If `key_id` differs from the Mac's: `WrongKey`. Stop and write nothing.
  - If `epoch` differs from `sync.v2.epoch`, or `head_seq` is below the cursor, the server's copy
    was restored or rewound (M1). The Mac re-merges without forgetting what it synced (security
    re-check 4):
    1. `synced_rev` and `synced_gen` stay as they are. The cursor goes to 0, and the epoch becomes
       `remerging` until the re-merge finishes (`set_meta_if`, guarded).
    2. The engine pulls the whole feed through `apply_remote`, so the rules above still hold. A
       revision below the floor comes back `Stale`, and is rebased and pushed. A remote version that
       replaces a different live local version during the re-merge is also flagged in `conflicted`:
       after a restore the Mac can't tell whether it descends from what the Mac synced. The local one
       is kept as a snapshot.
    3. After the last page, `mark_missing_unsynced(present)` takes every document with a
       `synced_rev` (tombstones included, except those never on this account) that the feed didn't
       carry. The server lost it, so it is pushed again with `base_rev = null`, and its floor stays.
       Deletions are never lost.
    4. A normal push follows. Conflicts keep both versions.

    The report sets `remerged`, and the app says "The server's copy was restored from a backup; this
    Mac sent its changes again." The cursor is never just reset, and `reset_sync` is only for
    binding to a different account. `head_seq` below the cursor is a best-effort check: it misses a
    rewind once any Mac has pushed past this Mac's cursor again. A restore must rotate the epoch
    (`admin restore`).
- **Paging.** Continue while `page.more`; after applying a page, save `page.last_seq` with
  `set_meta_if` (guarded).
- **Undecryptable feed documents** (format `0x01` that fails to open) never stop the pull; the
  cursor moves past them. They are repaired automatically only when the Mac has a local document
  with that wire ID and the broken copy's wire revision equals `wire_rev(id, synced_rev)`: the
  server holds a damaged copy of exactly the version this Mac last synced. Then the local version
  gets a new revision one generation higher and is pushed with the broken revision as its base,
  which replaces the server copy (M3); `report.repaired` lists it. Any other undecryptable document
  is only skipped and listed in `report.skipped`; `kanso-sync verify --repair` can replace it from
  this Mac.
- **Too-new documents** (format above `0x01`) stop the pull before them, keeping the cursor. The
  report sets `too_new`, which gives the problem `UpdateKanso`. Pushing continues, and is never a
  repair.
- **Stale documents** (M2): `apply_remote` returns `Stale { new_rev, conflict }`. The remote version
  never reverts the Mac. The local version is rebased onto it like a local win: generation
  `max(local, remote) + 1`, with `synced_rev` set to the remote revision. The remote version is kept
  as a snapshot and flagged when rule 4 says so. The next push sends it. `report.stale` lists the
  stale documents and the cursor moves on.
- **Files.** Opened bytes must match the document's `md5digest`, or they count in `files_missing`.
  After the pull, the engine fetches again up to `SyncOptions::refetch_files` (default 8) live file
  documents whose bytes this Mac lacks, meaning no blob matches the document's `md5digest`:
  - It goes in ID order from where the last sync stopped (`sync.v2.refetch_after`), skipping files
    this sync already tried.
  - Bytes are kept only when they open and match the `md5digest`.
  - They are stored in a guarded write that also checks the document still has the revision they
    were fetched for (`Store::fill_blob`).
  - A hit counts in `files_down`, a miss in `files_missing`.
  - It never stops the sync or the push after it (review-verify 4). A request that fails (offline,
    a 5xx, an answer over the cap) counts as a miss and ends this sync's refetching, since the
    rest would likely fail too and each can take minutes. Only a 429 `session-account` stops the
    sync, as on every route (`BackOff`).
  - A file it missed (gone from the server, not matching its `md5digest`, unreadable, or a failed
    request) waits a day before it is asked for again: `sync.v2.refetch_missed` maps its ID to that
    time. Bytes gone for good, such as imported attachments without them, cost one request a day,
    not eight a sync. At most 512 files wait, and past that the ones due soonest are forgotten.
    A wait already due, or more than a day and an hour ahead (saved while the clock ran ahead), is
    dropped. The pull's own fetch of a new version's bytes doesn't wait.
- **Push.** Before each push request, `mark_sent` records the batch's revisions (guarded). Results:
  - `Conflict` with a current version that opens: applied by the rules above. A local win gets the bumped generation and goes in the next round.
  - `Lost` (`current: null` with a `base_rev`): `mark_unsynced` (the floor stays), then pushed again with `base_rev = null` in the next round (M1).
  - `Behind`: kept pending; the engine pulls before the next round.
  - `Unreadable` (old format): repaired, as above. `Unreadable` (newer format): kept pending, and the report sets `too_new`.

  None of these lands in `rejected`, which goes away.
- **Quota** (review-sync 6). The Worker counts `writes-account` before it runs a batch, so a refused
  batch still costs the whole day's budget for its documents. A 413 `quota_exceeded` is therefore
  remembered, per family, in a guarded meta write:
  - Document quotas (`docs`, `doc_bytes`, and `server` from a push) go in `sync.v2.quota.docs`.
  - File quotas (`files`, `file_bytes`, and `server` from an upload) go in `sync.v2.quota.files`.

  Each entry is `<kind> <used> <limit> <at_ms> <level>`. It holds documents back from the push
  until usage drops:
  - `new` holds what the server doesn't have yet: documents with no `synced_rev` (new IDs and
    never-synced tombstones), and uploads of new files.
  - `edits` also holds live documents the server has (and their uploads). The engine moves to it when
    a batch with no new documents is refused.
  - `all` holds the whole family. It applies when even a batch of tombstones is refused.

  Tombstones and edits of documents the server has keep going, so a user can delete their way back
  under quota. A refused batch is retried at once, without what the new level holds; that is the old
  "tombstones only" retry, generalized.

  The first time a sync with an entry would hold a document back, the engine reads
  `GET /v2/account` (once a sync). Usage dropped when the kind's usage is below the `used` the 413
  reported, or its quota rose above `limit`. A `server` entry can't be measured there, so it retries
  after an hour. When usage dropped, held documents go out again, within the headroom, each charged
  once a sync:
  - live documents up to `quota.docs − usage.docs`: every live document is charged one potential
    slot, because an edit based on a revision the server has since tombstoned revives it and grows
    `docs` just like a new one (a tombstone of a document the server has costs nothing);
  - documents the level holds, at their estimated sealed size (an upper bound) plus 512 each, up to
    the `doc_bytes` headroom;
  - uploads the level holds, at their size plus 1,088 each, up to the file-bytes headroom, and live
    ones up to the file-count headroom.

  If the request fails, the entry stays. A 413 later in the same sync takes no more from the room.
  A sync that ran to the end, held nothing back and got no 413 deletes the family's entry. A live
  file document held for any reason (its document, its upload, or the upload hold) keeps both
  entries, since its upload was never tried. While an entry is in place, `report.quota` stays set,
  so the app keeps saying "Account full". So writes only grow by what is actually sent: a refused
  batch is never sent again until usage drops, and then only what fits.
- **Pauses (429).** `report.paused_limit` keeps the 429's `limit`, so the app can tell the server's
  cap from the account's own.
  - `writes-account` or `uploads-account` stops pushing for this sync and sets
    `report.paused_until`, which gives the problem `Paused`.
  - `writes-global` does the same, and gives `ServerBusy`.
  - `uploads-global` on an upload doesn't stop the sync or block pulls. Uploads are held until
    `retry_after`, in `sync.v2.uploads_until` (guarded), along with the live file documents waiting
    for them. Everything else keeps syncing, and `report.uploads_held_until` says until when.
  - `session-account` (the per-account request limit on every session route, about a minute) on
    any sync request (changes, push, upload, download) is a short back-off, not a pause. The sync
    stops where it is with `SyncError::BackOff { until }`, `until` being now plus `retry_after`.
    Nothing after the last finished step is marked done: the cursor stays at the last page applied,
    and a document whose push or bytes didn't go stays pending. `problem` gives `BackOff { until }`,
    whose schedule waits until then.
  - **Bounded waits** (review-verify 5). Every time a 429 sets (a pause, the upload hold, a back-off,
    and so the app's schedule) is now plus `retry_after`, clamped to 24 hours: a server's answer,
    however large, never holds a Mac back longer than a day. The saved times are read back with the
    clock they were saved by, which may have run ahead: a quota entry whose `at_ms`, or an upload
    hold whose time, is more than 25 hours ahead of now is deleted when read, so nothing waits on
    it (before, only a re-bind cleared it). A deleted quota entry costs at most one more refused
    request, which saves it again with the right time.
- **The guard** (fix 2 / M6, security re-check 3). Every `apply_remote`, `mark_synced`,
  `mark_sent`, `mark_unsynced`, `mark_missing_unsynced`, `fill_blob` and meta write (cursor, epoch,
  quota entries, holds) carries `SyncGuard { expect: &[("sync.v2.origin", session.origin),
  ("sync.v2.account_id", session.account_id)] }` (`binding::guard_pairs`). The store checks every
  pair inside the same write transaction and fails with `StoreError::GuardFailed` otherwise. The
  engine then stops with `SyncError::Cancelled`. `check_bound` also runs right before every push and
  upload.

Every remote write still goes through `apply_remote`, which snapshots the version it replaces.

### Problems

| `SyncProblem` | From | Sidebar | Settings help | Schedule |
|---|---|---|---|---|
| `SignedOut(reason)` | 401 | "Signed out" | Reason-specific: "Your password was changed on another Mac." / "This Mac was signed out from another Mac." / "Your account was recovered with its recovery key." / "Signed out to make room for another Mac." / "Signed out after 180 days without syncing." Any other reason: "This Mac was signed out." Each ends "Sign in again to keep syncing." | Stops |
| `AccountDisabled` | 403 | "Account turned off" | "This account is turned off on the server. Ask the server's owner." | Stops; Sync now retries |
| `Paused { until }` | 429 `writes-account` or `uploads-account` | "Sync paused" | "This account made a lot of changes today. Sync resumes at *time*." | Waits until then |
| `ServerBusy { until }` | 429 `writes-global` | "Server busy" | "The server has taken all the changes it can for today. Sync resumes at *time*." | Waits until then |
| `BackOff { until }` | 429 `session-account` (`SyncError::BackOff`) | "Syncing shortly" | "This Mac sent the server a lot of requests at once. Sync resumes in a moment." | Waits until then (about `retry_after`) |
| `QuotaExceeded(kind)` | report | "Account full" | "Your account is full (*which*: notes and notebooks / attachments / number of attachments / number of items). Delete something or ask the server's owner for more space." | Keeps running |
| `ServerFull` | report (`server`) | "Server full" | "The server is out of space. Ask its owner." | Keeps running |
| `Skipped { count }` | report (`skipped`: undecryptable documents not repaired) | "Some items skipped" | "*N* items on the server couldn't be read. They were skipped; your copies here are unchanged." | Keeps running |
| `UpdateKanso` | report (too new) | "Update Kanso" | "Another Mac uses a newer Kanso. Update this one to keep syncing." | Stops; Sync now retries |
| `WrongKey` | key ID | "Keys don't match" | "The server's account keys don't match this Mac's. Sync is stopped; your notes are safe here. Sign out and in again." | Stops |
| `ServerOutdated` | 404 health | "Server needs an update" | "That server doesn't run this version of Kanso sync." | Stops; Sync now retries |
| `Offline` | transport | "Offline" | "Can't reach the server. Your notes are safe on this Mac." | Keeps running |
| `Other` | anything else | "Couldn't sync" | "Something went wrong. Try Sync now." | Keeps running |

`report_problem` picks, in this order of precedence, `UpdateKanso`, `Paused` or `ServerBusy` (from
`paused_until`: `ServerBusy` when `paused_limit` ends with `-global`), `ServerFull`, `QuotaExceeded`,
`Skipped`. An `uploads-global` hold sets `uploads_held_until` and no problem, so the schedule keeps
pulling. Stale and repaired documents aren't
problems: the Mac fixed them. `SyncState::Synced` gains `problem: Option<SyncProblem>`, and `SyncState::Failed`
carries a `SyncProblem`. `SYNC_PROBLEMS` (sidebar) and `SYNC_PROBLEM_HELP` (settings) become
functions of `SyncProblem`, which removes substring matching.

### Keychain items

The service is `Kanso`. On macOS this is the Keychain, on Windows Credential Manager and on Linux
Secret Service through the existing `keyring` crate. Item names use the library ID, so data
directories never share secrets.

| Account (item name) | Value (JSON) | Lifetime |
|---|---|---|
| `sync-account:<library_id>` | `{"v":1,"server","origin","account_id","username","device_id","token","data_key","device_proof"?}` | Sign-up, sign-in, recovery or a completed approval until sign-out, deletion, a 401 or unbind. `device_proof` is present when a passkey or provider sign-in bound the session to its flow's device rather than this library's: the event socket and a link's or proof's start send it. It is zeroized and never printed; an item without it uses the library's own device. |
| `sync-pending:<library_id>` | `{"v":2,"server","origin","account_id","username","device_id","device_proof","token","key_id","epoch","evicted_device"?:{"id","sealed_name"?},"approval"?:{"request_id","requester_secret","expires_at","other_devices","approver_epk"?,"approver_device_id"?,"sealed_name"?}}` | From a passkey or provider sign-in that returned no usable wrap until the device is unlocked (then it becomes `sync-account` and this item goes), the request is cancelled, or the session ends. It never holds the data key, password, KEK or PRF output. The provider or passkey device proof and an in-flight approval's requester secret are zeroized and never printed. It is loaded by library ID and self-validated even when no config account/binding exists; a v1 record cannot resume password or approval and is ignored. |
| `sync-recovery-key:<library_id>` | `{"v":1,"origin","account_id","username","server","recovery_key","unconfirmed"}` (`server` and `unconfirmed` left out when empty or false) | From just before the request that sets the key leaves (sign-up, recovery, a password change, "New recovery key") until "I've saved it" or sign-out. A refused request puts back what was there. A confirmed key also goes when this Mac's session ends with `password_changed` or `recovered`: another device replaced it. While it exists, the Account page shows the key screen again. |
| `sync-logout:<library_id>` | `{"v":1,"pending":[{"server","token"}]}` | Until each logout succeeds |
| `sync-token` (legacy) | The old shared token | Never read or deleted by the new app; the owner removes it after the purge |

On load, the item's `(origin, account_id)` must equal the library's `(sync.v2.origin,
sync.v2.account_id)` and the config's account (security re-check 3). If it doesn't, the item is
ignored, and the Mac counts as signed out. The same pair decides everywhere else whether two
accounts are the same. `Session.server` and
`origin` always come from this item; `config.sync.server` only pre-fills the signed-out forms. The
data key stays in memory once loaded, and is zeroized on sign-out or quit.

### Library binding

**Meta keys** (H5). Only `unbind` touches the legacy `sync.last_seq`.

| Key | Holds |
|---|---|
| `sync.v2.origin` | The origin of the account's server |
| `sync.v2.account_id` | The account the library syncs with |
| `sync.v2.username` | Its username |
| `sync.v2.server` | The server URL |
| `sync.v2.last_seq` | The cursor |
| `sync.v2.epoch` | The epoch last seen (`remerging` while a re-merge runs) |
| `sync.v2.quota.docs` | A document quota hit still holding documents back: `<kind> <used> <limit> <at_ms> <level>` |
| `sync.v2.quota.files` | The same for file quotas |
| `sync.v2.uploads_until` | Uploads held until this time (ms) after an `uploads-global` 429 |
| `sync.v2.refetch_after` | The file ID where the last sync's search for missing bytes stopped |
| `sync.v2.refetch_missed` | The files a refetch missed, as a JSON object of file ID to the time (ms) each may be asked for again |
| `sync.v2.library_id` | Device identity |
| `sync.v2.device_secret` | Device identity |
| `sync.v2.hardware` | Device identity |

**kanso-store APIs** (frozen in step 1):

```rust
/// Every (meta key, value) pair must hold; the engine passes origin and account ID.
pub struct SyncGuard<'a> { pub expect: &'a [(&'a str, &'a str)] }
pub enum ApplyOutcome { Applied, Unchanged, RemoteWon, LocalWon { new_rev: String }, Stale { new_rev: String, conflict: bool } }
/// One row's sync facts (see "Sync with encryption › What a Mac remembers per document").
pub struct SyncState { pub id: String, pub rev: String, pub deleted: bool, pub synced_rev: Option<String>,
                       pub synced_gen: u64, pub sent_rev: Option<String> }
impl SyncState { pub fn pending(&self) -> bool; pub fn never_on_account(&self) -> bool; }
impl Store {
    /// One transaction: live documents get synced_rev = NULL, tombstones get synced_rev = rev
    /// and synced_gen = 0 (never on this account: never pushed, never protected), every floor
    /// and sent_rev is cleared, and the meta entries are written (Some) or removed (None).
    pub fn reset_sync(&self, meta: &[(&str, Option<&str>)]) -> Result<()>;
    /// One transaction: removes every document, note-tag row, blob, revision, FTS row and archive
    /// row, then writes the meta entries. Broadcasts a deletion per document (Origin::Sync).
    pub fn replace_library(&self, meta: &[(&str, Option<&str>)]) -> Result<usize>;
    pub fn apply_remote(&self, doc: &SyncDoc, attachment: Option<Attachment>, guard: Option<&SyncGuard>) -> Result<ApplyOutcome>;
    pub fn mark_synced(&self, id: &str, rev: &str, guard: Option<&SyncGuard>) -> Result<()>;
    /// Right before a push: each (id, rev) is in flight (sent_rev).
    pub fn mark_sent(&self, docs: &[(&str, &str)], guard: Option<&SyncGuard>) -> Result<()>;
    /// synced_rev = NULL; the floor stays.
    pub fn mark_unsynced(&self, id: &str, guard: Option<&SyncGuard>) -> Result<()>;
    /// After a re-merge's full pull: documents with a synced_rev (tombstones included, not those
    /// never on this account) whose IDs aren't in `present` get synced_rev = NULL, so they are
    /// pushed with base_rev = null. The floor stays.
    pub fn mark_missing_unsynced(&self, present: &HashSet<String>, guard: Option<&SyncGuard>) -> Result<Vec<String>>;
    pub fn sync_states(&self) -> Result<Vec<SyncState>>;
    /// Live file documents with no blob matching their md5digest: (id, rev, md5), in ID order after `after`.
    pub fn files_without_bytes(&self, after: &str, limit: usize) -> Result<Vec<(String, String, String)>>;
    /// Stores refetched bytes iff `id` is still live at `rev` and they match its md5digest.
    pub fn fill_blob(&self, id: &str, rev: &str, blob: Attachment, guard: Option<&SyncGuard>) -> Result<bool>;
    pub fn set_meta_if(&self, key: &str, value: &str, guard: &SyncGuard) -> Result<()>;
    /// Removes a meta entry (a quota entry, the upload hold) while `guard` holds.
    pub fn remove_meta_if(&self, key: &str, guard: &SyncGuard) -> Result<()>;
    pub fn pending_count(&self) -> Result<usize>;
}
```

`StoreError` gains `GuardFailed`.

`binding::local_library(store, origin, account_id)` returns one of these, where the same account
means the same `(origin, account_id)`:

- `Empty`: no live documents.
- `Same`
- `Unbound { notes, unsynced }`
- `Other { username, notes, unsynced }`

`unsynced` counts documents with changes that never reached any server (`pending_count`). The plan
for each flow and library:

| Flow | Library | What happens |
|---|---|---|
| Sign up | Empty or unbound | Bind fresh, then upload. The form says "Your *N* notes on this Mac will be encrypted and uploaded." |
| Sign up | Bound to another account | Before sign-up: "This Mac's notes came from *alex*. Upload them to the new account?" [Upload to new account] [Cancel]. Upload means merge. |
| Sign in or recover | Empty | Bind fresh, then pull. |
| Sign in or recover | Same | Resume. |
| Sign in or recover | Unbound with notes, or bound to another account | After authenticating, before saving anything (below). Cancel logs the new session out (`sign_out_locked`, or `discard_session` when it is unlocked) and deletes its pending item; the library's other Keychain items, which may be the signed-in account's or hold a recovery key still on screen, stay. |

The question in the last row:

- Title: "This Mac already has notes".
- Body:
  - "*N* notes on this Mac (*U* with changes that were never synced) aren't in *ngoni*'s account."
  - Or, when the library came from another account: "came from *alex*".
- Choices:
  - [Add them to *ngoni*]: "They're encrypted and uploaded. A note deleted in *ngoni*'s account but
    still on this Mac comes back, and a note changed on both sides keeps both versions."
  - [Replace them with *ngoni*'s notes]: "Kanso saves a backup of this Mac's library first, then
    downloads *ngoni*'s notes."
  - [Cancel]

What the choices do:

- **Fresh and merge (add).** `reset_sync` with the origin, account, username, server, `last_seq = 0`,
  and the epoch, quota entries, upload hold and refetch position removed. The first sync pulls first:
  identical revisions become `Unchanged`; the account's live copy of a note deleted here comes back
  (the tombstone was never on this account); the rest resolve as conflicts with both versions kept.
- **Replace.**
  1. `Store::backup_now(paths::backups_dir(data_dir), now)`: the same manual backup as Settings ›
     Back up now, outside the daily rotation. The path is shown with "Show in Finder".
  2. `replace_library` with the same meta, in one transaction (fix 3). A crash leaves either the old
     library or an empty one bound fresh, which pulls.
- **Unbind** (after deleting the account, and for rollback). `reset_sync` that removes the origin,
  account keys, cursor, server and epoch, keeps the device identity, and sets the legacy `sync.last_seq =
  0`. It also deletes `sync-account:<library>`.

**Order and cancellation.** Every flow that calls `reset_sync`, `replace_library` or `unbind`, or
touches the Keychain item, first cancels the running sync and joins its thread (fix 2). On sign-in:

1. Answer the question, if there is one.
2. Cancel and join the running sync. For Replace, every open editor saves first, so the backup has
   its edits; editor saves then wait and the local API (with MCP) stops until the bind ends (A10).
   An editor with edits typed meanwhile stays on its note, saves them afterwards, and asks about
   them instead when the Replace took the note away.
3. Save the Keychain item (A1, sync 7): a library is never bound to an account whose item is
   missing.
4. Bind. If it fails, the Keychain item is put back as it was (gone, or the previous account's),
   the new session is logged out, and a backup Replace made first is still shown.
5. Write the config (`account`, `enabled = true`).
6. Start sync.

A chain of tasks that stopped the running sync and leaves the same session signed in (it failed,
or its question was cancelled) starts an automatic sync again once its last task ends (A8). A sync
asked for while another runs, or while that one is being stopped, runs right after it: as Sync now
if any of the requests was, otherwise as an automatic sync that respects the schedule's hold
(A9, sync 8).

Startup counts the Mac as signed in only when the config, the Keychain item and the binding agree.

### Command line

Run `cargo run --release -p kanso-sync -- <command>` from the deployed commit. Each command takes
the data-directory lock, so quit Kanso first. macOS asks once to let `kanso-sync` read the
`sync-account` item.

```text
status   --data-dir DIR                             account, server, device, cursor, pending changes
sync     --data-dir DIR [--full]                    sync once with this library's session
verify   --data-dir DIR [--repair] [--legacy M.json --out R.json]
estimate --data-dir DIR                             charged document and file bytes this library needs
unbind   --data-dir DIR                             sign out locally and reset bookkeeping for the old app
```

`verify` pulls the whole feed into memory without applying it, then decrypts every document and
live file. It compares them with the library: every local document whose `synced_rev == rev` must
be on the server at that revision with equal fields, and every live file's bytes must match. It
prints the counts (matched, missing, extra, different, undecryptable, still pending), and exits 0
only when nothing is missing, different, undecryptable or pending. With `--repair`, it pushes this
Mac's version over each undecryptable server copy of a document the Mac has, with a new
generation, and counts them as repaired.

With `--legacy`, it also checks each row of the manifest from `admin legacy-export` (H4):

- **A live legacy document** must be in the library, either at that revision or at a higher
  generation, and must have passed the checks above.
- **A live legacy file** must have a local blob with the manifest's `md5digest`, and must be on the
  encrypted server.

It then writes `R.json` (`{"manifest_sha256", "account_id", "ok", "rows", "failed"}`).

**The manifest hash** (security re-check 6) covers the rows only, never `exported_at`. Sort the rows
by `id` (bytewise UTF-8); write each as `id TAB rev TAB ("1" if deleted, else "0") TAB (md5digest,
or nothing when null) LF`; the hash is SHA-256 of those bytes as 64 lowercase hex digits
(`verify::manifest_sha256`). `purge-legacy` recomputes it the same way from a fresh export. Test
vector: the rows `["note:b","2-bb",false,null]`, `["file:a","1-aa",false,
"d41d8cd98f00b204e9800998ecf8427e"]` and `["tag:c","3-cc",true,null]` hash to
`a0dbdfeedc42d0d264500b4faad3718aa823528946c653f34e62b2be42575b92`; no rows hash to SHA-256 of the
empty string.

## App

### Config (`config.rs`, `CONFIG_VERSION = 2`)

```rust
pub struct SyncSettings {
    pub enabled: bool,                    // set true on sign-up/sign-in; sync only runs while signed in
    pub interval_secs: u64,
    pub server: Option<String>,           // forms only; None = DEFAULT_SERVER
    pub account: Option<AccountRef>,      // Some while signed in
    pub endpoint: Option<String>,         // legacy, preserved untouched so a rolled-back app finds it
}
pub struct AccountRef { pub origin: String, pub id: String, pub username: String }
```

Startup compares `(origin, id)` with the Keychain item and the binding (security re-check 3).

The migration to version 2 copies `endpoint` into `server` when it isn't the default, keeps
`endpoint` and `enabled` as they are, and leaves `account` as `None`. The owner's Mac therefore
starts signed out with its notes intact. Older app builds ignore the new fields.

### Services (`services.rs`)

- `TokenState` goes. `AccountState` is one of `SignedOut`, `SignedIn { username }`, `SignedInLocked
  { username }` (a session without the data key, waiting for approval, a password or the recovery
  key), `SessionEnded { username, reason }` or `Busy(Step)`.
- `Services` holds:
  - `Arc<Session>` once loaded;
  - `Arc<dyn Backend>`: `HttpBackend` in `main.rs`, `MemoryServer` in tests;
  - `Arc<dyn SecretStore>`;
  - the running sync's `SyncHandle { cancel, thread }`.
- **Account tasks** run on a named `std::thread` and send `Step` updates and the result over an
  `async_channel`. Any task that re-binds or signs out cancels and joins `SyncHandle` first, on that
  background thread, with "Stopping sync…" as its step.
- On `SignedOut(reason)`, the app deletes the Keychain item, shows `SessionEnded`, and stops the
  schedule. With `password_changed` or `recovered`, a confirmed unsaved recovery key is deleted
  too, and leaves the key screen: the server no longer holds it (sec-client 3). An unconfirmed one
  stays, since it may be the only working key.
- **This Mac's own recovery** (review-verify 3). "Forgot your password?" while signed in ends this
  Mac's old session too, so a request still in flight on it (the Account page's storage and
  devices) gets a 401 before the recovery has signed in again: `unknown` once the new session
  replaced the device's row, or `recovered`. While a recovery task runs, such a 401 never deletes
  the unsaved recovery key, which is the recovery's own new one. And whatever the reason, the
  session item is deleted only while it still holds the ended session
  (`keychain::delete_ended_session`), so a session the recovery saved meanwhile stays for the next
  launch.
- **The launch check** (`restore`) reads the Keychain on a background thread. Account tasks wait for
  it (`restoring`), and its answer is dropped if a session appeared, a task started or the config's
  account changed meanwhile, so it never overwrites a newer state (A4). Pending logouts are retried
  after it answers. An unsaved recovery key shows whatever the account.
- **Writes held** (`writes_held`, `hold_writes`, `release_writes`): around a Replace, as in Library
  binding › Order and cancellation. The local API starts again only if it ran.

### Account page (Settings › Account)

`SettingsPage::Sync` becomes `SettingsPage::Account`: title "Account", a person icon, id
`settings-account`, between Appearance and Local API. The action `OpenSyncSettings` becomes
`OpenAccountSettings`. The page lives in a new `account_page.rs`, with its forms as `InputState`
entities.

**Signed out.** The status row reads "Not signed in" (quiet), with the detail "Sync your notes
between Macs, end-to-end encrypted. Your notes stay on this Mac either way." Below it: [Create
account] [Sign in].

- **Sign in.** The methods the server advertises, in this order, then the username form:
  - [Continue with a passkey] — only when health advertises `passkeys` and the app can run the
    platform ceremony itself: the native sheet on macOS (for its associated domain) or the Win32
    WebAuthn sheet on Windows. A device with no passkey says so and the page stays.
  - [Continue with a security key] — Linux only, when health advertises
    `passkeys.security_key` and the app has enumerated a compatible CTAP2 authenticator. It asks
    for the username first, then runs the libfido2 ceremony. Linux never shows a platform-passkey
    button. With no key attached, the page says "Connect a FIDO2 security key to use one"; a key
    without `hmac-secret` can sign in but not unlock a new device, and the app says so.
  - [Continue with Apple], [Continue with Google], [Continue with GitHub] — only the advertised
    ones, with the provider's own button rules.
  - Below the methods, a rule with "or", then Username and Password with their labels above them
    and "Forgot your password?" beside the Password label, then [Sign in] across the column.
  - "Forgot your password?" switches to Username, Recovery key, New password, Confirm and
    [Set new password].
  - The server row shows "sync.kansoapp.app (default)" or the custom URL, with "Use a different
    server…" as today. Changing the server asks it for its methods again; a method it doesn't
    advertise is not shown.
- **Create account.** The same method list above a username form:
  - [Create with a passkey] — asks for a username (suggested), registers the passkey, and shows
    whether it can unlock.
  - [Create with a security key] — Linux's equivalent, with the same username-first form and
    libfido2 ceremony.
  - [Create with Apple] / [Google] / [GitHub] — completes the provider flow, then asks for a
    username (suggested, with "You can change this before creating the account").
  - [Create with a username] — Username, Password, Confirm password, with the hints as today,
    shown in place of the method list; Cancel there returns to the list. A server that offers
    only passwords opens this form directly.

  Every signed-out form is one left-aligned column, 360 points wide, headed "Sign in to Kanso",
  "Create your account", "Reset your password" or (after a provider) "Choose a username": the
  method buttons at the column's full width, the fields with labels above and hints below, the
  primary button across the column, then a quiet Cancel ("Back to sign in" from recovery, "Other
  ways to create an account" from the username form), and the server as a footnote with
  [Change…]. The status row shows only on the first page and after a session ends.
  - Server as today.

  The social and passkey forms then show "Your *N* notes on this Mac will be encrypted and uploaded.
  You'll get a recovery key instead of a password reset." and [Create account]. When
  `server_info` isn't `Open`, every create button is disabled and the line reads "This server isn't
  taking new accounts." `server_info` is asked again each time the form opens, one question per
  server at a time, and an answer counts only while the form still shows that server (A6).
- **Session ended.** The sign-in page, prefilled with the username, under the reason's sentence from
  the problems table.
- **While a task runs.** Inputs are disabled, and the button spins with the step: "Stopping sync…",
  "Securing your keys…", "Contacting the server…", "Waiting for your other device…", "Unlocking…",
  "Saving…". The password fields are cleared as soon as their task starts.

**Recovery key screen,** after sign-up, recovery and "New recovery key":

- Title "Save your recovery key". Text: "If you forget your password, this key and your username are
  the only way back into your account. Kanso and the server's owner can't reset it. Anyone with it
  can take over your account and read your notes, so keep it somewhere safe, like a password manager
  or on paper. Each time it's used, Kanso gives you a new one."
- The key in large monospace.
- **[Copy]** goes through a small macOS helper, `clipboard::copy_secret`, built on `NSPasteboard`.
  It calls `prepareForNewContentsWithOptions(NSPasteboardContentsCurrentHostOnly)` so Universal
  Clipboard doesn't send the key to other devices. It adds the `org.nspasteboard.ConcealedType` and
  `org.nspasteboard.TransientType` markers, and clears the pasteboard after 60 s if its change count
  is still ours. Quitting sooner clears it too, on the same condition
  (`clipboard::clear_secret_on_quit`, from `on_app_quit`). It notifies "Copied. It leaves the
  clipboard in a minute."
- **[Save as file…]** opens a save panel suggesting `Kanso recovery key.txt` and writes a 0600 file
  holding the key, username, server, date and warning. It never overwrites silently. If the chosen
  folder syncs to iCloud (`NSURLIsUbiquitousItemKey`, or a path under `~/Library/Mobile
  Documents/`), it asks first: "This folder syncs to iCloud. Save the key there anyway?"
- **"I've saved my recovery key"** enables [Done], which deletes `sync-recovery-key:<library>` if it
  still holds that key.
- **A note over the key**, in the warning colour, when it needs one: after a password change, "Your
  password changed, and so did your recovery key: the one you saved no longer works." When the
  answer was lost, that Kanso couldn't tell yet whether the server took the key (or created the
  account, or changed the password), to save it, and, except after a sign-up, to keep the old
  recovery key too until Kanso confirms which one works. When the account was created or the
  password changed but this Mac couldn't finish signing in, that sentence, with "Sign in with your
  password" (or the new password).
- **Checking an unconfirmed key** (review-verify 6). While the screen shows a key Kanso couldn't tell
  the server took, the app asks the server once (`TaskKind::CheckKey`, above). If the server holds
  it, the note becomes "The server took this key: it's your recovery key now, and the old one no
  longer works." (or says the account was created, or the password changed). If it doesn't, the
  key and its Keychain item go, and a notification, in place of [Show key], says nothing changed:
  "The server didn't take the new recovery key, so nothing changed: your old one still works." (or
  that the account wasn't created, or the password wasn't changed). If the ask fails, the note
  stays as it is until the next launch asks again.
- **Elsewhere.** A key that arrives while Settings › Account isn't showing (from a task, a failure
  carrying one, or the launch check) raises the notification "Save your new recovery key. It's on
  the Account page." with [Show key]; a second key replaces it.

**Signed in.** Sections:

- **Status row:** as today, with [Sync now].
- **Account:**
  - Username and server.
  - Storage, from `usage`, for all kinds of item: "850 items · 3.1 MB of 100 MB · attachments 14 MB
    of 500 MB (15 of 10,000)".
- **Sync:** Sync automatically and Sync every.
- **Sign-in methods:**
  - Password: "Password set" with [Change password…], or "No password" with [Set a password…].
  - Linked: one row per provider, "Apple · linked 12 Oct", with [Unlink]. [Link a provider…] lists
    the advertised providers that aren't linked.
  - Passkeys: one row per credential, "Passkey · added 12 Oct · last used 2 hours ago · can unlock"
    (or "signs in only"), with [Remove]. The title names the manager when the AAGUID is a known
    one ("Passkey in iCloud Keychain", "Passkey in 1Password"…). "Last used" is to the hour but
    never earlier than "added". [Add a passkey…] registers another; on a Mac it isn't offered while
    the account has an iCloud Keychain passkey, which that Mac already has, and the sheet would
    refuse a second.
  - A quiet line: "Your recovery key still works if you lose these." and, when only one method
    remains, "This is your last way to sign in."
  - Every button here asks for a fresh proof first (a passkey sheet, the password, or the provider),
    then does the change. Unlinking or removing the last method asks about the recovery key and
    sends `recovery_auth_key` when the user confirms it is saved.
- **Devices:** active devices with their opened names ("A Mac" while unnamed), "This Mac", and "Last
  active 2 hours ago". Other devices have [Sign out], which confirms: "Sign out *name*? It stops
  syncing until someone signs in on it again. Its notes stay on it." Afterwards a notification
  says how it went: "Signed out *name*.", or the error's words (A5). A device the server no longer
  knows counts as signed out.
- **Security:**
  - Change password… (Current, New, Confirm), described as "Your other Macs sign out and ask for
    the new password. Your recovery key changes too: the one you saved stops working, and Kanso
    shows the new one to save." It leads to the key screen. The success screen also lists the
    passkeys that stay linked, with their dates, and [Remove them all] when the change is being made
    because something is wrong.
  - New recovery key… (Current password, or a proof when there is no password; a provider proof
    also asks for the current recovery key), which leads to the key screen.
  - Forgot your password? (Recovery key, New, Confirm). It says it will remove every passkey.
- **Conflict handling:** as today.
- **Footer:**
  - [Sign out]: "Sign out of *ngoni*? Notes stay on this Mac. Sync stops until you sign in again."
    With an unsaved recovery key, the question starts "Your new recovery key hasn't been saved yet;
    signing out deletes it." When the Keychain refuses to delete the sign-in, Kanso is signed out
    all the same and says "Signed out. Kanso couldn't remove this Mac's sign-in from the Keychain,
    but it won't use it again."
  - [Delete account…]: "This deletes *ngoni*'s encrypted notes, attachments and devices from the
    server. Notes on this Mac stay here, and other Macs keep their copies. The server's backups keep
    a copy for up to 30 days. This can't be undone." Fields: "Type *ngoni* to confirm" and the
    account's proof (Password, or a passkey sheet when there is no password; a provider proof also
    asks for the recovery key).

`dialogs.rs` gains three things:

- `form`: several fields, async submit, an inline error, and the dialog staying open while the task runs.
- `choose`: up to three described choices.
- `confirm_typed`: a confirm dialog that requires typing a word.

### Device approval in the app

A device that signed in but has no data key is `AccountState::SignedInLocked { username }`, not
`SignedIn`. The sidebar says "Waiting for approval" in the sync line; Settings › Account, the editor
and every sync action are behind the usual signed-out gate, and the engine never starts. The
pending Keychain item (`sync-pending:<library>`) is what survives a quit; on launch the app reads it
and shows the waiting page again, with the same request if it is still alive and a [Start again]
button if it isn't.

**On the new device** the page replaces the signed-out forms:

- Title "Approve this device".
- "Open Kanso on a device that's already signed in and approve this one."
- The code in large monospace, once the challenge arrives: "It should show **123 456**." Above it,
  while waiting: "Waiting for another device…" with a quiet spinner.
- [Use your recovery key] and [Cancel]. After two minutes, or at once when the request said there
  are no other devices, the text becomes "No other device answered. Use your recovery key to open
  this Mac." and [Use your recovery key] is the primary button.
- If the codes differ: "That code doesn't match. Don't approve on the other device. Cancel and try
  again." with [Cancel] only.

**On an already signed-in device** the request is a sheet, raised by `approval.created` on the
account event socket; launch, every Settings › Account open, foreground and every reconnect also do
one approvals `GET`:

- Title "A device wants to join *ngoni*".
- The device name from the sealed name ("Alex's MacBook Air", or "A Mac" until it arrives), the
  time ("asked at 14:02"), and "Does it show **123 456**? Only approve if it does."
- [Approve] and [Cancel]. Approve is disabled until the code is shown; the sheet never approves
  itself and never auto-approves.
- A second request while one is open queues behind it; the list is capped at three.
- The sheet doesn't appear on a locked device, and a session that can't unwrap the data key shows
  "Sign in again to approve devices" instead of a broken button.
- The code is announced as "one two three, four five six", never put in a notification, a log or a
  crash report, and cleared from the screen when the sheet closes.

The account event socket is opened when an unlocked account is signed in and reconnects with
bounded backoff. It carries only event types and approval IDs; the app does a light `GET` after each
event and after every connect. The fallbacks are launch, every Settings › Account open and
foreground. There is no 30-second timer, and nothing about approval uses a push service. An event
that arrives while the app is in the background queues the sheet; the code is still shown only in
the sheet.

### Elsewhere in the app

- **Sidebar footer.** The username and its initials when signed in, the macOS user name when
  signed out. The sync line reads "Sign in to sync" (quiet; clicking it opens Settings › Account),
  or one of the labels in the problems table, or today's states.
- **First-run welcome** (`empty_state.rs`, used from `editor.rs`). While signed out, a row reading
  "Already use Kanso on another Mac? Sign in to sync", which dispatches `ShowSignIn`.
- **Palette** (`commands.rs`). `Needs` gains `SignedIn` and `SignedOut`.

  | Id | Label | Needs | Runs |
  |---|---|---|---|
  | `settings-account` | Settings: Account | — | `OpenAccountSettings` |
  | `sign-in` | Sign in to sync… | `SignedOut` | `ShowSignIn` |
  | `create-account` | Create account… | `SignedOut` | `ShowCreateAccount` |
  | `change-password` | Change password… | `SignedIn` | `ChangePassword` |
  | `new-recovery-key` | New recovery key… | `SignedIn` | `NewRecoveryKey` |
  | `sign-out` | Sign out | `SignedIn` | `SignOut` |
  | `sync` | Sync now | `SignedIn` | `SyncNow` |
- **Menu.** "Sync Now" is disabled while signed out.

### Plain-language errors

| Cause | Words |
|---|---|
| `invalid_credentials` (sign-in) | "That username and password don't match." |
| `invalid_credentials` (recovery) | "That recovery key doesn't match this account." |
| `invalid_credentials` (current password) | "That isn't your current password." |
| Recovery-key checksum | "That recovery key has a typo. Check it and try again." |
| `username_taken` | "That username is taken. Try another." |
| `invalid_username` | "Use 3–32 lowercase letters, digits, - or _, starting and ending with a letter or digit." |
| `reserved_username` | "That username is reserved. Try another." |
| Password too short | "Use at least 12 characters. A few random words work well." |
| Password equals username | "Choose a password that isn't your username." |
| Confirm mismatch | "The passwords don't match." |
| `signup_closed` | `closed`: "This server isn't taking new accounts." `full`: "This server is full." `busy`: "This server has taken enough new accounts today. Try tomorrow." |
| `rate_limited` | "Too many attempts. Try again in *N* minutes." |
| `account_disabled` | "This account is turned off on the server. Ask the server's owner." |
| `identity_taken` | "That *Apple/Google/GitHub* account is already linked to another Kanso account." |
| `last_method` | "This is your only way to sign in. Add another, or confirm your recovery key." |
| `reauth_required` | "Confirm it's you to change how you sign in." |
| `flow_expired` | "That took too long. Try again." |
| `nonce_reused`, `provider_state` | "That sign-in didn't come from where it should. Try again." |
| `provider_denied` | "Sign-in was cancelled at *Apple/Google/GitHub*." |
| `provider_unavailable` | "*Apple/Google/GitHub* didn't answer. Try again." |
| `approval_expired` | "That approval request expired. Ask again." |
| `approval_used` | "That request was already answered. Ask again." |
| `self_approval` | "A device can't approve itself." |
| `credential_exists` | "That passkey is already added." |
| `already_has_password`, `no_password` | "This account's password state changed. Reopen Settings › Account." |
| The approval codes differ | "The codes don't match. Don't approve on the other device. Cancel and try again." |
| `KeyMismatch`, `UnsafeKdf` | "Something's wrong with the server's copy of your keys. Nothing was changed on this Mac." |
| Transport | "Can't reach the server. Your notes are safe on this Mac." |
| `ServerOutdated`, or a URL that doesn't answer `/v2/health` | "That server doesn't run this version of Kanso sync." |

Sync problems use the words in the problems table.

## Admin script

`npm run -s admin -- [--local] <command>` (`cloud/scripts/admin.ts`, planning in
`src/accounts.ts`). It runs through `wrangler d1 execute` (plus `wrangler d1 export`, `wrangler r2
object get`, `wrangler secret delete` and `wrangler d1 time-travel restore` where stated), with
every value validated and quoted:

```bash
CLOUDFLARE_API_TOKEN="$IMNGONII_CLOUDFLARE_API_TOKEN" npm run -s admin -- accounts
```

| Command | Effect |
|---|---|
| `accounts` | Each account's username, ID, created date, whether it has a password, how many providers and passkeys it has, usage against quotas, writes today against its budget, bytes uploaded today, active devices, last seen and status. Also the settings, global sums and pending purges. |
| `disable <username>` / `enable <username>` | Set or clear `disabled_at`. |
| `delete <username> --confirm <username>` | The deletion batch, plus the purge row. |
| `signups open` / `signups close` / `signups only <username>` | `settings.signup`. `only` then checks whether that name already has an account, and says so: the reservation is spent once it does. Its help and answers say to run it immediately before that name signs up, and, if someone took the name first, to `delete <username> --confirm <username>` and run `only` again (security review 2, finding 6). |
| `max-accounts <n>` | Subject to the overcommit check unless `--force` |
| `quota <username> [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--writes-per-day N]`, `quota <username> --default` | Overrides, in MiB; overcommit check. `--writes-per-day` sets `quota_writes_per_day`, refused over `global_writes_per_day` unless `--force` |
| `default-quota [--docs N] [--doc-mib N] [--files N] [--file-mib N]` | Defaults; overcommit check |
| `writes-per-day <n>` / `upload-bytes-per-day <n>` | `settings.writes_per_day` / `settings.upload_bytes_per_day`, the per-account daily budgets |
| `global-writes-per-day <n>` / `global-upload-bytes-per-day <n>` | The server-wide daily caps |
| `recount [<username>]` | Recompute the counters from the tables, setting `recounted_at` |
| `restore <time \| bookmark> [--confirm <time \| bookmark>]` | The only way to restore D1 (security review 2, finding 10). The point is a Unix time in seconds or an RFC 3339 time with a zone, within the last 30 days, or a bookmark as `wrangler d1 time-travel info` prints it (8-8-8-32 lowercase hex). Refused with `--local`. Without `--confirm` repeating the point, it only prints what it would run. With it, it runs `wrangler d1 time-travel restore kanso-sync --timestamp=… \| --bookmark=… --json`, then at once `restored --all`, and prints the bookmark to undo with. If the new epochs fail, it says to run `restored --all` before any Mac syncs. It also says what a restore undoes: accounts and devices added since are gone, and every sign-out, deletion and password or recovery key change since is undone. |
| `restored [<username> \| --all]` | A new epoch (`lower(hex(randomblob(8)))`), so Macs re-merge (M1). `restore` runs it for every account; alone, it is for a restore that went wrong halfway. |
| `legacy-status` | Whether `docs`/`counter` exist, their live document and file counts and size, and the `files/` purge state |
| `legacy-export --dir DIR` | `manifest.json` (`{"exported_at", "rows": [id, rev, deleted, md5digest]}`), `legacy.sql` (`wrangler d1 export --table docs --table counter`), and every live legacy file from R2 into `DIR/files/` |
| `purge-legacy --account <username> --verified R.json` | Refuses unless `R.json` is `ok`, names that account, and its `manifest_sha256` equals the manifest hash (under "Command line") of a fresh export of the legacy rows (nothing changed since). Then drops `docs` and `counter`, queues the `files/` purge, and runs `wrangler secret delete SYNC_TOKEN` with stdin attached. |

The four daily commands refuse, unless given `--force`, a value after which one account's budget
(the default, any override, or the upload budget) is over its server-wide cap. They and
`max-accounts` print each server-wide cap's fair share (the cap ÷ `max_accounts`), how many
accounts at their whole budget reach the cap, and the day's bound of about twice the cap.

## Tests

The conventions hold: proptest with `PROPTEST_RNG_SEED=20261017`, fast-check with the pinned
`FAST_CHECK_SEED`, regression files kept, and example tests only as a supplement. The engine and
property tests that used to read server state directly (`remote.file(id).md5_hex` at
`tests/sync.rs:218` and `properties.rs:489`, `server_docs`, the plaintext `remote.push` at
`sync.rs:357` and `properties.rs:614`) use `MemoryServer::plain_view(&session) -> PlainView`. It decrypts the server's
documents and files under the test session and can push plaintext as another client.
`MemoryServer::drop_next_responses(route, n)` replaces `drop_next_push_responses` for any route.

| Area | Tests | Where |
|---|---|---|
| Crypto | Round trips for documents, files, wraps and device names. Every bit flip, wrong key, AAD change (account, wire ID, wire rev, deleted, kind, format byte) and shape mismatch is rejected. A format byte above 1 is "too new", not undecryptable. Rewrap keeps the data key; recovery rotates the recovery key; old KEKs can't open new wraps. NFC and NFD passwords derive the same key. Changing the origin or username in the salt input changes `auth_key`. | `kanso-sync/tests/crypto.rs` |
| Recovery key | Round trip; every single substitution and adjacent data transposition detected; lenient parsing | same |
| Golden vectors | `crypto-v1.json` (owner: Rust builder, step 2a): Argon2 at test parameters with a fixed origin and username, every HKDF info, both wraps, sealed document, file and name with fixed nonces, wire ID and rev, key ID, recovery-key text. One `#[ignore]` vector at production parameters. | `kanso-sync/tests/fixtures/` |
| KDF | `prop_prelogin_outside_floor_or_ceiling_is_refused` (no auth key is sent) | `tests/accounts.rs` |
| Usernames | Shared fixture; idempotent; case-insensitive; accepted names match the pattern; non-ASCII refused | Rust and `cloud/test/usernames.test.ts` |
| Generations (M2) | `prop_accepted_generations_only_grow` over random multi-Mac histories; a local win's new revision beats the remote one; replayed older revisions and tombstones come back `Stale`, except a live version a Mac never held over its deletion, which wins (review-verify 2); `Lost` and the re-merge's missing pass keep the floor | `kanso-store` tests, `tests/properties.rs` |
| Integrity of `synced_rev` | Add never pushes a tombstone over the account's live copy (`prop_add_never_pushes_a_tombstone_over_a_live_copy`, and the example for 0–3 edits on either side, notes and files); a restore never silently drops an edit (`prop_no_live_edit_is_lost_silently` with restores, and the two-Mac example); our own echo after a lost answer never reverts or flags (`prop_our_own_echo_never_reverts_an_edit`); missing bytes are fetched again within the bound (`prop_missing_bytes_come_back`); over quota, a refused batch isn't sent again until usage drops (`prop_over_quota_writes_only_grow_by_what_is_sent`) | `tests/integrity.rs` |
| Sync over encryption | The existing convergence and idempotence properties and `sync.rs` cases, run over `Sealed<MemoryServer>` with `PlainView` | `tests/properties.rs`, `tests/sync.rs` |
| Hostile server | `prop_a_hostile_server_never_changes_local_data_wrongly`, with a sequence of attacks per sync. A wrapper flips bits; swaps sealed values between documents and accounts; forges tombstones; relabels revisions; replays old versions; rewinds the account (lower `head_seq`); changes the epoch; leaves documents out of a re-merge; returns `current: null`; answers a push with an old version as the conflict's `current`; and sends unknown format bytes. Invariants: local content is always some client's write for that same ID; the floor never drops; a document's content only changes to a version above the floor it had before the sync; nothing pending is lost; replays are `Stale`; rewinds re-merge; a wrong key ID writes nothing; too-new never overwrites. | `tests/properties.rs` |
| Repair (M3) | A broken server copy (format 1) is replaced from the local copy; one bad document never stops sync | `tests/sync.rs` |
| Rebinding (fix 2) | `prop_rebinding_during_a_sync_never_mixes_accounts`: a remote that rebinds the library mid-sync (between any two calls); no write of A's lands after B's binding; B's cursor is never A's | `tests/binding.rs` |
| Binding | `prop_binding_plan` (model-based): notes reach another account only through an explicit add; Replace always leaves a backup with every document; sign-out never changes the library; sign-up into an empty account pushes no tombstones; a crash between any two steps of Replace leaves either the old library or an empty one bound fresh | `tests/binding.rs` |
| Rollback (fix 1 / H5) | `v2_then_unbind_then_v1_loses_no_edit`: a library synced under v1 rules, then v2 with edits, creations and deletions, then `unbind`, then v1 rules again. Every v2-window edit and creation reaches the v1 server; v2-window deletions come back (asserted, as the rollback steps say). v1 rules come from a plaintext legacy server in `tests/common/legacy.rs` (today's `MemoryRemote`, conformance-tested against the v1 Worker) and the cursor key `sync.last_seq`, so no live old Worker is needed. | `tests/rollback.rs` |
| Client retries | A lost response for sign-up, password change, recovery finish, new recovery key, push and file upload is recovered by the documented retry; a kept key stays kept when a later retry never leaves or is refused (review-verify 1); `recovery_key_held` says whether the server holds a key (review-verify 6) | `tests/accounts.rs` |
| Unconfirmed keys in the app | An unconfirmed key is asked about once: one the server holds is confirmed and says so; one it doesn't hold goes, with a notification; a failed ask keeps the note and asks again only at the next launch (review-verify 6) | `kanso-app` `account_flow_tests.rs` |
| Client quota and pause | Deletions still go through after a 413; a refused batch isn't sent again until usage drops, and then only what fits (`over_quota_syncs_spend_writes_only_on_what_the_server_stores`, and `prop_over_quota_writes_only_grow_by_what_is_sent`); a file-upload quota error holds new uploads; a 429 pauses and keeps its `limit`; `writes-global` is `ServerBusy`; an `uploads-global` 429 holds uploads without blocking pulls; a `session-account` 429 on the feed, a push, an upload or a download backs off briefly with the cursor kept and nothing marked done | `tests/sync.rs`, `tests/integrity.rs` |
| Keychain | An item with a different origin or account ID is ignored; sign-out deletes an unconfirmed recovery key; pending logouts are retried until 204 or 401 | `tests/accounts.rs` (with `MemorySecrets`) |
| Device identity | Same account → same ID; different accounts → unlinkable IDs; a hardware hash mismatch regenerates the identity and leaves the binding | `tests/binding.rs` |
| Plaintext canary | `prop_nothing_plain_leaves_the_mac`: every byte the client sends (push, files, auth, device name) is captured. No marker string, local ID, local rev, file MD5, password, recovery key, data key or device name appears in it, raw, hex or base64 (all alignments, standard and URL). | `tests/canary.rs` |
| Conformance | `conformance.json` (v2 sync cases, quotas and the global caps included) and `auth.json` (owner: Worker builder): same-device replacement, 11th-device eviction, password change signing others out with its reason, recovery rotating the key, logout then 401 `signed_out`, disabled after credentials, sign-up closed / full / busy, expiry, `account_id_taken`, the known-device proof, the idempotent sign-up retry (and the right auth key in a different request not being one), the per-account daily budgets and their overrides, and every rate-limit rule. Both run by the Worker and by `MemoryServer`; `tests/fixtures.rs` keeps their shapes equal to `api.rs` from step 1. The v1 cases in `conformance-v1.json` now only check the rollback test's stand-in for the old Worker. | `cloud/test`, `tests/conformance.rs` |
| Isolation (Worker) | b4e2733's two properties with three signed-up accounts. R2 keys stay under each account's own prefix. A device revocation can't reach another account's session, even one with the same device ID. | `cloud/test/isolation.test.ts` |
| Races (H1) | Through `handle(request, env, now)` with `Promise.all`: N parallel sign-ups at `max_accounts − 1` → exactly one succeeds; N parallel wrong logins → at most the rule's maximum are checked; parallel pushes and file uploads near every quota → no counter ever exceeds its quota; parallel uploads of one ID → `files` and `file_bytes` stay ≥ 0 and `recount` restores them | `cloud/test/races.test.ts` |
| Auth (Worker) | Every route and status; `prop_only_the_registered_auth_key_signs_in`; `prop_sessions_follow_the_model`; idempotent retries; nothing in D1 equals a submitted proof or token; prelogin is deterministic and shaped like a real answer; `Origin` → 403; wrong content type → 415; non-canonical base64url → 400 | `cloud/test/auth.test.ts` |
| Provider tokens | For Apple, Google and GitHub (mocked `fetch` and a local JWKS): a bad signature, an unknown `kid`, `alg: none` and HS256 are refused; wrong `aud`, wrong `iss`, an expired token, a token issued in the future and a replayed `nonce` are refused; the JWKS is cached and refetched once on an unknown `kid`; no token, code, email, name or subject reaches a log or D1; GitHub's numeric `id` is the only field read | `cloud/test/sign_in.test.ts` |
| Provider flows | `state` mismatch, an expired code, a cancelled consent, a loopback `redirect_uri` that isn't `127.0.0.1`, a PKCE verifier that doesn't match, a device-flow `slow_down`, `expired_token` and `access_denied`; a new identity returns a ticket, an existing one returns a session, a link to an account that already owns it is 409 `identity_taken`, and every finish/poll is idempotent | `cloud/test/sign_in.test.ts` |
| Passkey crypto | `PRF_INPUT` is the pinned constant; `passkey_kek` round-trips; another credential ID, account, origin or purpose fails; a wrap moved between credentials fails; a credential without PRF has no wrap; the CTAP salt equals `SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)` and the authenticator output is used without another hash; the golden vector pins the derivation | `kanso-sync/tests/crypto.rs` |
| Passkey assertions | A valid assertion signs in; a bad signature, wrong challenge, wrong origin, wrong RP ID hash, missing UV, unknown credential and a non-growing sign count are refused; a stored count of 0 is exempt; a disabled account is 403 after the signature; the session and the wraps come back | `cloud/test/sign_in.test.ts` |
| Approval protocol | `prop_no_transcript_without_the_keys_yields_the_data_key`: for random keys and transcripts, an attacker who sees only the Worker's rows (both public keys, the sealed name, the wrapped key) never obtains the data key or the code; `prop_a_substituted_key_changes_the_code`: substituting either ephemeral key on either side changes the six-digit code; replay, expiry, single use, self-approval, cancellation, a cross-account request and the zero-device path | `kanso-sync/tests/approvals.rs`, `cloud/test/sign_in.test.ts` |
| Account events | The upgrade refuses a missing/wrong device proof and an `Origin`; one socket per device replaces the old one; hibernation wakes on an event and delivers `approval.created`/`named`/`cancelled`; frames carry no name, code, key or note content; reconnect performs one GET; no 30-second timer exists | `cloud/test/account_events.test.ts`, `kanso-sync/tests/sign_in.rs`, `kanso-app` `account_flow_tests.rs` |
| Linking and method changes | Two accounts linking one identity race to one 409; two sign-ups with one username or account ID race to one 201; unlinking or removing the last method is 409 `last_method` without a recovery proof and 204 with one; a `proof` is single use, expires in 5 minutes and is checked for the session's account; changing a way in with a session alone is 401 `reauth_required` | `cloud/test/sign_in.test.ts`, `kanso-sync/tests/accounts.rs` |
| Sign-in conformance | `cloud/test/fixtures/sign_in.json` (owner: Worker builder): the health advertisement (none; passkeys with and without `security_key`; each provider; events present/absent), passkey start/finish, security-key-shaped messages, sign-up, provider tickets, approvals, method lists, every new error and rate-limit row. Run by the Worker and by `MemoryServer`; `tests/fixtures.rs` checks the shapes against `api.rs` | `cloud/test`, `kanso-sync/tests/conformance.rs` |
| App sign-in | The method list follows health (and hides a method the server doesn't advertise, platform passkeys where there is no native ceremony, and Linux security keys when none is attached); the passkey sheet, Linux security-key form, provider, live approval and waiting states; a locked session can't sync; the code mismatch path; the proof sheet; the pending secret-store item survives a launch; `MemoryServer` + `TestAuthenticator` headless flows for passkey and Linux security-key sign-up/sign-in, provider sign-up/sign-in with approval, linking, unlinking, adding and removing a passkey, setting a password, and recovery removing passkeys | `kanso-app` `account_flow_tests.rs` |
| Rate limits | `prop_limiter_matches_the_model` (fixed windows, counting first); known-device exemption; keys hold no raw IP or username; IPv6 grouping | `cloud/test/limits.test.ts` |
| Quotas | `prop_counters_equal_recomputed_sums`; `prop_growth_past_quota_is_refused_and_shrinking_always_passes`; the reported quota order; global caps; the row-cost measurement; the overcommit refusal | `cloud/test/quota.test.ts` |
| Migration | `prop_0002_leaves_legacy_rows_untouched`; settings seeded closed; the file applies through `applyD1Migrations`; e2e applies it through `wrangler d1 migrations apply --local` | `cloud/test/migration.test.ts`, e2e |
| Admin | Each command's SQL; the overcommit check; the `purge-legacy` manifest-hash guard; b4e2733's injection properties | `cloud/test/admin.test.ts` |
| Purges and cron | Purges finish within the round budget; a row is kept an hour after its prefix empties; limits rows expire; sessions expire at 180 days; revoked rows go after 90 | `cloud/test/purge.test.ts` |
| App | `prop_every_sync_problem_reads_plainly`, `prop_every_error_has_words`, form validation, the config v2 migration, and headless flows against `MemoryServer` + `MemorySecrets` (sign-up through Done, sign-in with each binding answer, session ended with each reason, password change, new recovery key, deletion, sign-out during a sync). Snapshots of every Account page state. | `kanso-app` |
| End to end | Below | `kanso-sync/tests/e2e.rs`, `cloud/scripts/e2e-local.sh` |

Existing tests that must change:

- `services.rs`: `syncing_two_libraries_reports_synced`.
- `account_words.rs`: `server_label` and the form validation tests, which now also cover the method
  buttons and the approval and waiting pages' words.
- `account_page` snapshots: every signed-out and signed-in state gains the method list and the
  sign-in methods section; the signed-in approval sheet and the waiting page are new snapshots.
- `cloud/test/fixtures/conformance.json` and `auth.json`: `health` gains `auth`, and the auth
  fixture gains `reauth_required`, `last_method` and the new limits. The old cases keep their
  meaning.
- `config.rs`: `prop_migration_only_lifts_the_old_default_size`, which becomes a v2 property.
- `sidebar.rs`: the `SYNC_PROBLEMS` properties and `sync_line_examples`.
- `settings.rs`: `sync_status_examples`, `every_sync_problem_has_help` and
  `prop_sync_status_agrees_with_the_sidebar`.
- `palette.rs`: the tests that type "sync" and expect "Sync now" (about lines 2151 and 2478–2503).
  They now run signed in.
- `settings_window_tests.rs`: `sync_now_needs_a_worker` and `a_token_can_always_be_set`, which are
  replaced.

**End to end.** `cloud/scripts/e2e-local.sh`:

1. Make a temporary state directory, and beside it a directory for the canary list.
2. Apply the migrations with `wrangler d1 migrations apply kanso-sync --local --persist-to $dir`.
3. Set `signup` to `open` with `wrangler d1 execute --local`.
4. Start `wrangler dev --local --persist-to $dir --port $KANSO_E2E_PORT --test-scheduled --var
   AUTH_PEPPER:<test>`. Without `KANSO_E2E_PORT` the port is 0: the OS picks a free one and the
   script reads it from wrangler's `Ready on` line, so runs sharing a machine never collide. A
   fixed port something already listens on is refused, so the test never talks to another server.
   This is all Miniflare: no Cloudflare credentials.
5. Run `KANSO_SYNC_E2E_ENDPOINT=http://127.0.0.1:$port KANSO_SYNC_E2E_CANARIES=$list cargo test
   -p kanso-sync --test e2e -- --nocapture`.
6. Stop `wrangler dev`, then run the ignored test `the_stopped_state_holds_no_canary` with
   `KANSO_SYNC_E2E_STATE=$dir`. While the Worker runs, D1's recent writes aren't in its files yet;
   the scan's positive control caught that.

The test runs at production KDF parameters:

1. Two libraries sign up and sign in, then exchange notes, a conflict, a 2 MB image and deletions.
2. A password change gives the other device 401 `password_changed`.
3. Recovery rotates the key: the old key gets 401.
4. Wrong passwords end in 429 with `limit` `login-user` (the binding's 429s are waited out), and the
   known Mac then signs in anyway: its device proof exempts it.
5. A full-size push (4 MiB) and an 8 MiB changes page complete; their wall time is printed as the
   CPU bench.
6. Deleting the account gives 401, and its R2 objects are gone after `/__scheduled`. A second
   account, the witness, stays, with one file.
7. Finally, once the Worker has stopped, every file under the state directory (D1's SQLite and R2's
   blobs) is scanned for the canaries: the marker, the password, the recovery key, the local
   revision hashes and the deleted account's sealed files. As a positive control the scan must find
   the witness's username (stored as it is in D1) and its sealed file (in R2).

The `e2e` CI job runs the script on a port the OS picks: jobs share the self-hosted Mac. After deploy, the Workers dashboard's CPU-time metric for `/v2/push` and
`/v2/changes` during the owner's first upload confirms the bench (expected well under 100 ms,
against a 30 s limit).

## Rollout

### Build order

**Step 1: the contract.** One PR, assembled by the Rust builder, with the Worker builder's files in
it. Merging it freezes the files below. After that, a change needs the review of every builder in
the "Consumers" column.

| Frozen file | Author | Consumers |
|---|---|---|
| `docs/ACCOUNTS.md` | — | all |
| `cloud/migrations/0002_accounts.sql`, applied by `wrangler d1 migrations apply --local` and by a minimal vitest | Worker | Worker, Rust (`MemoryServer` mirrors it) |
| `cloud/test/fixtures/usernames.json` | Worker | Worker, Rust |
| `cloud/test/fixtures/conformance.json` (v2) and `cloud/test/fixtures/auth.json`, checked against `api.rs` by `crates/kanso-sync/tests/fixtures.rs` | Worker | Worker, Rust |
| `crates/kanso-sync`: every public item in "Public API", with bodies `todo!()` except constants and pure validators. Also `wire.rs` types, `SyncReport` (new fields `skipped`, `stale`, `repaired`, `too_new`, `remerged`, `quota`, `paused_until`; `rejected` removed), `SyncProblem`, `problem`, `report_problem`, the binding types (`LocalLibrary`, `Flow`, `Plan`, `Choice`, `local_library`, `plan`, `bind`, `unbind`, `DeviceIdentity`, `device_identity`), the keychain types (`SecretStore`, `Keychain`, `MemorySecrets`, save/load/delete), and `MemoryServer` with its controls (below). `Send + Sync` on `Backend` and `Session`. | Rust | App |
| `crates/kanso-store`: `SyncGuard`, `ApplyOutcome`, `StoreError::GuardFailed`, `reset_sync`, `replace_library`, `mark_unsynced`, `mark_missing_unsynced`, `set_meta_if`, `pending_count`, and the guard parameter on `apply_remote`/`mark_synced`, with call sites updated. Bodies may be `todo!()` only where no current test calls them. | Rust | App, Rust |

`MemoryServer` controls: `new()`; `set_now(ms)`; `set_setting(name, value)`;
`set_signup(Signup::{Open, Closed, Only(name)})`; `set_max_accounts`; `set_default_quota`;
`set_quota(username, …)`; `set_writes_per_day`; `disable`/`enable(username)`; `devices(username)`;
`revoke_all(username, reason)`; `expire_sessions(unseen_for)`; `set_epoch(username)` and
`rewind(username, to_seq)` (restores); `drop_next_responses(route, n)`; `stats()`;
`plain_view(&Session)`.

**Contract notes.** Decisions made while freezing step 1, binding on every builder. The sections
above already say each of them; this list is the index.

*Security re-check, applied in step 1:*

1. **Settings fail closed.** Every trigger reads its setting with `coalesce(…, 0)` (`'closed'` for
   `signup`); `settings_keep` and `daily_keep` refuse deleting a row, and renaming one is refused
   too. Proven in a scratch database with the guards dropped: each missing setting refuses.
2. **Server-wide daily caps.** Settings `global_writes_per_day` (300,000) and
   `global_upload_bytes_per_day` (5,000,000,000; uploads weighted by size, replacements included).
   The `daily` table holds the UTC day's sign-ups, document writes and upload bytes; only triggers
   write it, so deleting accounts can't lower a count. `signups_per_day` is per UTC day. Past a cap:
   429 `rate_limited` with `limit` `writes-global` or `uploads-global` and `retry_after` to the next
   UTC midnight.
3. **The same account is the same `(origin, account_id)`.** Meta key `sync.v2.origin`;
   `SyncGuard { expect }` over both (`binding::guard_pairs`); `local_library(store, origin,
   account_id)`; `keychain::load_session` and `load_recovery_key` take both, and the recovery item
   holds `origin`; `AccountRef` gains `origin`. The wrap AAD gains `lp(account_id)`.
4. **Re-merge keeps `synced_rev`** and pulls from 0. `Stale { new_rev }` rebases the local version
   and pushes it like a local win; `mark_missing_unsynced` re-pushes synced documents (tombstones
   included) the full feed lacks, with `base_rev = null`.
5. **Device proof.** `HKDF(device_secret, "kanso/v1/device-proof" || lp(origin) ||
   lp(account_id))`, sent as `device.proof` at sign-up, sign-in and recovery; `sessions.proof_hash`
   holds its SHA-256. Only a matching proof exempts a sign-in from `login-user`. Types:
   `crypto::DeviceProof`, `DeviceIdentity::device_proof`, `api::NewDevice { id, sealed_name, proof }`,
   `api::LoginDevice { id, proof }`.
6. **Legacy manifest hash**: rows only, sorted by `id`, the byte format and test vector under
   "Command line"; `verify::manifest_sha256` implements it.
7. **Sign-up retry is idempotent on the server**: the same request again (username, account ID,
   KDF parameters, salt, auth key, wrapped keys, recovery auth key and key ID) → 201 with the
   existing account, whatever the settings. The right auth key with anything else different is not
   a retry. `auth.json` has both cases.
8. **R2 and D1 drift**: GET needs the `account_files` row; the cron reconciles orphans and sizes.
   The only schema addition is `account_files.uploaded_at`, which the upload cap needs anyway.
9. `login-user` is keyed by account ID, with one shared row for unknown names; the binding groups
   IPv6 by /48; `limits` rows go when their window ends.
10. Automatic repair only when the broken wire revision is `wire_rev(id, synced_rev)`, pushed one
    generation higher; anything else is skipped and left to `verify --repair`. The store method for
    that bump is the Rust builder's to add (additive, not frozen).
11. Credential updates (recovery finish, password change, new recovery key) are compare-and-swap.
12. Deletion: the client counts only 401 `unknown` as success. The login batch keeps at most 90
    revoked rows per account.
13. `signups only <name>` checks for an existing account afterwards; `check_endpoint` refuses a
    host ending in a dot.
14. `DEFAULT_SERVER` is `https://sync.kansoapp.app`, a later name of the owner's server, which
    derives as its first, `https://kanso.antonlabs.cc` (see Origins).

*Other step-1 decisions:*

- **Fixtures.** The v1 file is renamed `conformance-v1.json` and still runs against the v1 Worker
  and `MemoryRemote` until v1 code goes. `conformance.json` is v2. The step language is in each
  file's `about`. Every case starts with sign-ups open and the clock at a UTC midnight. Raw steps
  are Worker-only; `MemoryServer` runs the rest through `Api`. The expected values were checked
  against an independent model before freezing.
- **What the fixtures pin.** Health says `closed` for `closed` and `only:`, `full` when open at
  `max_accounts`, else `open`. A missing or malformed bearer is 401 `unknown`. Revoking the current
  device is a logout (`signed_out`). Sign-up triggers answer before `UNIQUE`, and
  `account_id_taken` before `username_taken`. Deletion checks the key before the username. Device
  proofs must be canonical like auth keys. `login-ip` counts before `login-user`, and the first
  over its maximum is named. `retry_after` is the seconds to the window's end, rounded up.
  `quota_exceeded` with `server` reports the server-wide sum and cap. Push validates all but the
  sealed alphabet before counting `writes-account`; a refused body counts 1, and an empty push
  counts nothing (security review 2); file uploads check
  path and headers before counting. `limit` outside 1–500, or a negative `since`, is 400.
- **`MemoryServer`.** `new()` starts with sign-ups open and the migration's other settings
  (`DEFAULT_SETTINGS`, checked against the migration by a test). `set_setting` panics where D1's
  `CHECK` would refuse (`is_valid_setting`). `set_now` fixes its clock.
- **API surface.** `Api` has every v2 route with explicit tokens, so the conformance runner can
  replay any token; `WireRemote` is the session-bound sync subset, and `Backend::remote` defaults
  to an adapter over `Api`. `Route` names are the fixtures' route names.
- **Account.** `Step` gains `StoppingSync`. `AccountError` gains `Protocol`, `Endpoint` and
  `Keychain`. `check_new_username` and `UsernameError::Reserved` refuse reserved names on the
  client. `kanso_sync::origin(server)` computes the origin.
- **Binding.** `Machine { hardware_uuid, computer_name }` feeds `device_identity`, so tests can fake
  the Mac. `device_name` trims again after cutting, so it is idempotent. `bind(store, session,
  choice, backups_dir, now_ms)` returns the backup's path after a Replace; `Cancel` binds nothing.
  `local_library` checks Same, Empty, Other, Unbound, in that order. The meta keys live in
  `kanso_sync::meta_keys`.
- **Keychain.** `set_pending_logouts` replaces the list; `PendingRecoveryKey` and `PendingLogout`
  are the item types.
- **Engine.** `SyncReport.skipped` holds the wire IDs of undecryptable documents not repaired;
  `stale` and `repaired` aren't problems. `report_problem` precedence: `UpdateKanso`, `Paused`,
  `ServerFull`, `QuotaExceeded`, `Skipped`. `problem` maps a 429 to `Other`; the engine itself turns
  a push or upload 429 into `paused_until`. The v1 engine (`sync_once`, `sync_with`, `MemoryRemote`,
  `HttpRemote`, `credentials`, `SyncConfig`) keeps working until 2b removes it.
- **Left to the Rust builder:** `verify` gains `repair` and `VerifyReport.repaired`; `PlainView` has
  inherent methods. `Sealed<W>`, the plaintext types in `remote.rs` and crypto internals beyond
  `Password`, `RecoveryKey`, `KdfParams` and the key types are internal and may change.
- **Implemented in step 1** (pure or trivial): username, password and recovery-key validation; wire
  patterns and `ApiError` parsing; `problem`, `report_problem` and `schedule`; `plan`;
  `device_name`; `manifest_sha256`; `origin`; the guard helpers, `mark_unsynced`,
  `mark_missing_unsynced`, `set_meta_if` and `pending_count`; `MemorySecrets`. `LocalWon.new_rev`
  is the unchanged local revision until 2a adds the generation bump.

*Step 3 (integration) decisions:*

- **v1 is gone.** The plaintext engine, `HttpRemote`, `MemoryRemote`, the shared-token Keychain
  code, `SyncConfig`, `CURSOR_KEY` and the legacy tests are deleted; the Worker serves no `/v1`.
  `kanso-sync unbind` (the rollback path) stays. The rollback test's stand-ins for the old app and
  Worker live in `tests/common/legacy.rs`, checked against `conformance-v1.json`; the old app's
  store rules are behind `kanso-store`'s `legacy-v1-rules` feature, which only `kanso-sync`'s
  dev-dependencies enable.
- **Daily budgets.** `writes_per_day` is 20,000 (a fifteenth of `global_writes_per_day`, 300,000);
  `accounts.quota_writes_per_day` (NULL: the setting) overrides it per account, set with `quota
  <user> --writes-per-day N`, and `/v2/account` reports the effective value. The per-account
  upload cap is the setting `upload_bytes_per_day` (1 GiB), rule `uploads-account`, counted from
  `Content-Length` after `writes-account` passes; its 429 pauses sync like any push or upload 429.
  Both maxima are read from D1 in the counting batch, and a missing value refuses. The admin
  script has `upload-bytes-per-day`, and its four daily commands refuse, unless `--force`, a value
  that lets one account's budget exceed the server's cap. Amended in `0002_accounts.sql` (not yet
  applied anywhere), the fixtures, `MemoryServer` and the Worker together.
- **Sign-up retry** (note 7) is the same request again, not just the same auth key; `MemoryServer`
  now checks every field as the Worker does.
- **Cancelled binding questions** log the new session out with `kanso_sync::discard_session`,
  which leaves `sync-account` and `sync-recovery-key` alone (`sign_out` deleted them: the signed-in
  account's session, or a cancelled recovery's new key). A failed sign-up still deletes the
  recovery key it saved, which nobody saw. An account task that stopped the sync and leaves the
  same session signed in starts it again.
- **The editor and sync revisions.** A local win, a stale rebase or a repair gives the open note a
  new `_rev` with the same content (`Change` with `Origin::Sync`). `editor::same_content`
  compares versions apart from `_rev` and the derived task counts; the editor adopts such a
  revision without asking, unsaved typing and all, and a save that meets one first retries once
  on top of it.
- **Tests.** The account flow tests run the real sync (`Services::no_sync` is gone);
  `services.rs` has its two-library test back over `MemoryServer`. The e2e canary scan runs after
  the Worker stops, with a positive control, and the wrong-password step asserts `login-user`.

**Step 2: in parallel.**

- **W, the Worker builder.** Everything in `cloud/`, against the frozen migration and fixtures.
  Done when `npm test` and `npm run typecheck` are green.
- **R, the Rust client builder.**
  - *2a*: `crypto.rs`, `username.rs`, `account.rs`, `binding.rs`, `keychain.rs`, `MemoryServer`,
    the store APIs, and `crypto-v1.json`. Its tests are green and `tests/conformance.rs` passes
    `auth.json`.
  - *2b*: `sealed.rs` and the engine changes, `http.rs`, `verify.rs`, the CLI, and the canary,
    rebinding, rollback and hostile-server tests. `cargo test -p kanso-sync -p kanso-store` is green.
- **A, the app builder.**
  - *A1*, from step 1: config v2, services, `account_page.rs`, dialogs, sidebar, welcome, palette,
    menu, the clipboard helper, and snapshots, using the stubs and stub states.
  - *A2*, after Rust 2a merges: the headless flow tests against `MemoryServer`, and the existing
    tests listed above.

**Step 3: integration.** `cloud/scripts/e2e-local.sh` is green (needs W and 2b). Then the app
against `wrangler dev` by hand, and CI green: fmt, clippy `-D warnings`, workspace tests, worker,
e2e.

**Step 4: deploy and migrate**, below.

### Deploy and the owner's migration

0. **While v1 still works.**
   1. Tag the current head `sync-v1`.
   2. In a worktree of `sync-v1`, on every Mac, quit Kanso and run `cargo run --release -p
      kanso-sync -- --data-dir DIR --endpoint https://kanso-sync.imngonii.workers.dev --full`.
      Repeat until the report shows `pending`, `files_missing` and `invalid` at zero.
   3. Reopen Kanso and don't edit until step 7 (H4, fix 10).
1. **Backups.**
   1. Settings › General › Back up now.
   2. Copy `/Applications/Kanso.app` to `Kanso-old.app`.
   3. Record the D1 Time Travel bookmark (`npx wrangler d1 time-travel info kanso-sync`).
   4. Confirm `wrangler d1 migrations list kanso-sync --remote` shows only `0001_init`.
   5. Run `npm run -s admin -- legacy-export --dir ~/KansoLegacy-<date>`. Keep that folder with
      your backups; it holds the old plaintext.
2. **Pepper.** Set `AUTH_PEPPER` as above.
3. **Migrate.** `npm run migrate:remote` applies `0002_accounts` (additive; sign-ups closed). The old
   Worker keeps working.
4. **Deploy the Worker.** `npm run deploy`, with the cron, the rate limiter and invocation logs off.
   From here `/v1` answers 404.
5. **Install the app.** Quit Kanso, then `scripts/bundle-macos.sh` and copy it to `/Applications`.
   It opens signed out with every note in place. Run `kanso-sync estimate` and compare it with the
   quotas.
6. **Create the owner's account.**
   1. Open Settings › Account › Create account and fill it in, but don't submit yet.
   2. Immediately before submitting: `npm run -s admin -- signups only <you>`. Until the account
      exists, anyone who guesses the name can take it, so leave no gap.
   3. Submit. Save the recovery key, then Done.
   4. If the name was taken (the app says so, or `signups only` said the reservation is spent
      before you signed up), someone else got it first: `npm run -s admin -- delete <you> --confirm
      <you>`, then repeat from 2.
   5. If the estimate exceeded a default, run `quota <you> …` and Sync now.
7. **Verify.** Wait for "Synced". Quit Kanso, then run `cargo run --release -p kanso-sync -- verify
   --data-dir ~/Library/Application\ Support/Kanso --legacy ~/KansoLegacy-<date>/manifest.json
   --out ~/KansoLegacy-<date>/verified.json`. It must exit 0.
8. **Other Macs, one at a time.** Install the new build, Sign in, then choose [Add them to *you*].
   They were synced in step 0, so this only adds their local-only changes. Then run `kanso-sync
   verify` on each.
9. **Purge the legacy data.** `npm run -s admin -- purge-legacy --account <you> --verified
   ~/KansoLegacy-<date>/verified.json`. Within 15 minutes, `legacy-status` shows the `files/` purge
   done. Time Travel still holds the purged plaintext for 30 days, and `~/KansoLegacy-<date>` holds
   it for good.
10. **Open sign-ups:** `npm run -s admin -- signups open`.
11. **Clean up.**
    1. `security delete-generic-password -s Kanso -a sync-token`.
    2. Update `cloud/DEPLOYED.md`, `cloud/README.md` and `README.md`.
    3. Delete `Kanso-old.app`.

**Other people's Macs.** The builds are ad-hoc signed and not notarized, so:

- **Gatekeeper.** On first launch, Control-click the app and choose Open, or use System Settings ›
  Privacy & Security › Open Anyway.
- **Keychain.** Each new build has a different signature, so macOS may ask once per build to let
  Kanso use its Keychain items. Choose Always Allow.

`README.md` says this.

### Sign-in methods: build order

This is a second, additive build after v2 is deployed. Days are builder-days for one person; the
Worker and the Rust/app sides can overlap where noted.

| Step | Work | Days | Done when |
|---|---|---|---|
| 1. Schema and providers | `0006_sign_in.sql`; `account_identities`, `account_passkeys`, `auth_flows`, `device_approvals`; the health advertisement; Apple/Google/GitHub token verification with a cached JWKS and mocked `fetch`; `cloud/test/sign_in.test.ts` and `fixtures/sign_in.json` | 4 | The migration applies through `wrangler d1 migrations apply --local`; health advertises nothing, then passkeys, then each provider as its secrets appear; every token case (bad signature, wrong `aud`/`iss`, expired, replayed nonce, `alg: none`) is refused; no token, email or name reaches D1 or a log. Tests: the provider and provider-flow rows above. |
| 2. Passkeys and the PRF wrap | Registration and assertion on the Worker; `crypto.rs`'s `PRF_INPUT`, `passkey_kek`, `WrapPurpose::Passkey`; the native macOS and Windows ceremonies (no web page); the Linux libfido2 CTAP2 client, its CTAP salt mapping and the same WebAuthn shapes | 8 | A platform passkey creates an account, signs in, and unlocks a new device with PRF on macOS 15; on macOS 13/14 it signs in and falls back to approval; a Linux USB HID security key registers, signs in, and unlocks with `hmac-secret`; a key without it signs in and falls back to approval; a cloned counter is refused; the wrap round-trips and a moved wrap fails. No Worker page is loaded. Tests: passkey crypto, CTAP salt vectors, libfido2 fakes and assertions, plus the golden vector. |
| 3. Device approval and events | The X25519 request/challenge/name/approve protocol; the Worker routes and limits; `ACCOUNT_ROOM` with hibernating WebSockets; `events.rs`; the approver sheet, live refresh and the waiting page | 6 | Two devices complete the flow; both show the same code; a substituted key shows different codes; a revealed key that doesn't match the commitment is refused by the Worker and by the approver; the request expires, is single use, can be cancelled and refuses self-approval; zero devices offers recovery. An unlocked approver receives `approval.created` and `approval.named` over the socket; the socket survives idle hibernation, replaces a duplicate device socket, rejects a missing or wrong device proof, and recovers through one GET after reconnect. No frame contains note content; no 30-second timer remains. Tests: approval property tests, account-event auth/hibernation, reconnect and app flow. |
| 4. Provider flows and linking | Apple's web flow (Services ID, client secret, `form_post`, callback exception); Google's loopback + PKCE; GitHub's device flow; new-account tickets; link/unlink; `provider.rs`; the app's browser sessions | 6 | Each provider creates an account and signs in; linking and unlinking work; a second account linking the same identity gets 409; no provider secret is in the app; the callback works with an `Origin` header, the other routes still refuse one. Tests: provider flows, linking races, app flows. |
| 5. App UI and Keychain | The signed-out method list, the create-account variants, Settings › Sign-in methods, the pending item, the locked state, the proof sheet, snapshots | 4 | Every state has a snapshot; a locked session can't sync; the pending item survives a relaunch; the method list follows health; the code and the name are accessible. Tests: the app sign-in row and the snapshots. |
| 6. Sensitive changes and rotations | `Proof`; set password; adding/removing passkeys; unlinking the last method; recovery removing passkeys; recovery-key and delete with a proof | 3 | A session alone can't change a way in; a proof is single use and expires; the last method needs the recovery key; recovery leaves the account with a password and no passkeys; a passwordless account can still delete. Tests: linking and method changes, rotations. |
| 7. End to end, docs and deploy | `e2e-local.sh` gains a two-device approval and a dropped-socket reconnect; passkey ceremonies are checked by hand on macOS, Windows and Linux; `cloud/README.md`, `cloud/DEPLOYED.md`; owner setup; security review fixes | 3 | The e2e passes locally; the canary scan still finds no plaintext; the two docs name every secret and route; a real Apple/Google/GitHub account completes each flow on macOS; the Windows platform passkey flow works; Linux offers only a security-key ceremony and completes it with a real USB key; the live approval sheet needs no 30-second wait. |

**Total: about 34 builder-days.** With a Worker builder and a Rust/app builder in parallel, about
four to five calendar weeks after the accounts v2 rollout. Linux security keys add about two days
and the live approval channel adds about two; Apple's developer resources are already configured.
Leaving Windows without platform passkeys saves about two days.

### Deploying sign-in methods

The order matters: migration, secrets, deploy, then the app. Nothing is destructive, so a Worker
rollback is `wrangler rollback` to the previous version; the new tables stay and are ignored.

1. **Back up.** Record the D1 Time Travel bookmark (`npx wrangler d1 time-travel info kanso-sync`)
   before the migration. Old app builds keep signing in to password accounts; they can't sign in
   to an account created without a password, so update the app before making one.
2. **Migration.** `npm run migrate:remote` applies `0006_sign_in` through `0010_end_flows_with_sessions`
   (all additive).
3. **Secrets and vars.** `APPLE_PRIVATE_KEY` is already uploaded to the `kanso-sync` Worker. In
   `cloud/`, set the four Apple vars from the owner table in `wrangler.jsonc`; add
   `GOOGLE_CLIENT_SECRET` with `wrangler secret put`, and the passkey vars and `APP_ASSOC_APPS`. A
   provider appears in health only when its values are present, so they can be added one at a time.
4. **Deploy.** `npm run deploy`.
5. **Check the AASA file.** `curl -sI
   https://sync.kansoapp.app/.well-known/apple-app-site-association` must be 200, with no redirect
   and `application/json`. Apple's CDN may take hours to pick up a change. Sign in with Apple needs
   no domain-association file.
6. **Test with real accounts.** One Apple, one Google and one GitHub account, each on a new
   account and on a linked existing one; a platform passkey on macOS 15; a Linux USB security key;
   a second device approved by the first over the live event socket, then again after dropping and
   reconnecting it; a recovery that removes a passkey. Nothing is written to a shared document; the
   test accounts are the owner's.
7. **Docs.** `cloud/README.md` gains:
   - a row for every new route in its route table, and the Apple callback's `Origin` rule;
   - the secrets and vars table (what each is, and that the secrets go through
     `wrangler secret put`, never a file);
   - the `auth` and `events` shapes of `GET /v2/health` and what a deployment without them means;
   - the AASA path and why Apple's CDN may lag;
   - migration `0006_sign_in`, and the sentence that no email, name, profile or provider token is
     stored, that the GitHub device code is sealed under `AUTH_PEPPER`, and that the PRF output
     never leaves the app.

   `cloud/DEPLOYED.md` gains: the date and Worker version, the migration and the D1 bookmark from
   just before it, which providers are on, the RP ID, the AASA URL, the Services ID and the public
   client IDs. Never a secret.
8. **Rollback.** `wrangler rollback` to the previous Worker. Health drops `auth`, the app hides the
   methods, and the tables stay. An account created without a password can't sign in with a
   password on the old Worker; it can still use its provider or passkey if the old Worker is only a
   rollback of the sign-in code (it isn't: the old Worker has no routes for either). Treat a
   rollback as temporary and keep the new build available; the accounts are not lost, and the new
   Worker restores their access.

#### What the owner must provide

The owner does all of this in their own accounts and pastes secrets only into `wrangler secret put`
or `.dev.vars` (never into a file that is committed, a chat, or a document). Secret values are never
repeated in this document.

| # | Thing | Where it goes |
|---|---|---|
| 1 | Apple Developer Program membership (paid), team `4973Y692JU`. | — |
| 2 | The release App ID `app.kanso.Kanso` with **Associated Domains** enabled, and a **Developer ID provisioning profile** that authorizes it. Embed it at `Kanso.app/Contents/embedded.provisionprofile` and sign with it. This is what makes native macOS passkeys work. Do **not** add Sign in with Apple to this App ID: Apple's table doesn't offer it to Developer ID. | The app build, not the Worker. |
| 3 | Sign in with Apple: primary App ID `app.kanso.signin` (`4973Y692JU.app.kanso.signin`) with the capability; Services ID `app.kanso.Kanso.web` grouped under it, with domain `sync.kansoapp.app` and return URL `https://sync.kansoapp.app/v2/auth/apple/callback`; key "Kanso Sign in with Apple", Key ID `MS6Y3S5ZL7`. | The `.p8` is already uploaded as `APPLE_PRIVATE_KEY` on the `kanso-sync` Worker; `APPLE_CLIENT_ID`, `APPLE_TEAM_ID`, `APPLE_KEY_ID` and `APPLE_REDIRECT_URI` are not set. |
| 4 | The Associated Domains entitlement `webcredentials:sync.kansoapp.app` and the AASA app list `["<TEAMID>.app.kanso.Kanso"]`. | The app's entitlements and `APP_ASSOC_APPS` (var); the Worker serves the AASA. |
| 5 | A Google Cloud project with an OAuth consent screen (External; only `openid`, no email or profile scopes) and an OAuth client of type **Desktop app**. | `GOOGLE_CLIENT_ID` (var), `GOOGLE_CLIENT_SECRET` (`wrangler secret put`). |
| 6 | A GitHub **OAuth App** (not a GitHub App) with Device flow enabled; homepage and callback `https://sync.kansoapp.app`. | `GITHUB_CLIENT_ID` (var). GitHub uses the device flow only; there is no secret. |
| 7 | The passkey host. `sync.kansoapp.app` already resolves to the Worker; no new DNS is needed. `PASSKEY_RP_ID=sync.kansoapp.app`, `PASSKEY_ORIGINS=["https://sync.kansoapp.app"]`. Linux security keys use the same RP ID. | `wrangler.jsonc` vars. |
| 8 | Apple's CDN fetch and a test device. Nothing to configure on Cloudflare beyond not blocking `/.well-known/*`; the Worker's `assets` config must let those paths reach the Worker (they are routes, not files). | — |
| 9 | Test accounts: one Apple ID, one Google account, one GitHub account, two devices for approval, and one Linux-compatible USB or NFC security key. | — |

**Apple resources: confirmed in the live developer account on 2026-10-23.** Re-opening the
Services ID showed the domain and return URL still present. The `.p8` was downloaded once, uploaded
to `APPLE_PRIVATE_KEY` on the `kanso-sync` Worker, and the downloaded file was deleted; no key
material is in the repository, this document or a chat.
The Worker vars are still to be set and no sign-in has been implemented or deployed, so a real Apple
login remains unverified.

#### Open questions

Recommendations are on merit; the effort is the added builder time if the other option is chosen.

1. **Which host is the RP ID?** *Recommendation:* `sync.kansoapp.app`, the public default. It keeps
   `share.kansoapp.app` and every other subdomain out of the passkey scope, and native macOS uses
   the same host even when the account's canonical origin is `kanso.antonlabs.cc`. Using
   `kansoapp.app` would let any subdomain ask for the credential; it saves nothing.
2. **Do social accounts get a username?** *Recommendation:* yes, suggested and editable at sign-up.
   A nullable username would touch `accounts`, invites, `kanso-identity-bind`, recovery and
   `AccountRef`, and a table rebuild: ~3 days, and a worse product (no invites, no
   `GET /v2/users/<username>`). The suggested form adds no days.
3. **Does recovery remove passkeys and security keys?** *Recommendation:* yes. A credential added
   while someone knew the password must not survive the break-glass path; re-adding one is one
   ceremony from the recovered device. Keeping them would save no time and leave a takeover path.
   The screen says how many go.
4. **Does every change need a fresh proof?** *Recommendation:* yes. A stolen session could otherwise
   add an attacker's provider, passkey or security key and persist after a password change. The
   proof is the passkey sheet, the password, or a provider flow; it adds about 3 days (step 6).
   Session-only changes would save that and leave a real escalation.
5. **GitHub device flow or web + PKCE?** *Recommendation:* device flow. It needs no loopback
   listener, no client secret and no redirect; the user reads a code. GitHub supports PKCE now, so
   the web fallback remains (~1 day) for a deployment that can't enable Device flow. *Decided:*
   device flow only, with no fallback.
6. **Native platform passkeys on Windows?** *Recommendation:* the Win32 `webauthn.h` call, for
   platform passkeys on macOS and Windows. It is the only way to reach Windows Hello directly, and
   a page served by the Worker is rejected on E2E grounds (see "Where the ceremony runs"). Doing
   nothing on Windows would save about 2 days and leave the platform with the most desktop users
   without platform passkeys. Linux does not depend on this: its external security-key path is
   decided above.
7. **Six digits or eight?** *Recommendation:* six, shown as two groups of three. With the
   commitment, a key-substituting server wins with probability 10⁻⁶ per request, and each attempt
   is a visible request the user can cancel; six is easier to read aloud and compare. Eight would
   take that to 10⁻⁸ at no extra build cost, for a harder comparison.
8. **Should a passwordless account be deletable?** *Recommendation:* yes, with a `proof` and the
   typed username (a provider proof also needs the recovery key). Requiring a password would
   strand the accounts this revision creates; the proof is the same bar as the password.

### Rollback

- **Before step 9.** The legacy tables, `SYNC_TOKEN` and the Keychain `sync-token` item are intact.
  1. On every Mac, quit Kanso and run `cargo run --release -p kanso-sync -- unbind --data-dir DIR`
     (from the new commit). This removes the `sync.v2.*` account keys and the Keychain item, sets
     live `synced_rev` to NULL, and sets the legacy `sync.last_seq` to 0.
  2. `npx wrangler rollback` to the previous Worker version.
  3. Reinstall `Kanso-old.app`. Its Worker URL is still in `config.json` (`endpoint` was preserved);
     re-enter it if missing.

  The old app pulls everything and pushes every change made during the v2 window. Newer edits win,
  and both versions are kept. Notes deleted during the v2 window come back; delete them again. The
  v2 tables stay, unused.
- **After step 9.** There is no return to plaintext sync; fix forward. `npm run -s admin --
  restore <step 1 bookmark>` (within 30 days) restores the database and then makes every Mac
  re-merge. The files are in `~/KansoLegacy-<date>`.
- **Restoring D1, at any time.** Only through `admin restore <time | bookmark>`, never with
  `wrangler d1 time-travel restore` or the dashboard. A restore alone rewinds every account's feed
  while each Mac still trusts its cursor and epoch, so pushes and pulls would go wrong silently;
  `restore` gives every account a new epoch at once, which makes each Mac re-merge (M1). A
  restore also brings back the sessions, passwords and recovery keys of that time: a device token
  revoked since works again until it is revoked again, and anyone who changed a password or
  recovery key since must change it again. Run it first without `--confirm` to see what it would
  do.
- **Client-only problems.** Sign out (notes stay), install a fixed build, and sign in. "Same"
  resumes.

## Collaboration

Kanso users can share a note with each other and edit it together, live or each on their own
time, with comments and @mentions. The server stores and relays what they write and can't read
any of it. Private notes are untouched: they sync as above, and only a shared note changes how it
travels.

### What it is

- **Sharing.** The note's ⋯ menu, Note › Share…, or the palette › Share… opens the Share sheet. The
  owner invites another Kanso user by username, or makes an invite link, and gives each person a
  role:

  | Role | Read | Comment | Edit | Manage people |
  |---|---|---|---|---|
  | View | ✓ | | | |
  | Comment | ✓ | ✓ | | |
  | Edit | ✓ | ✓ | ✓ | |
  | Owner | ✓ | ✓ | ✓ | ✓ |

  A shared note has one owner, who shared it. The owner changes roles, removes people, cancels
  links, approves link requests and stops sharing. Anyone else can leave.
- **Shared with me.** Notes others shared with you sit under "Shared with me" in the sidebar,
  beside the invitations waiting for you, on every device signed in to your account. Accepting an
  invitation brings the note in. A note you were removed from, or whose owner stopped sharing,
  stays as a private copy of its last text, marked as no longer shared.
- **Merging.** Two people editing the same note at the same time, or one of them offline, both
  keep their edits: shared notes are merged by a CRDT (Yrs, the Rust port of Yjs, wrapped in
  `crates/kanso-collab`), never settled as "theirs vs mine".
- **Live.** While a shared note is open, edits arrive within about a second, and everyone in it
  sees where the others' cursors and selections are, each in their own colour, with their name.
  Remote edits never move your cursor or selection, in Vim mode too.
- **Comments.** Select text and choose Comment (or the Comments tab in the inspector). Comments sit
  outside the note's text, anchored to the words they are about; a comment whose words were deleted
  shows as orphaned, at the nearest place that is left. Comments can be edited and deleted by their
  author and resolved or reopened by anyone who may comment.
- **@mentions.** `@username` in a comment, for a member of the note, sends them a notification;
  they see it in "Shared with me".
- **History.** The note's history says who made each version.
- **Locked notes are never shared.** A locked note, or a note in a locked notebook, can't be
  shared; a shared note can't be locked, or moved into a locked notebook, until sharing stops.

### Identity keys

Sharing needs a public key per account, which v1 accounts didn't have. The first time a signed-in
device needs one (opening the Share sheet, Shared with me, or Settings › Account), it makes them,
on the device:

| Key | Algorithm | Use |
|---|---|---|
| `sign` | Ed25519 (`ed25519-dalek`) | Signs every update, comment, presence message and member list the account writes |
| `box` | X25519 (`curve25519-dalek`'s Montgomery ladder, clamped) | Receives note keys and notifications |

- **Sealed for the account's devices.** `identity_key = HKDF(data_key, info "kanso/v1/identity")`.
  `sealed_identity = base64url(0x01 || nonce(24) || XChaCha20-Poly1305(identity_key, nonce, aad,
  sign_seed || box_secret))`, 105 bytes or 140 characters, with `aad = "kanso-identity" || 0x01 ||
  lp(account_id) || sign_pub || box_pub`. The server keeps it, so every device of the account opens
  the same keys; the server can't.
- **Binding.** `binding = Ed25519(sign, "kanso-identity-bind" || 0x01 || lp(account_id) ||
  lp(username) || box_pub)`, 64 bytes. It ties the box key to the sign key, the account and the
  username; a client checks it on every public key it is given.
- **Fingerprint.** `SHA-256("kanso-fingerprint" || 0x01 || sign_pub || box_pub)`, its first 30
  bytes read as six 5-byte big-endian numbers, each mod 100,000 and written as five digits:
  `48213 00917 66305 12874 90321 55102`. Settings › Account shows yours.
- **Verified on first share.** The server hands out public keys, so a hostile server could hand out
  its own. The first time you share with someone (or accept from them), the sheet shows their
  fingerprint beside yours and asks you to compare them some other way (in person, a call, a
  message); "Mark as verified" pins it. Kanso pins every key it has seen to its account ID
  (`collab_contacts`): a key that changes for a pinned account is refused with "<name>'s key
  changed", and nothing is wrapped to it until you look at the new fingerprint and accept it.
- **Accepting a changed key** pins exactly the keys whose fingerprint you compared: if the server
  offers others by then, nothing is pinned.
- **Never replaced silently.** The server refuses a different identity once one is set (409
  `identity_exists`). There is no way to change it in this version; it lasts as long as the
  account.

Routes (session, JSON):

- `PUT /v2/account/identity` `{sign_pub, box_pub, binding, sealed}` (43, 43, 86 and 140
  characters) → 201 `{ok}`; 200 when the same identity is already set; 409 `identity_exists` for a
  different one.
- `GET /v2/account/identity` → `{sign_pub, box_pub, binding, sealed}`, or 404.
- `GET /v2/users/<username>` → `{account_id, username, sign_pub, box_pub, binding}`; 404 for an
  unknown or disabled account, or one without an identity. Counted by the `lookup-account` rule
  (120 an hour per account).

### Keys for shared notes

- **`note_key`.** 32 random bytes per shared note per **epoch**. Epochs count from 1. Every change
  to who is in the note, or to anyone's role, starts a new epoch with a new key.
- **Wrapped to each member.** For a member with box key `B`: a fresh ephemeral X25519 pair
  `(e, E)`; `dh = X25519(e, B)`, refused when all zero; `k = HKDF(ikm dh, salt E || B, info
  "kanso/v1/note-key")`; `wrapped = 0x01 || E || nonce(24) || XChaCha20-Poly1305(k, nonce, aad,
  note_key)`, 105 bytes, 140 characters, with `aad = "kanso-note-key" || 0x01 || lp(share_id) ||
  u32(epoch) || lp(member_account) || role`. `role` is one byte: `o`, `e`, `c` or `v`. A wrap
  opens only for its member, note, epoch and role.
- **Key commitment.** `key_commit = hex(SHA-256("kanso-note-key-commit" || 0x01 || lp(share_id) ||
  u32(epoch) || note_key))` is in the signed member list. The wrap itself doesn't say who made it,
  so a member also checks that the key it opened matches the owner's commitment: a server can't
  hand a member a key of its own choosing.
- **At rest on a device.** Opened note keys are kept in the library (`collab_keys`), sealed under
  `vault_key = HKDF(data_key, info "kanso/v1/collab-vault")` with `aad = "kanso-vault" || 0x01 ||
  lp(share_id) || u32(epoch)`.

`u32(n)` and `u64(n)` are big-endian.

### The member list

For each epoch the owner signs the list of who is in the note:

    {"v":1,"share":"<share id>","epoch":<n>,"owner":"<account id>","key_commit":"<64 hex>","at":<ms>,
     "from_seq":<n>,"members":[{"account":"a_…","username":"…","role":"owner|edit|comment|view",
                 "sign_pub":"<43>","box_pub":"<43>"}, …]}

Compact JSON, keys in this order, members sorted by account ID, exactly one `owner` (the owner
themself), at most 20 members. `from_seq` (from epoch 2 on; never in epoch 1's list) is the
log's head when the epoch began: the owner's device signs the head it read, and the server takes
the epoch only while that is still the head (409 `head_moved` `{head}` otherwise, and the device
signs again with that head). `list_sig = Ed25519(owner sign, "kanso-members" || 0x01 ||
list_bytes)`. On the wire both travel as base64url (`list`, `list_sig`), so the bytes signed are
the bytes received.

- **Clients check every list:** the signature under the owner's pinned key (looked up, its binding
  checked, and pinned the first time), its share ID, the owner's own entry, and each member's keys
  against the pinned ones. A member seen for the first time is looked up by username: the keys
  their account published (binding checked) must be the list's, or the list is refused. A list
  never pins anyone else's keys.
- **No going back.** The current list must be no older than the newest epoch the account has
  seen: on the device, or on another of its devices, through the epoch in the note's `kansoShare`
  (which private sync carries end-to-end encrypted). A server replaying an older list, with
  someone removed since still in it, is refused.
- **A list per epoch.** The server keeps every epoch's list (`collab_lists`) and gives each member
  the lists of the epochs it holds keys for. A key is kept only when its epoch's list is the
  owner's and the key matches that list's `key_commit`; a wrap opens with the member's role in
  that list (or, for an epoch from before they came, the role they have now).
- **Clients enforce roles too.** Every envelope's author must be in the owner-signed list of the
  envelope's own epoch, with that list's keys; nobody else's envelope is opened. An update whose
  author isn't `edit` or `owner` there is dropped, and so is a comment from a `view` member, even
  though such a member holds the key: the server refuses them (below), and a client would refuse
  them from a server that didn't.
- **An epoch ends where the next begins.** An envelope of epoch `e` at a sequence number past the
  `from_seq` of a later epoch's list is dropped: a server can't relay a removed or demoted editor's
  late writes into the new epoch. (The server's check of `from_seq` against the head isn't atomic
  with storing the epoch, so clients drop these whatever the server took.)
- **Leaving and declining.** The member's device tells the server; the owner's device, on its next
  refresh, sees them gone from the server's members and starts an epoch without them. Until then
  the Share sheet shows them as left, and no key is wrapped to them again: someone the server no
  longer has as a member is never put back in a list.
- **Members whose keys can't be checked.** A listed member first seen when their account is gone
  (deleted or disabled, so the lookup finds nobody) or the lookup fails is shown as unverified, and
  the owner's next list leaves them out. What they wrote under a list that names them still merges,
  checked against that list's keys: the owner checked those keys when adding them, and dropping
  their entries would make members' copies differ. One share's error (a list that doesn't check,
  an owner whose account is gone) never stops the others' refresh or sync, or hides other
  invitations; only being offline or signed out does.
- **Pending invitations pin only the owner.** The other members are pinned once you accept. The
  invitation shows the owner's fingerprint, marked verified when it matches one you compared.
- **The server reads it** (it is not secret: the server knows who is in the note anyway) to know
  each member's role.

### Updates, snapshots, comments and presence

Everything written to a shared note travels in one envelope:

    header   = 0x01 || kind || u32(epoch) || lp(author_account) || lp(author_device) || u64(counter)
    sealed   = XChaCha20-Poly1305(note_key[epoch], nonce(24), aad, plaintext)
    aad      = "kanso-collab" || 0x01 || lp(share_id) || header            (|| u64(base_seq) for a snapshot)
    signature= Ed25519(author sign, "kanso-collab-sig" || 0x01 || lp(share_id) || header || nonce || sealed)
    envelope = header || nonce || sealed || signature

| Kind | Byte | Plaintext | Stored |
|---|---|---|---|
| Update | `u` | A Yrs v1 update | In the note's log until a snapshot covers it |
| Snapshot | `s` | The whole Yrs state, as a v1 update from the empty state | Replaces every update up to `base_seq` |
| Comment | `c` | A comment event (JSON, below) | In the log, never compacted |
| Presence | `p` | `{"cursor","head","name"}` (sticky positions as base64url) | Never: relayed live only |

- `counter` grows by one for each envelope a device writes to a note; with the author and device it
  names an envelope.
- The signature makes every change attributable: nobody, the server included, can write as
  someone else, and history says who wrote what from signatures, not from the server.
- **Long edits go in parts.** Parts are cut by their encoded size: at most 64 KiB of inserted text
  and operations, counting 48 bytes for each operation, so a rewrite with many small changes
  splits too. Each part is well under the 256 KiB cap. If the server still refuses one as too
  large, the device keeps the edit as a private note ("<title> (not shared)") beside the shared
  one, starts the note again from the log, and tells you.
- **A note's whole state fits one snapshot.** An edit that would grow the state past 4 MiB less
  4 KiB is kept as a private note instead, and you are told. The Share sheet warns when a note is
  near that.
- **The library's limits.** A merged title or body longer than a note can be here (128 and
  1,048,576 characters) is cut, on whole characters, for the library only; the CRDT keeps all of
  it, and the cut is never sent as a deletion. A write the library refuses (a limit, a lock) never
  stops a merge: you are told why the note didn't change.
- **Locked notes.** A shared note can't be locked or moved into a locked notebook. If one is anyway
  (another device locked it before it learnt the note is shared), the library refuses to write the
  shared text into it, and you are told.
- **Coming back.** A note shared with you is `note:s<share id>` in your library. If that ID is taken
  by another note (the private copy left when you were removed), it is `note:s<share id>.<epoch>`,
  the epoch you joined at, so every device of the account picks the same note.
- **What can't be merged is skipped.** An update that doesn't decode (or that Yrs refuses) is
  skipped like a forged one, so one bad entry never stops a note's log. A device whose updates the
  server refuses for good (its role was lowered) drops them and starts the note again from the
  log; the library keeps its text until the log's has merged.
- **Keystrokes are batched.** The editor turns its changes into Yrs operations as you type, and
  sends what it has every 250 ms while live, or once on the next sync otherwise.
- **The body stays plain text locally.** A shared note's `body` (and `title`) in the library are
  the CRDT's text, written back after every merge. Search, links, the graph, export, the local
  REST API and MCP read it as they read any note. Every local write (the editor, the API, MCP)
  becomes Yrs operations by diffing the text the CRDT last had against the new text
  (`similar`, by characters); the CRDT is never written directly.
- **Comment events.** `{"id": <22>, "op": "add|edit|resolve|reopen|delete", "anchor": {"start",
  "end", "quote"}, "body", "mentions": [<account>], "at"}`. `anchor` and `mentions` only on `add`;
  `body` on `add` and `edit`. A comment is folded from its events in log order: `edit` and
  `delete` count only from the comment's author; `resolve` and `reopen` from any member who may
  comment; events from a `view` member, or for a comment never added, are dropped. The anchor's
  `start` and `end` are Yrs sticky positions; when both resolve to the same place and the quoted
  text is gone, the comment is orphaned.
- **Snapshots (compaction).** After 500 updates or 1 MiB of them since the last snapshot, an
  editor's device writes a snapshot of the whole state at the log's head. The server takes it only
  if `base_seq` is still the head (compare-and-swap, 409 `head_moved` otherwise, and the device
  tries again after reading what's new). A snapshot takes no sequence number of its own: its `seq`
  is its `base_seq`. A new epoch gets a snapshot under its key, written by the owner's device right
  after the epoch changes (or on its next sync).
- **Newcomers read what came before.** A member added in a later epoch is also given the keys of
  the earlier epochs (wraps that name their `epoch`), so they can read the comments written before
  they came, which are never compacted, and any updates the newest snapshot doesn't cover yet.
- **Order.** A device merges the log strictly in sequence order, never skipping a gap: entries the
  room sends ahead of a gap wait until the log has filled it. Presence is sealed under counter 0
  and never stored.

### The server: routes

All are session routes under `/v2/collab/`, JSON unless they say bytes. `<id>` is a share ID: 16
random bytes as 22 base64url characters, chosen by the owner's device.

| Method | Path | Who | Does |
|---|---|---|---|
| POST | `/v2/collab/shares` | anyone with an identity | Share a note: `{share_id, list, list_sig, wraps: [{account, wrapped}], snapshot}` (epoch 1, the snapshot's `base_seq` 0). Other members start as `invited`. 201 `{share_id, epoch, head}` |
| GET | `/v2/collab/shares` | | Every share you are in or invited to: `{shares: [{share_id, owner, owner_username, role, status, epoch, list, list_sig, lists: [{epoch, list, list_sig}], wraps: [{epoch, wrapped}], head}]}` (`lists`: every epoch you hold a wrap for, and the current one, oldest first); for your own shares also `members: [{account, status}]`, `links: [{link_id, role, created_at, expires_at}]` and `requests: [{account, username, link_id, role, requested_at, sign_pub, box_pub, binding}]` |
| POST | `/v2/collab/shares/<id>/accept` | invited | Accept: 204 |
| DELETE | `/v2/collab/shares/<id>/membership` | member or invited, not the owner | Leave or decline: 204 |
| POST | `/v2/collab/shares/<id>/epoch` | owner | New epoch: `{epoch, list, list_sig, wraps: [{account, wrapped, epoch?}]}`, `epoch` the current one + 1, the list's `from_seq` the room's head (409 `head_moved` `{head}` otherwise), exactly one wrap of the new epoch for every member, and wraps naming an earlier `epoch` for listed accounts only (each account and epoch once, at most min(20 × epoch, 400) in all). Accounts no longer listed lose their rows and every wrap; new ones are `invited`, or `member` when they asked through a link. 200 `{epoch}`; the same request again 200 (a retry); 409 `epoch_changed` `{epoch}` |
| DELETE | `/v2/collab/shares/<id>` | owner | Stop sharing: every row, wrap, link and request goes, and the log is purged. 204 |
| GET | `/v2/collab/shares/<id>/log?since=&limit=` | any member | `{head, snapshot: {seq, base_seq, envelope} \| null, entries: [{seq, envelope}], more}`. The snapshot comes when `since` is 0 or below its `base_seq`; entries are every update and comment after `since` (updates the snapshot covers are gone; comments stay), at most `limit` (1–1000, default 500) and about 8 MiB |
| POST | `/v2/collab/shares/<id>/log` | bytes | Append one envelope: an update (edit, owner) or a comment (comment, edit, owner). 200 `{seq}` |
| POST | `/v2/collab/shares/<id>/snapshot` | bytes, `x-kanso-base-seq` | Edit or owner. 200 `{seq}` (its `base_seq`; the head doesn't move); 409 `head_moved` `{head}` |
| GET | `/v2/collab/shares/<id>/live` | WebSocket | Live editing (below); 426 `upgrade_required` without `Upgrade: websocket` |
| POST | `/v2/collab/shares/<id>/links` | owner | `{link_id, proof_hash, role, expires_at}` (role edit, comment or view; at most 30 days; at most 10 live links). 201 |
| DELETE | `/v2/collab/shares/<id>/links/<link_id>` | owner | 204 |
| POST | `/v2/collab/join/<link_id>` | anyone with an identity | `{proof}`: asks to join. 202 `{share_id, owner_username, role}`; 404 for an unknown, expired or cancelled link or a wrong proof alike. Counted by `join-account` (20 an hour) |
| DELETE | `/v2/collab/shares/<id>/requests/<account>` | owner | Decline a request: 204 |
| POST | `/v2/collab/notifications` | member who may comment | `{share_id, recipient, sealed}` for a member or invitee of that note. 201. Counted by `notify-account` (500 a day) |
| GET | `/v2/collab/notifications` | | `{notifications: [{id, share_id, from, sealed, at}]}`, oldest first, at most 500 kept |
| DELETE | `/v2/collab/notifications/<id>` | recipient | 204 |
| GET | `/j/<link_id>` | public | A static page: "Open Kanso, choose Join a shared note…, and paste this link." It never reads D1 |

**Who sees what.** Someone not in a share gets 404 on all of its routes; an invited account that
hasn't accepted gets 404 on the log, snapshots, `/live` and notifications too. Owner-only routes
answer a member 403 `forbidden_role`. Errors are `{"error": <code>, "message", …}`; besides those
above: 403 `identity_required` (sharing or joining without an identity), 409 `unknown_member`
`{account}` (a listed account has no identity), 409 `member_keys_mismatch` `{account}` (a listed
member's keys or username aren't their account's identity), 409 `share_id_taken` and `share_retired`, 409
`link_id_taken`, 413 `quota_exceeded` `{quota, used, limit}` with `quota` one of `members`,
`shares`, `links`, `requests`, `doc_bytes` or `comment_bytes`, and 429 `rate_limited` `{limit,
retry_after}` with `Retry-After`. Creating a share, an epoch or a link again with the
same request answers success again, so a device can retry after a dropped answer.

**What the server checks on every envelope** (header only; it never opens one), in order: the
route's body cap (413 `too_large`); a well-formed header (the format byte, a known kind, an epoch
of 1 or more, the author an account ID `a_<32 hex>` and the device a device ID `d_<32 hex>`, so a
header is always 86 bytes, then room for a nonce, a tag and a signature) and the kind for the
route (else 400 `bad_request`); `epoch` equal to the share's current epoch (else 409
`epoch_changed` `{epoch}`); `author_account` and `author_device` equal to the session's (else 403
`forbidden_author`); the caller's role for the kind (else 403 `forbidden_role`); the envelope's
size for its kind (413 `too_large`). `cloud/test/fixtures/collab.json` pins the header cases; the
Worker and `kanso-collab` both run every one. The server checks the member list's shape (share,
epoch, owner, key commitment, sorted unique members, one owner) but not its signature: members do.

**Links.** The device that makes a link remembers its role, and approving gives that role,
whatever the server says the request asked for. An invite link is `<server>/j/<link_id>#<secret>`: a 16-byte ID and a 32-byte secret,
both base64url, the secret only ever in the fragment. `proof = HKDF(secret, info
"kanso/v1/join-proof")`, and the server keeps `proof_hash = hex(SHA-256(proof))`. Someone with
the link asks to join with the proof; the owner sees their username and fingerprint in the Share
sheet and approves (a new epoch with them in it) or declines. A link alone grants nothing: a leaked
link lets a stranger ask, not read.

**Notifications.** `sealed` is the note-key wrap construction for the recipient's box key, with
`info "kanso/v1/notice"` and `aad = "kanso-notice" || 0x01 || lp(recipient) || lp(share_id) ||
lp(sender)`, over `{"comment": <id>, "title", "excerpt", "at"}` (at most 4 KiB sealed). It only
points at the comment: the seal doesn't say who made it, so the recipient keeps a notification
only once the note's log has that comment, signed by its sender, adding it and mentioning them,
and shows the note's title and the comment's words from its own library, never the
notification's.

### The server: storage and the live room

- **D1** (migration `0005_collab.sql`, which only adds tables): `identities`, `collab_shares`
  (owner, epoch, the current list and signature), `collab_members` (role and status), `collab_wraps`
  (per member per epoch), `collab_links`, `collab_requests`, `collab_notifications`,
  `collab_retired` (a share ID that stopped is never used again) and `collab_purges`.
- **One Durable Object per shared note** (`NoteRoom`, binding `NOTE_ROOM`, keyed by share ID) holds
  the note's log in its own SQLite storage (`log(seq, kind, epoch, author, envelope, at)`, the
  snapshot, the head, the counters), so a keystroke never writes to D1. The Worker authenticates
  the session and reads the caller's role and the epoch from D1, then forwards to the room with
  them; the room is reachable only through the Worker. On a new epoch, a removal or stop, the
  Worker tells the room, which updates or closes the sockets at once.
- **WebSockets with hibernation.** `GET …/live` upgrades to a WebSocket accepted by
  `ctx.acceptWebSocket` with the caller's account, device and role in its attachment
  (`serializeAttachment`), so the room can sleep between messages without being billed. Frames:
  - client → room, binary: one envelope (`u`, `c` or `p`).
  - room → clients, binary: `u64(seq) || envelope`, to everyone in the room the sender included
    (that is the sender's acknowledgement), `seq` 0 for presence, which goes to the others only and
    is never stored.
  - room → client, text JSON: `{"type":"hello","head","epoch"}` on connect, `{"type":"epoch",
    "epoch"}` when it changes, `{"type":"error","error":<code>,…}` for a refused frame (with its
    `kind` and `counter`, as a decimal string, when the header parsed; the frame is dropped), and
    `{"type":"closed","reason":"removed"|"stopped"}` before the room closes the socket with code
    1000.
  - The upgrade request carries `Authorization: Bearer …` and no `Origin`. Snapshots are never
    sent to sockets.
  The device reads anything it missed (over `…/log`) when it connects, then lives on frames, and
  falls back to HTTP when the socket drops.

### Limits

| Limit | Value |
|---|---|
| Update | 256 KiB |
| Snapshot | 4 MiB |
| Comment | 16 KiB |
| Presence | 2 KiB |
| Log since the last snapshot | 10,000 entries or 16 MiB; past that, 409 `compact_first` until a snapshot |
| Room bytes per UTC day | 64 MiB per note, snapshots included, presence not (429 `rate_limited`, `limit` `room-bytes`) |
| WebSocket messages | 30 a second per socket, in bursts of 60 (an `error` frame `rate_limited`, `limit` `socket`, the frame dropped) |
| Presence | 10 a second per account in a room, across its sockets, in bursts of 20 (an `error` frame `rate_limited`, `limit` `presence`, the frame dropped) |
| Sockets | 8 per account per note (429 `rate_limited`, `limit` `sockets`, `retry_after` 5) |
| Comments | 8 MiB per note (413 `quota_exceeded`, `quota` `comment_bytes`) |
| Members per note | 20 |
| Shares owned per account | 200 |
| Live links per note | 10; pending requests per note 50 |
| Notifications kept per recipient | 500 |

Shared notes count towards their owner's storage. Every envelope a room stores is checked against
the owner's `quota_doc_bytes`: their documents, plus their other shared notes, plus this room's
live size (snapshot and log, comments included). Past it, 413 `quota_exceeded` `{quota:
"doc_bytes", used, limit}` (an `error` frame on a socket). Each room reports its size to D1
(`collab_shares.bytes`) when it has grown 64 KiB since it last did, at every snapshot, and
otherwise 30 seconds after a change, so the owner's other notes can count up to that late. An
owner past the quota can't share another note.

### What the server learns

On top of what it learns from private sync, the server sees: each account's public keys and its
sealed private keys; who shares which (opaque) note with whom, with which roles, invited and
accepted when; who asked through which link; when each member reads and writes the note, the
size and timing of every update (keystroke timing, bucketed by the 250 ms batches), comment and
presence message, and who is in a note together live; who mentions whom, and when. It does not
see the note's title or text, comments, cursor positions, names typed in comments, or the note
keys.

### Threat model

- **A curious or compromised server** can't read shared notes, comments, presence or
  notifications, can't write as a member (every envelope is signed), can't add a member (clients
  only wrap keys to, and accept updates from, the members of a list the owner signed), and can't
  give a member a key of its own (the key commitment).
- **Key substitution.** The server hands out public keys. Until you compare fingerprints, it could
  give you its own key for someone you share with, the first time only (keys are pinned after
  that). Compare fingerprints for anything sensitive.
- **Removing a member** starts a new epoch whose key they never get, so they can't read anything
  written after, and the server stops serving them the log. They keep whatever they already had:
  no system can take that back, and the sheet says so.
- **A viewer holds the key,** so cryptography alone can't stop them writing. The server refuses
  their updates, and every member's client drops updates from anyone the signed list doesn't make
  an editor. A commenter can write comments only.
- **Withholding and reordering.** A hostile server can hold back or drop updates, or show members
  different subsets. It can't forge or alter any; the CRDT merges whatever arrives, in any order,
  to the same text. Updates a removed or demoted editor wrote under an earlier epoch can't be
  slipped in after the next epoch began (`from_seq`).
- **A malicious editor's snapshot** is signed by them and opens with the note key, but nothing
  proves it holds everything the log had: an editor (not the server) can write a snapshot that
  leaves earlier text out, and once the server compacts, devices that start from it never see what
  was left out. Devices that had already merged that text keep it. Give edit rights to people you
  trust with the note's history; each device's version history keeps what it saw.
- **Open sockets.** A socket is checked when it opens. Removal, leaving, stopping and deleting the
  account close it at once; signing out, or an account being disabled, doesn't until it next drops.
- **Metadata** as listed above.

### Rust crates

`crates/kanso-collab` holds the protocol and wraps Yrs (`yrs = "0.28"`, MIT), so a switch to Loro
stays a change inside it: `identity.rs`, `keys.rs`, `members.rs`, `envelope.rs`, `doc.rs` (the
CRDT, diffing and sticky positions), `comments.rs`, `api.rs` (`CollabApi` and the wire shapes),
`http.rs` (`HttpCollab` over `ureq`, and the WebSocket client over `tungstenite` and `rustls`),
`memory.rs` (`MemoryCollab`, an in-memory server with the Worker's rules, for tests) and `engine/`:
`sharing.rs` (sharing, epochs, links, invitations), `log.rs` (pulling, pushing, compaction,
notifications), `live.rs` (one thread per open note's room), `people.rs` (lookups and pinned keys)
and `mod.rs` (the open editor, comments, presence) against a `Store`. A note shared with you gets
the ID `note:s<share id>` in your libraries; the owner's devices keep the note's own ID. `ed25519-dalek` and `curve25519-dalek`
were already in `Cargo.lock`. The library keeps its side in `collab_*` tables (store migration 9):
the CRDT state of each shared note, its note keys (sealed), the outbox, folded comments, pinned
contacts and notifications. Shared notes carry `kansoShare` (`{shareId, role, owner, ownerName, epoch}`) in the note
so the account's other devices see it; private sync sends a shared note without its `body` and
`title` (the CRDT carries those), and keeps the local text when it applies one. Only Kanso writes
`kansoShare`: the local API and MCP keep it as they keep `kansoPublish`, and a `view` or `comment`
member's local writes to the text are refused. While a shared note is open, the editor's text
reaches the library only through the engine (`sync_editor`), which merges it with what arrived and
hands the editor back splices that never move its caret.

## Threat model

Protected against: a curious or compromised server or owner; a leaked D1 database, R2 bucket or
Time Travel copy; network observers; malicious web pages (M7); and a malicious second Kanso server
(M5).

- They get ciphertext and the metadata listed above.
- A D1 leak alone doesn't allow signing in: auth hashes are hashes of 256-bit keys, and session
  tokens are stored hashed.
- Writes are authenticated, so the server can't forge, alter, relabel, cross-link or move documents,
  files, deletions or device names without the Mac noticing.
- Replaying an older version is refused once a Mac has seen a newer one (M2), also after the server
  claims it lost the document or rotates its epoch. The exception is a live version that a Mac
  never held, over its deletion: an edit beats a deletion (see the stale reads below).
- Rewinding a whole account makes the Macs re-merge rather than lose changes (M1).

### Sign-in methods and providers

The normal model still holds: a provider or its token proves identity; it never becomes a key.
What changes is who can authenticate, and what a malicious party can do with the new paths.

A provider sees a sign-in to Kanso: its client ID, the person's account, their IP and the time. It
does not see the library, the account ID, the username or any content. Apple returns no email
because no scope is requested; Google and GitHub data are discarded.

| Threat | What it can and can't do | Mitigation |
|---|---|---|
| A curious or compromised Worker owner | Can deny, delay, roll back, drop a flow, show different metadata to two devices, or insert rows directly. Sees the provider's ID token claims and, for GitHub, a token with no scopes beyond public identity; none of that opens the library. Can't read the data key, the PRF output, the approval code, the sealed name or the wrapped key. Can't forge a passkey signature, an Apple or Google ID token, or a GitHub user. | Every key path is end-to-end; sessions are not enough to change a way in; the approval code comes from a DH secret the server doesn't have; the client is trusted (below). |
| A malicious server substituting ephemeral keys in an approval | Can replace `epk_N` or `epk_A` in its rows. Can't compute the DH secret, and can't choose its stand-in keys so the two codes agree: the requester commits to its key before seeing the approver's, so each substitution matches with probability 10⁻⁶. | Commit-then-reveal; both devices show the code; a mismatch cancels and saves nothing. |
| A leaked D1 database or Time Travel copy | Gets subjects, credential public keys, PRF-less session hashes, flow metadata, sealed names, wrapped keys, and GitHub's short-lived device code (sealed under the pepper). Can't sign in without a passkey, a password, a provider token or a live session; can't read a library. | Auth hashes and session tokens are hashed; device codes are sealed and short-lived; passkeys are public-key checked; the data key is never in D1. |
| A compromised identity provider (Apple, Google or GitHub) | Can authenticate as any of its users, including issuing `sub` or a token for a person it shouldn't. That is account takeover of the *authentication*, not decryption. It can then request approval from a device, which shows the request, the device name and the code; or use the recovery path if it has the key. | The data key stays wrapped; approval needs the user's other device and its confirmation; a provider that is compromised for one account is visible in the sign-in methods list. There is no way to make a compromised IdP harmless; don't use a provider you don't trust. |
| A stolen provider access token | Nothing: the Worker exchanges codes server-side, never stores a provider token, and the app never holds one. | Tokens are discarded immediately; no refresh tokens. |
| A network attacker (on-path or on the local network) | Can see that a TLS connection goes to the provider or the Worker, and can block it. Can't read or alter the OAuth exchange, the passkey ceremony or the approval transcript: TLS, PKCE, `state`, the nonce and WebAuthn's origin binding protect them. Can't replay a flow. | TLS everywhere; exact redirect URIs; single-use flows; the approval code is compared on screen, not sent. |
| A stolen session token | Can sync ciphertext, list devices and request an approval, but not decrypt. Can't open the account event socket without the device proof, and can't link, unlink, add or remove a passkey, set a password or delete without a `proof`. | The data key is in the device secret store, not the token; the socket needs the per-device proof; sensitive routes need a fresh proof; sessions expire and can be revoked. |
| A stolen unlocked device | Can read the library and approve another device (it holds the data key). | Sign out or revoke the device from another one; change the password (signs sessions out); recover (removes every passkey) if the device is lost. A passkey on it is protected by the platform's user verification. |
| A stolen device with a passkey but no unlocked session | The passkey needs the platform's biometric or PIN; the session is gone. | Platform user verification; the session's 180-day life; device revocation. |
| A provider account takeover | Can sign in (a session) and request approval. Can't read a library without the user's approval or recovery key. Can unlink the provider it controls (and nothing else) and try to set a password, replace the recovery key or delete the server copy with a provider proof, though all three also need the recovery key it doesn't have. It can't add a passkey or link another provider without a password or passkey proof. | The approval prompt shows a device and code the person didn't initiate; recovery removes passkeys and replaces the password; the method list shows unexpected links; local copies survive a deletion. |
| An attacker who knows the username | Can enumerate usernames at sign-up, as before; can start a passkey challenge; can't complete one. | Existing rate limits; the passkey start says no more than sign-up does. |
| A malicious app build or a compromised operating system | Can read the screen, the Keychain and the data key. | Out of scope, as for the password path: build from source or trust the builder. |

The following are **not** protected against, and the app or the docs say so where they matter:

- A compromised identity provider, as above. Passkeys are the method that doesn't depend on one.
- A person who approves a request they didn't initiate. The code and the name are the whole defence;
  the app never auto-approves and never hides the code in a notification.
- A platform passkey provider that syncs the credential to an account the user loses (for example a
  compromised iCloud or Google account). The passkey is then usable wherever it syncs, subject to
  that platform's own protections; the recovery key and the method list are the response.
- A lost or stolen external security key. It needs its PIN or on-device UV, and it can sign in even
  when it has no `hmac-secret` wrap. Without the wrap it still needs another unlocked device or the
  recovery key to open a new library. Remove it from Settings and recover if the key was
  compromised; recovery removes every passkey and security key.

Residual risks:

- **Offline password guessing.** Anyone holding an account's D1 row (the owner, a D1 leak, or Time
  Travel within 30 days) can guess passwords against the wrapped data key, at one Argon2id (1 GiB,
  t = 3) per guess. A weak password falls. Mitigations: the 12-character minimum, the passphrase
  hint, and memory-hard parameters.
- **Password changes don't replace the data key (M4).** Anyone who once knew the password and
  fetched the wrapped key, or who read the data key from a signed-in Mac, can decrypt anything they
  later obtain from the server, until a rekey. v1 has no rekey; see "Deferred". A password change
  does replace the recovery key, in the same update (security review 2, finding 1): a recovery key
  that such a person set while they knew the password stops working, so they can't take the
  account over by recovery after the owner changes it. What remains is reading, not control.
- **Stale reads and withholding.** A malicious server can withhold changes, delete data, or replay
  an old version to a Mac that never saw a newer one. It can bring a deleted note back on a Mac
  with a version another Mac wrote that this Mac never held, since an edit always beats a deletion
  (review-verify 2). It can't fabricate content, and every replaced local version is kept in history.
- **A passkey added while someone else had the password survives a password change.** A password
  change revokes sessions but keeps passkeys, because removing a user's own passkeys would be
  surprising and lock them out. The success screen lists every passkey with its dates and offers to
  remove them all; a recovery removes them all. A person who suspects someone else had the password
  should recover, or at least read that list.
- **Metadata.** Sizes, counts per kind, timing and IPs are visible, as listed above. Sign-in adds
  which providers an account uses, each provider's stable subject, when it was linked, how many
  passkeys or security keys exist and when each was added and last used (to the hour), which devices
  are asking to be approved, and which devices hold an account event socket. It does not add an
  email, a name or a profile.
- **A compromised provider.** Apple, Google or GitHub can authenticate as the person, which is
  account takeover at the front door; it still can't decrypt. This is inherent to using a provider
  and is why passkeys and security keys exist; see "Sign-in methods and providers".
- **A person who approves the wrong request.** The device name and the six-digit code are the
  defence, and the app never auto-approves. A user who taps Approve without comparing the code has
  handed the library to the requester.
- **A passkey's syncing provider.** A passkey in iCloud Keychain, Google Password Manager or a
  third-party manager is only as safe as that account and that manager. Losing that account can
  lose the passkey; a compromise there can use it, subject to the platform's own checks. The
  recovery key and the ability to remove the passkey are the response.
- **The client is trusted.** Friends run ad-hoc builds the owner makes. End-to-end encryption
  protects them against the server, but not against whoever builds the app: a modified build could
  send keys anywhere. Anyone who doesn't trust the owner should build Kanso from source, and can
  point it at their own Worker.
- **The recovery key.**
  - It is as powerful as the password. It sits in the Keychain until the user confirms it's saved,
    and wherever the user puts it after that. The clipboard copy stays on this Mac and is cleared
    after a minute, and saving to iCloud warns.
  - A used key stops working, and so does every key from before a password change.
  - A key replaced on another Mac with "New recovery key" can still show on this one as unsaved:
    that change signs no session out, and the Worker gives no key ID to compare. Saving it saves a
    key that no longer works; the one the other Mac showed does. A password change or a recovery
    elsewhere ends this Mac's session, and the unsaved key goes with it (sec-client 3).
- **A compromised Mac.** The local library is plaintext, and the Keychain holds the data key and the session.
- **Lost password and recovery key.** The server copy is unrecoverable, by design. Local copies
  remain. A lost passkey (and no other device) needs the recovery key; an account with only one
  provider and no password or passkey is recovered the same way.
- **Lockout.** Up to 10 sign-in attempts per account per 15 minutes from devices that aren't
  signed in can be used up by anyone. Signed-in Macs, which prove themselves with their device
  proof, are unaffected (M8, security re-check 5). A Mac that was signed out (by itself, another
  Mac, a password change, recovery, eviction or expiry) counts like a stranger until it signs in
  again, so an old copy of its library can't be used to guess passwords past the limit.
- **Floods and cost.** Without a custom domain and a WAF rule, a large flood can drive Workers
  request costs. Per-account write budgets and `SESSION_LIMITER` cap what one account costs, the
  global caps bound storage, and the server-wide daily caps bound a day's writes and uploads at
  about twice each cap. A few accounts at their own budgets can no longer pause everyone else's
  sync: an account within its fair share is never refused by a server-wide cap.
- **Unaudited crates.** The crypto crates are recent RustCrypto releases (argon2 0.6,
  chacha20poly1305 0.11), pinned in `Cargo.lock`.

## Deferred (v1 behaviour stated)

- **Rekey (M4).** v1 has no way to replace the data key. Later: `POST /v2/account/rekey`, taking
  `current_auth_key` plus new `kdf`, `salt`, `auth_key`, `wrapped_key`, `recovery_auth_key`,
  `recovery_wrapped_key` and `key_id`. In one batch it would:
  1. Delete the account's documents and files (and queue the R2 purge).
  2. Replace both wraps, both hashes and the key ID, with a new epoch.
  3. Revoke every other session (`rekeyed`).

  The calling Mac then unbinds, rebinds fresh and re-uploads. The schema, the `rekeyed` reason and
  the epoch logic already support it; it needs no format change.
- **Platform passkeys outside macOS and Windows.** Linux uses external security keys (above), but
  Linux's platform passkey UI and WebView/browser ceremonies remain deferred. A page served by the
  Worker is still rejected: it is server code, and it could take the PRF output. Until a platform
  API exists, Linux does not offer platform passkeys; it does offer external USB/NFC security keys,
  username and password, providers, approval and the recovery key. A self-hosted server's own
  domain needs its own app build with that domain in the associated-domains entitlement for platform
  passkeys; external security keys do not.
- **Moving a server's URL.** v1 binds accounts to their server's origin (M5) and has no flow for a
  changed URL; owners must keep the URL. Later, a move would work like a password change:
  connected through the new URL, derive the current key with the old origin from the Keychain item,
  and the new one with the new origin. No format change.
- **Email, 2FA, a web client, suggested edits (track changes) and editing shared notes on the web:** out
  of scope. Sharing is in "Collaboration".

## Critical Files for Implementation

**Step 1, contract**

- Rust builder:
  - `crates/kanso-sync/src/{lib.rs, wire.rs, api.rs, account.rs, crypto.rs, username.rs, binding.rs, keychain.rs, memory.rs, remote.rs, engine.rs, sealed.rs, verify.rs, http.rs}` (stubs and types)
  - `crates/kanso-sync/tests/fixtures.rs` (fixture shapes)
  - `crates/kanso-store/src/{sync.rs, lib.rs}` (store API signatures)
- Worker builder:
  - `cloud/migrations/0002_accounts.sql`
  - `cloud/test/fixtures/{usernames.json, conformance.json, auth.json}`; `conformance-v1.json` (renamed)
  - `cloud/test/migration.test.ts` (minimal), with `MIGRATION_DB` in `cloud/vitest.config.ts` and `cloud/test/env.d.ts`

**Worker builder**

- Create:
  - `cloud/src/auth.ts` (prelogin, sign-up, login, recovery, logout, password, recovery key, delete, devices, session lookup)
  - `cloud/src/usernames.ts`
  - `cloud/src/limits.ts` (binding filter and counting upsert)
  - `cloud/src/sync.ts` (push, changes and files, with quota error mapping)
  - `cloud/src/purge.ts` (purges and the cron jobs)
  - `cloud/scripts/e2e-local.sh`
- Change:
  - `cloud/src/index.ts` (v2 router, `Origin` and content-type checks, `handle(request, env, now)`, `scheduled`)
  - `cloud/src/protocol.ts` (v2 patterns and limits)
- From b4e2733, rewritten: `cloud/src/accounts.ts` (admin planner, overcommit check, legacy commands), `cloud/scripts/admin.ts`, `cloud/scripts/tsconfig.json`
- Config: `cloud/wrangler.jsonc`, `cloud/package.json`, `cloud/vitest.config.ts`
- Tests:
  - `cloud/test/{helpers.ts, env.d.ts, setup.ts, worker.test.ts, conformance.test.ts, properties.test.ts}`
  - `cloud/test/{isolation.test.ts, auth.test.ts, usernames.test.ts, limits.test.ts, quota.test.ts, races.test.ts, migration.test.ts, admin.test.ts, purge.test.ts}`
- Docs and config: `cloud/README.md`, `cloud/DEPLOYED.md`, `.gitignore`, `.github/workflows/ci.yml` (e2e job)

**Rust client builder**

- Workspace: `Cargo.toml` (crypto dependencies), `crates/kanso-sync/Cargo.toml`
- 2a:
  - `crates/kanso-sync/src/{crypto.rs, username.rs, account.rs, binding.rs, keychain.rs, memory.rs, api.rs}`
  - `crates/kanso-store/src/{sync.rs, write.rs, lib.rs}` (`reset_sync`, `replace_library`, guard, `Stale`, generation bump)
  - `crates/kanso-sync/tests/{crypto.rs, accounts.rs, binding.rs, conformance.rs}`
  - `crates/kanso-sync/tests/fixtures/crypto-v1.json`
- 2b:
  - `crates/kanso-sync/src/{sealed.rs, wire.rs, remote.rs, engine.rs, http.rs, verify.rs, bin/kanso-sync.rs}`
  - `crates/kanso-sync/tests/{common/mod.rs, common/legacy.rs, sync.rs, properties.rs, canary.rs, rollback.rs, e2e.rs}`
- Delete: `crates/kanso-sync/src/credentials.rs`

**App builder**

- Create:
  - `crates/kanso-app/src/account_page.rs`
  - `crates/kanso-app/src/clipboard.rs` (`copy_secret`)
- Change:
  - `crates/kanso-app/src/config.rs`
  - `crates/kanso-app/src/services.rs`
  - `crates/kanso-app/src/settings.rs`
  - `crates/kanso-app/src/dialogs.rs`
  - `crates/kanso-app/src/sidebar.rs`
  - `crates/kanso-app/src/empty_state.rs` and `crates/kanso-app/src/editor.rs`
  - `crates/kanso-app/src/commands.rs` and `crates/kanso-app/src/palette.rs`
  - `crates/kanso-app/src/workspace.rs` (actions `OpenAccountSettings`, `ShowSignIn`, `ShowCreateAccount`, `ChangePassword`, `NewRecoveryKey`, `SignOut`; menu)
  - `crates/kanso-app/src/main.rs` (`HttpBackend`, startup reconciliation, `flush_pending_logouts`)
- Tests: `crates/kanso-app/src/{snapshots.rs, settings_window_tests.rs, ui_tests.rs}`
- Docs: `README.md` (accounts, Gatekeeper and Keychain prompts), `docs/PLAN.md` (Sync section points here)

**Sign-in methods (revision 3)**

- Worker:
  - Create: `cloud/migrations/0006_sign_in.sql`, `cloud/src/sign_in.ts` (health methods, passkey
    ceremonies, approvals, method list and event publication), `cloud/src/account_room.ts`
    (`ACCOUNT_ROOM`: authenticated hibernating sockets, no storage), `cloud/src/oauth.ts` (Apple,
    Google, GitHub: start, token exchange, JWKS, verification), `cloud/src/wellknown.ts` (AASA).
  - Change: `cloud/src/index.ts` (routes, the Apple callback's `Origin`/content-type exception,
    `/v2/account/events`), `cloud/src/auth.ts` (`has_password`, provider and passkey sessions,
    proofs, proof hash for the socket), `cloud/src/env.ts`, `cloud/src/protocol.ts`,
    `cloud/src/accounts.ts` (delete and admin), `cloud/src/purge.ts` (expired flows and approvals),
    `cloud/wrangler.jsonc`, `cloud/vitest.config.ts`.
  - Tests: `cloud/test/sign_in.test.ts`, `cloud/test/account_events.test.ts`,
    `cloud/test/fixtures/sign_in.json`, and the provider rows added to `auth.test.ts`.
- Rust:
  - Create: `crates/kanso-sync/src/{passkey.rs, security_key.rs, approval.rs, events.rs,
    provider.rs}`, `crates/kanso-sync/tests/{approvals.rs, sign_in.rs}`,
    `crates/kanso-sync/tests/fixtures/sign_in.json`.
  - Change: `crates/kanso-sync/src/{crypto.rs, account.rs, api.rs, http.rs, memory.rs, keychain.rs}`;
    `crates/kanso-sync/Cargo.toml` gains `tungstenite` and `rustls` for the account-event socket;
    both are already workspace dependencies in `Cargo.lock`. It does not gain the Linux libfido2
    binding.
- App:
  - Create: `crates/kanso-app/src/{approval.rs, provider_flow.rs, security_key_linux.rs,
    account_events.rs}` (or the same in `account_page.rs` and `account_flow.rs`), plus
    `crates/kanso-app/src/webauthn_macos.rs` and `crates/kanso-app/src/webauthn_windows.rs` (the
    native platform calls).
  - Change: `crates/kanso-app/src/{account_page.rs, account_flow.rs, account_words.rs, services.rs,
    config.rs, snapshots.rs, account_flow_tests.rs, workspace.rs, palette.rs}`.
  - Dependencies: `crates/kanso-app/Cargo.toml` and `Cargo.lock` gain the Linux-only libfido2
    binding.
  - Build: `scripts/bundle-macos.sh` (embed `Contents/embedded.provisionprofile`, the
    associated-domains entitlement, and the `kanso` URL scheme for the Apple callback),
    `packaging/macos/entitlements.plist` (new).
- Docs: `cloud/README.md`, `cloud/DEPLOYED.md`, `README.md` (the methods and the Keychain prompt),
  `docs/PLAN.md` (Sync points here).
