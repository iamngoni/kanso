# Kanso sync Worker

A Cloudflare Worker that Kanso clients sync through, protocol v2: accounts with end-to-end
encrypted libraries. Each device seals documents and files with a key the server never sees; the
Worker stores and returns those sealed values per account. D1 holds accounts, sessions, documents
and the counters behind quotas and rate limits; R2 holds sealed file bytes under
`accounts/<account_id>/<wire_id>`. The full design is [docs/ACCOUNTS.md](../docs/ACCOUNTS.md).

| Route | Session | |
|---|---|---|
| `GET /v2/health` | | Liveness, protocol version, whether sign-ups are open, the sign-in methods (`auth`) and the event socket (`events`) |
| `POST /v2/auth/prelogin` | | KDF parameters, salt and account ID for a username |
| `POST /v2/auth/signup` | | Create an account and this device's session |
| `POST /v2/auth/login` | | Sign in a device |
| `POST /v2/auth/recover/start`, `/finish` | | Recover with the recovery key, setting a new password |
| `POST /v2/auth/logout` | yes | Sign out this device |
| `GET /v2/account` | yes | Account, usage and quotas |
| `POST /v2/account/password`, `/recovery-key`, `/delete` | yes | Change the password, replace the recovery key, delete the account |
| `PUT /v2/account/device`, `GET /v2/account/devices`, `DELETE /v2/account/devices/<id>` | yes | Device names, the device list, signing one out |
| `POST /v2/push` | yes | Up to 200 sealed documents, each with the revision it was based on |
| `GET /v2/changes?since=&limit=` | yes | The account's change feed |
| `PUT` / `GET /v2/files/<wire_id>` | yes | Sealed file bytes, checked against `x-kanso-sha256` |
| `PUT` / `DELETE /v2/shares/<id>` | yes | Publish (or replace) a sealed note, or unpublish it |
| `GET /v2/public/<id>` | | A published note's ciphertext; 410 once expired, rate-limited per IP by the binding, no D1 writes |
| `GET /p/<id>` | | The viewer page: opens the note with the key in the link's `#fragment` |
| `PUT` / `GET /v2/account/identity`, `GET /v2/users/<username>` | yes | Identity keys for sharing, and another account's public keys |
| `/v2/collab/…` | yes | Shared notes: sharing, invitations, epochs, the log, snapshots, the live WebSocket, links, requests and notifications |
| `GET /j/<link_id>` | | The page an invite link opens: "open Kanso and paste this link" |
| `POST /v1/crash-reports` | | A crash report the person chose to send from the mobile apps ([docs/CRASH-REPORTS.md](../docs/CRASH-REPORTS.md)): no session, strictly validated, 204 stored, 5 an hour per address (`crash-ip`), kept 90 days, no IP or account stored |
| `POST /v1/metrics` | | An anonymous usage count from the apps ([docs/METRICS.md](../docs/METRICS.md)): no session, strictly validated, 204 once valid (passed on to PostHog with the country, whether or not PostHog takes it; nothing without `POSTHOG_KEY`), 60 an hour per address (`metrics-ip`), nothing stored, no IP passed on |
| `POST /v2/auth/passkey/start`, `/finish` | | Sign in with a passkey or security key |
| `POST /v2/auth/passkey/signup/start`, `/finish` | | Create a passwordless account with a passkey |
| `POST /v2/auth/{apple,google,github}/start` | | Start a provider sign-in or sign-up |
| `POST /v2/auth/apple/poll`, `/google/finish`, `/github/poll` | | Complete it: a session, a new-account ticket, or pending |
| `POST /v2/auth/apple/callback` | | Apple's `form_post` (form-encoded); answers an HTML page that returns to `kanso://auth/apple` |
| `POST /v2/auth/social/signup` | | Create an account from a provider ticket |
| `GET /v2/account/sign-in` | yes | The account's ways in: password, linked providers, passkeys |
| `POST /v2/account/sign-in/proof/start` | yes | A passkey challenge for a fresh proof |
| `POST /v2/account/sign-in/password` | yes | Set the first password of a passwordless account (takes a proof) |
| `POST /v2/account/sign-in/passkeys/start`, `/finish`, `/<credential_id>/remove` | yes | Add or remove a passkey (takes a proof) |
| `POST /v2/account/sign-in/identities/<provider>/start`, `/unlink` | yes | Link a provider, prove one, or unlink it |
| `/v2/account/sign-in/approvals…` | yes | Device approval: a new device gets the data key from a signed-in one |
| `GET /v2/account/events` | yes | The account's event WebSocket (approval hints; needs the device proof) |
| `GET`, `HEAD /.well-known/apple-app-site-association` | | The Associated Domains file macOS reads before it allows passkeys for this host |

A session is `Authorization: Bearer kanso_…`. Every other `/v1` path and every other path answer 404. Any request
with an `Origin` header gets 403, except on the public routes (published notes, the viewer and
invite pages, the AASA file) and on the Apple callback, which accepts only Apple's own origin
(`https://appleid.apple.com`, a browser form post carries it); a body with the wrong
`Content-Type` gets 415. Published notes are described in
[docs/ACCOUNTS.md › Published notes](../docs/ACCOUNTS.md#published-notes); the
`PUBLIC_LIMITER` binding (60 requests a minute per IP group) sits in front of both public routes. There is no
admin route: accounts are administered from the command line (below).

## Configuration

`wrangler.jsonc` binds the D1 database `kanso-sync` (`DB`), the R2 bucket `kanso-files` (`FILES`)
and two rate limiters that write nothing to D1: `AUTH_LIMITER` (20 requests a minute per IP group,
in front of `/v2/auth/*` and `/v2/health`) and `SESSION_LIMITER` (120 requests a minute per
account, on every session route but logout). It runs the cron every 15 minutes (purges, expired
rate-limit rows and sessions, R2 reconciliation). Workers Logs keeps only the Worker's own lines
(route names and error codes); invocation logs and traces are pinned off. `wrangler tail` still
shows each request's URL and metadata live to whoever runs it (see `DEPLOYED.md`).

One secret is required, `AUTH_PEPPER`: 32 random bytes as base64url. It keys the fake answers
`prelogin` gives for unknown names and the rate-limit keys of IP addresses. Without it, health and
the sign-in routes answer 503 `misconfigured`; signed-in devices keep working. `SYNC_TOKEN` is no
longer read.

### Sign-in methods

Passkeys, Linux security keys, Sign in with Apple, Google and GitHub, device approval and the
account event socket are described in
[docs/ACCOUNTS.md › Sign-in methods](../docs/ACCOUNTS.md#sign-in-methods). Each method is optional
per deployment: `GET /v2/health` advertises only what is configured, and the app shows only those.

| Name | Kind | What it is |
|---|---|---|
| `PASSKEY_RP_ID` | var | The exact public host passkeys are scoped to (`sync.kansoapp.app`). Unset, or an IP, means no passkeys. |
| `PASSKEY_ORIGINS` | var, JSON | `["https://sync.kansoapp.app"]`; each must be the RP ID or under it. |
| `APP_ASSOC_APPS` | var, JSON | `["4973Y692JU.app.kanso.Kanso"]`: the apps allowed to use this host's passkeys, served in the AASA file. |
| `APPLE_CLIENT_ID`, `APPLE_TEAM_ID`, `APPLE_KEY_ID`, `APPLE_REDIRECT_URI` | vars | The Services ID, team, key ID and the exact callback URL. |
| `APPLE_PRIVATE_KEY` | secret | The Sign in with Apple `.p8` key, PKCS#8 PEM. |
| `GOOGLE_CLIENT_ID` | var | The Desktop-app OAuth client ID. |
| `GOOGLE_CLIENT_SECRET` | secret | Its client secret. |
| `GITHUB_CLIENT_ID` | var | The OAuth App's client ID. GitHub uses the device flow only, which needs no secret. |

Secrets go through `npx wrangler secret put NAME`, reading stdin, and never into a file, a commit
or a chat. Vars go in `wrangler.jsonc` (they are listed there, commented out, until the owner
switches them on).

`GET /v2/health` adds two objects. `auth` is `{"password": true, "passkeys": null | {"rp_id",
"origins", "security_key": true}, "providers": [...]}`, where a provider appears only when every
value its flow needs is present. `events` is `{"account": "/v2/account/events"}`. A server without
`auth` (an older Worker) means username and password only; without `events`, the app relies on its
GET fallbacks.

**The AASA file.** `https://sync.kansoapp.app/.well-known/apple-app-site-association` is served by
the Worker itself (not from `public/`): `{"webcredentials":{"apps":[…]}}` from `APP_ASSOC_APPS`,
`application/json`, `Cache-Control: public, max-age=300`, never a redirect, and 404 when the var is
unset or malformed. macOS fetches it through Apple's CDN, which caches it, so a change can take
hours to reach Macs. Sign in with Apple needs no file on the server.

**Sign in with Apple on every platform.** The callback page redirects to
`kanso://auth/apple?flow=<flow_id>&s=<secret>` and shows a "Return to Kanso." link. On macOS,
ASWebAuthenticationSession catches it. On Windows and Linux the system browser hands it to the
`kanso://` scheme handler the app registers. The Worker doesn't depend on the platform: the poll
must send `s`, whose SHA-256 the callback stored (migration 0008), and only the process that
received the redirect has it.

**Migrations.** `0006_sign_in` (passkeys, linked identities, sign-in flows, device approvals and
`has_password`), `0007_provider_completion` (idempotent provider completions), `0008_apple_app_secret`
(the Apple flow's `app_secret_hash` and `app_claimed_at`), `0009_socket_closes` (the admin
script's requests to close event sockets) and `0010_end_flows_with_sessions` (a session's end, or
an account's disable, consumes the link, proof and passkey flows it started). All are additive; apply them with
`npm run migrate:remote` before deploying the Worker that uses them.

**What is stored.** No email, name, profile data or provider token is stored, only each provider's
stable subject. The GitHub device code is sealed under `AUTH_PEPPER`. A passkey's PRF output never
leaves the app: the Worker stores only the data key wrapped under a key derived from it.

**Event sockets close when a session ends.** These close sockets at once: sign-out, device
revocation, a password change or first password (every other device), a recovery, eviction by the
ten-device limit (at every sign-in and sign-up route), account deletion, and the 180-day expiry
(when the expired session's next request is refused, and in the cron's sweep). The admin script's
`disable` and `delete` can't reach a Durable Object, so they leave a row in `socket_closes` and the
cron closes those sockets within 15 minutes.

## Shared notes

Shared notes ([docs/ACCOUNTS.md › Collaboration](../docs/ACCOUNTS.md#collaboration)) add a
Durable Object binding, `NOTE_ROOM` (class `NoteRoom`, `src/room.ts`): one room per shared note,
keyed by share ID, holding the note's sealed log in its own SQLite storage and its members' live
WebSockets (hibernating). The room is reachable only through the Worker. `wrangler.jsonc` declares
the class with the Durable Object migration `v1` (`new_sqlite_classes: ["NoteRoom"]`), which
`wrangler deploy` applies by itself.

D1 needs `migrations/0005_collab.sql` (identities, shares, members, wraps, links, requests,
notifications, retired IDs and room purges; additive only) **before** the Worker that uses it is
deployed: run `npm run migrate:remote`, then `npm run deploy`. The cron purges the rooms of shares
that stopped or whose owner was deleted (`collabPurged` in its report) and drops expired links.

## Deploying

Needs Node, and a Cloudflare API token that can edit Workers, D1 and R2. The token is only ever
referenced by name; never write it into a file.

```bash
cd cloud
npm ci
export CLOUDFLARE_API_TOKEN="$IMNGONII_CLOUDFLARE_API_TOKEN"

# First deployment only: create the database and bucket, and put the printed
# database_id in wrangler.jsonc.
npx wrangler d1 create kanso-sync
npx wrangler r2 bucket create kanso-files

# The pepper, once (read from stdin, never echoed or saved).
pepper=$(openssl rand 32 | base64 | tr '+/' '-_' | tr -d '=\n')
printf %s "$pepper" | npx wrangler secret put AUTH_PEPPER
unset pepper

# Optional: short links for published notes (src/rerout.ts). Paste a key of the Kanso project
# on Rerout; without it, published notes keep their long links.
npx wrangler secret put REROUT_API_KEY

npm run migrate:remote     # 0002 to 0010 are additive; sign-ups start closed
npm run deploy
```

Losing the pepper only changes the fake prelogin answers and resets rate-limit counters, so a
backup in a password manager is optional.

### Moving the existing deployment to accounts

The deployed Worker still runs v1 ([DEPLOYED.md](DEPLOYED.md)). The owner's move follows
[docs/ACCOUNTS.md › Deploy and the owner's migration](../docs/ACCOUNTS.md#deploy-and-the-owners-migration),
which has every command; in outline:

1. With the v1 build, sync every Mac until nothing is pending, then stop editing.
2. Back up: Settings › General › Back up now, a copy of `Kanso.app`, the D1 Time Travel bookmark,
   and `npm run -s admin -- legacy-export --dir ~/KansoLegacy-<date>` (the old plaintext; keep it).
3. Set `AUTH_PEPPER`, `npm run migrate:remote` (adds `0002_accounts`, sign-ups closed), then
   `npm run deploy`. From here `/v1` answers 404.
4. Install the new app and run `kanso-sync estimate`. Then, immediately before you create your
   account in Settings › Account, run `npm run -s admin -- signups only <you>`: until your account
   exists, anyone who guesses the name can take it. If the app says the name is taken, run
   `npm run -s admin -- delete <you> --confirm <you>`, then `signups only <you>` again, and create
   the account at once. Save the recovery key. If the estimate is over a
   default quota, raise your own account (`quota <you> …`); the daily budget of 20,000 writes
   covers a first upload of the current library several times over.
5. Wait for "Synced", quit Kanso, and run `kanso-sync verify --legacy …/manifest.json --out
   …/verified.json`; it must exit 0. Sign the other Macs in one at a time with Add.
6. `npm run -s admin -- purge-legacy --account <you> --verified …/verified.json` drops the old
   tables, queues the `files/` purge and deletes `SYNC_TOKEN`. Then `signups open` if others are
   to join, remove the old `sync-token` Keychain item, and update DEPLOYED.md.

Before step 6, `kanso-sync unbind` on each Mac, `npx wrangler rollback` and the old app go back to
v1 ([Rollback](../docs/ACCOUNTS.md#rollback)).

## Administering accounts

```bash
CLOUDFLARE_API_TOKEN="$IMNGONII_CLOUDFLARE_API_TOKEN" npm run -s admin -- accounts
npm run -s admin -- --local accounts        # the local database (wrangler dev's), no token needed
```

| Command | |
|---|---|
| `accounts` | Every account's usage against its quotas, writes today against its budget, bytes uploaded today, devices, last seen and status; the settings, global sums and pending purges |
| `settings` | The settings |
| `signups open` / `signups close` / `signups only <username>` | Open sign-ups, close them, or reserve them for one name. Run `only` immediately before that name signs up; if someone took the name first, `delete` it and run `only` again |
| `disable <username>` / `enable <username>` | Refuse, or allow again, the account's sign-ins and sessions; a disabled account's published notes aren't served |
| `shares <username>` | The account's published notes: share IDs, sizes, times and expiries (never content or keys) |
| `unpublish <share-id>` | Take a published note down (the ID after `/p/` in its link): the link answers 404 and the ID is retired for good |
| `delete <username> --confirm <username>` | Delete the account's rows and queue its R2 purge |
| `max-accounts <n> [--force]` | At most n accounts |
| `quota <username> [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--writes-per-day N] [--force]` | Per-account quotas, including the account's own daily write budget; `quota <username> --default` goes back to the defaults |
| `default-quota [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--force]` | The default quotas |
| `writes-per-day <n> [--force]` | Documents pushed plus files uploaded, per account per UTC day (the default) |
| `upload-bytes-per-day <n> [--force]` | Uploaded file bytes, per account per UTC day |
| `global-writes-per-day <n> [--force]` / `global-upload-bytes-per-day <n> [--force]` | Server-wide daily caps |
| `recount [<username>]` | Recompute the usage counters from the tables |
| `restore <time \| bookmark> [--confirm <time \| bookmark>]` | The only way to restore D1: Time Travel back to a Unix time in seconds, an RFC 3339 time or a bookmark, then at once a new epoch for every account. Without `--confirm`, prints what it would run |
| `restored [<username> \| --all]` | A new epoch, so each device re-merges; `restore` runs it for you |
| `legacy-status` | The single-user tables and the `files/` purge |
| `legacy-export --dir DIR` | `manifest.json`, `legacy.sql` and every live legacy file, into DIR |
| `purge-legacy --account <username> --verified R.json` | Drop the single-user tables, queue the `files/` purge and delete `SYNC_TOKEN`, once `R.json` (from `kanso-sync verify --legacy`) proves everything arrived |

**Daily budgets.** Each account may push documents plus upload files up to `writes_per_day`
(20,000) per UTC day, and upload at most `upload_bytes_per_day` (1 GiB, 1,073,741,824 bytes) of
file bytes, refused uploads and replacements included. The server as a whole stores at most
`global_writes_per_day` (300,000) document writes and `global_upload_bytes_per_day`
(5,000,000,000) upload bytes a day, shared fairly: past a server-wide cap only accounts above their
fair share (the cap ÷ `max_accounts`, 6,000 writes and 100,000,000 bytes by default) are refused,
so fifteen accounts at their write budget, or five at their upload budget, can no longer pause
everyone else until UTC midnight. The price is that a day's total stays under about twice each cap.
Each daily command, and `max-accounts`, prints the fair shares and how many accounts reach each
cap; `accounts` shows them beside today's counts. Past its budget an account's devices show "Sync
paused" and carry on after UTC midnight.
A library that needs more in one day (a first upload or a re-merge of a large library) can be given
its own budget with `quota <username> --writes-per-day N`; the owner does this for their own
account rather than raising the default. The daily commands refuse, unless given `--force`, a
value that lets one account's budget exceed the server's.

`max-accounts`, `quota` and `default-quota` refuse a change after which the quotas would promise
more than a global cap, unless given `--force`. Every value is validated before it reaches SQL.
The exit status is 0 on success, 1 when a command is refused or wrangler fails, and 2 for a usage
error. With `--local`, `--persist-to DIR` selects a state directory other than `.wrangler/`
(`legacy-export` can't use it: `wrangler d1 export` only reads the default one).

## Local development

```bash
npm run migrate:local
npx wrangler d1 execute kanso-sync --local --command "UPDATE settings SET value = 'open' WHERE name = 'signup'"
npx wrangler dev --var AUTH_PEPPER:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8
```

That pepper is the fixed test value (bytes 0 to 31) that `vitest.config.ts` also uses; never deploy
it. `.dev.vars` and `.wrangler/` are ignored by git.

## Tests

```bash
npm test          # vitest in the Workers runtime; fast-check seed 20261017 (FAST_CHECK_SEED, FAST_CHECK_RUNS)
npm run typecheck # the Worker and tests, then scripts/
scripts/e2e-local.sh
```

`test/fixtures/` holds cases the Worker and the Rust client's in-memory server must both pass:
`usernames.json`, `auth.json`, `conformance.json` and `sign_in.json` (protocol v2; `sign_in.json`
adds Worker vars, a deterministic Ed25519 authenticator and a fake GitHub, described in its
`about`), and `conformance-v1.json`,
which this Worker no longer serves: only the Rust rollback test's stand-in for the old Worker
still runs it. `scripts/e2e-local.sh` starts the Worker under `wrangler dev` on a throwaway state
directory (on a free port the OS picks, or `KANSO_E2E_PORT`; a port in use is refused) and runs `cargo test -p
kanso-sync --test e2e` against it at production KDF parameters, all in Miniflare with no
Cloudflare credentials. Once the Worker has stopped, a second test scans every file of its state
for the first one's canaries (password, recovery key, a marker in every note and name, revision
hashes, the deleted account's sealed files) and must find a live witness account in D1 and R2.
`KANSO_E2E_KEEP=1` keeps the state directory.
