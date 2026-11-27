# Deployed

https://sync.kansoapp.app (the apps' default), https://share.kansoapp.app (published notes) and
https://kanso.antonlabs.cc (its first name, still answering), all one Worker.

Since 2026-10-21 it answers on the kansoapp.app names too (Worker version
`8748e19d-dae3-4a49-9bc2-9d1a37a7d9a0`), with `SHARE_ORIGIN` set so short links lead to
share.kansoapp.app. The two short links made before then were pointed there by hand.

Since 2026-10-19 this runs the accounts Worker (protocol v2, migration `0002_accounts`,
`AUTH_PEPPER`), served only on the custom domain `kanso.antonlabs.cc`; the old
`kanso-sync.imngonii.workers.dev` address is off (`workers_dev: false`). The D1 Time Travel
bookmark from before the migration is `000000c8-00000000-00005108-c70da56708596ff0458be5e7c33f0f4e`;
restore only with `npm run -s admin -- restore <bookmark>`.

Since 2026-10-20 it also runs migration `0003_shares` (published links: the `shares` table, with
ciphertext in R2 under `shares/<id>`), deployed as Worker version
`f9acbb57-2eb8-4302-a8fd-a5e62bdc468d`.

Since 2026-10-21 (with Kanso 0.7.0) it also runs migration `0005_collab` (shared notes:
identities, `collab_*` tables) and the `NoteRoom` Durable Object (one per shared note, SQLite
storage, migration tag `v1`), deployed as Worker version `a0084598-4b14-4106-a56c-329721b1e89e`.
The D1 Time Travel bookmark from just before the migration is
`000005fe-00000000-0000510b-6bef7dc050eff27b48f8086073e12b82`.

Since 2026-10-25 it runs sign-in (issue #32): migrations `0006_sign_in` to `0010_end_flows_with_sessions`
(passkeys, security keys, Sign in with Apple, Google and GitHub, device approvals, account events
on the `AccountRoom` Durable Object), with the step-7 vars and the `GOOGLE_CLIENT_SECRET` and
`APPLE_PRIVATE_KEY` secrets set, deployed as Worker version `f3d2c292-04c0-4998-bd16-25b91b587a63`.
`/.well-known/apple-app-site-association` serves `4973Y692JU.app.kanso.Kanso`. The D1 Time Travel
bookmark from just before the migrations is
`00000b40-00000000-0000510f-b10566111c0634a2bf7087e3b325f71d`.

Since 2026-10-26 it runs push notifications (issue #48 step 8b): migration `0011_push` (push
columns on `sessions`, the clearing trigger) and the `AccountRoom` push alarm, with the
`APNS_TOPIC` var, deployed as Worker version `d5825e9b-6f8f-446e-89c4-b3eaeb657983`. Until the
`APNS_KEY_P8`, `APNS_KEY_ID`, `APNS_TEAM_ID` and `FCM_SERVICE_ACCOUNT` secrets are set, it stores
devices' tokens and sends nothing. The D1 Time Travel bookmark from just before the migration is
`00000dc9-00000000-00005110-7106e37824a26e2b3a16d4b6b2588907`.

Since 2026-10-30 it accepts crash reports from the mobile apps (issue #48 step 13, `POST
/v1/crash-reports`) and sealed attachments up to the 10 MiB limit plus 4 KiB (issue #44):
migrations `0012_crash_reports` and `0013_sealed_file_size`, deployed as Worker version
`3567154c-62b9-423e-8636-4b1363c0fb15`. `0013` rebuilds `account_files`; its 51 rows, their sizes and
upload times, and the three upload triggers (`uploads_global`, `uploads_count_insert`,
`uploads_count_update`) were checked the same before and after. The D1 Time Travel bookmark from
just before the migrations is `0000153b-00000000-00005114-16ce93800a1e5f0b05eb474d2cd0d582`.

## Restoring D1

Only with `npm run -s admin -- restore <time | bookmark>`, run first without `--confirm` to see
what it does. Never with `wrangler d1 time-travel restore` or the dashboard: a restore alone
rewinds every account while each device still trusts its cursor, and `restore` follows it at once with
a new epoch for every account so that each device re-merges. It prints the bookmark to undo with. A
restore also brings back the sessions, passwords and recovery keys of that time: revoke again
any device revoked since, and anyone who changed a password or recovery key since must change it
again.

## What the deployment records

`wrangler.jsonc` pins what Cloudflare keeps: Workers Logs stores only the Worker's own log lines
(route names and error codes), while invocation logs, which would store each request's URL, method
and metadata, and traces are off. Metrics stay on.

Live tailing can't be switched off. While someone with access to the Cloudflare account runs
`wrangler tail`, or opens the dashboard's live logs, they see each request as it happens: its URL
(document, file and device IDs, which are opaque), method, headers and Cloudflare's metadata about
the connection, such as the country it came from. Cloudflare redacts some values, such as the
`Authorization` header, by heuristic rather than by guarantee. A tail stores nothing unless its
viewer saves the output, so tail only to debug, and don't keep what it prints.

Since 2026-11-25 it runs migration `0023_auth_flow_app_scheme` and everything merged up to then,
including the feedback sheet's GitHub user search and credit (#499, `GET /v1/feedback/users`),
deployed as Worker version `8e598b00-6103-405a-bac5-054168fa2c20`. Before this, the Worker lagged
master and the apps' search and credited reports failed (#499 was merged without a deploy).

Since 2026-11-25 (later) it runs the fuller feedback triage (#507: longer summaries, a Screenshot
line), deployed as Worker version `089f046f-9e92-46ad-85a9-ba3994d14524`.

Since 2026-11-26 the viewer page lays published notes out at 75% of the window on screens wider
than 48em, with diagrams filling the column, for pages published before the renderer did the
same; deployed as Worker version `c418e031-0062-46db-8e60-0385148e4ceb`.
