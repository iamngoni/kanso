# Usage metrics

Kanso's apps send a small anonymous count, so we know how many people use Kanso, how, and on
which devices. There is no third-party SDK in the apps. This page is the one definition of what
may be sent: the shared client (`crates/kanso-metrics`) produces exactly this, and the sync Worker
(`cloud/src/metrics.ts`) refuses anything else before it passes the count on to PostHog.

What people read in the app is the note at [kansoapp.app/metrics](https://kansoapp.app/metrics)
(its text is [METRICS-NOTE.md](METRICS-NOTE.md)). It must change when this page does.

## The rules

1. **Never anything a person wrote.** No note text, titles, notebook or tag names, file names,
   theme or plugin names, usernames, email addresses, or anything typed. Every value is from a
   closed list or matches a pattern below. A value that doesn't fit is replaced with `unknown` (or
   `other`, `custom`) on the device, and the Worker refuses the whole batch if one still arrives.
2. **No address, no account.** The Worker reads the country Cloudflare gives a request and never
   passes the IP address on. PostHog gets no account ID, username, device ID or session.
3. **Said before it starts.** Nothing is sent until the person has seen the notice (below) and
   pressed OK. Metrics can't be turned off; the notice and this page say so.
4. **Releases only.** Debug builds, Kanso Dev, the mobile development entry point and tests never
   send.

## When a batch is sent

A batch is sent at launch, every hour while the app is open, and when the app quits (desktop) or
goes to the background (phones). A send with nothing due sends nothing.

The app keeps a small **ledger** on the device (desktop: `metrics` in the config; phones:
`mobile.metrics.ledger` in the library's SQLite metadata) of what has been counted and not yet
reported. A batch is built from it, and only an answer that settles the batch (`204`, or `400` /
`413` which would only fail again) clears what that batch carried; the ledger is saved after every
answer. So a restart never counts anything twice, and a batch that got no answer is built again.
Counts that arrive while a batch is on the wire stay for the next one. Anything more than a day old
is dropped, never sent late.

## Who is counted

Each batch carries one `id`:

| `id` | When | What it is |
|---|---|---|
| `a_` + 32 hex | the device is signed in to an account and holds its data key | the first 16 bytes of HKDF-SHA256(data key, no salt, info `"kanso/v1/metrics-id"`). Every device of the account computes the same value, so they count as one person. The sync server never holds the data key, so neither it nor PostHog can tell which account an ID belongs to. |
| `i_` + 32 hex | otherwise | 16 random bytes made the first time it is needed and kept on this device (in the ledger above). Never sent anywhere else. |

Signing in changes a device's ID from its `i_` to the account's `a_`; that day it can be counted
under both. Signing out goes back to the same `i_`.

## Events

Every event carries `platform`, `form` and `app_version`:

| Prop | Values |
|---|---|
| `platform` | `macos` \| `windows` \| `linux` \| `ios` \| `android` |
| `form` | `desktop` \| `phone` \| `tablet` (desktop platforms are always `desktop`) |
| `app_version` | `^[0-9]{1,4}(\.[0-9]{1,4}){2}$` |

### `app_active`

At most once per UTC day per `id`, when Kanso is open (at launch, and when the day changes while it
is open). This is the daily-active count.

| Prop | Values |
|---|---|
| `os_version` | `^[0-9A-Za-z._-]{1,32}$`, else `unknown` |
| `sync` | `off` \| `kanso` (Kanso's own server: `sync.kansoapp.app`, `kanso.antonlabs.cc`) \| `self_hosted` |
| `ai` | `true` \| `false`: AI is turned on |
| `notes` | `0` \| `1-9` \| `10-99` \| `100-999` \| `1000+`: notes in the library, as a range only |

These are always present. The props below are **optional**: an app sends one only if it can read it
cleanly on that platform, and leaves it out otherwise (see [Platform availability](#platform-availability)).

| Optional prop | Values |
|---|---|
| `locale_lang` | the UI language as an ISO 639-1 code (`^[a-z]{2}$`) from the languages the app ships, else `other` |
| `theme_mode` | `light` \| `dark` \| `system` |
| `theme` | the ID of one of Kanso's built-in themes (`^[a-z0-9][a-z0-9-]{0,31}$`, e.g. `kanso-dark`), or `custom` for any theme a person made, imported or downloaded from the gallery. Never a custom theme's name. |
| `editor_mode` | `edit` \| `live` \| `preview` \| `split`: the default mode |
| `plugins` | `0` \| `1-2` \| `3-5` \| `6+`: installed plugins |
| `notebooks` | `0` \| `1-4` \| `5-19` \| `20+` |
| `attachments_mb` | `0` \| `<10` \| `10-100` \| `100-1000` \| `1000+`: what attachments take, in megabytes (1 MB is 1,000,000 bytes; a range includes its lower end and not its upper) |
| `sync_errors` | `0` \| `1-5` \| `6+`: sync failures since the previous `app_active` was reported |
| `channel` | `stable` \| `beta` \| `dev`: the release line the app was built from |

### `app_opened`

Once per launch, at most 12 a day per `id`. No props beyond the common three.

### `session`

When a session ends: on quit (desktop) or when the app goes to the background (phones).

| Prop | Values |
|---|---|
| `duration` | `<1m` \| `1-5m` \| `5-15m` \| `15-60m` \| `1-3h` \| `3h+` |

### `feature_used`

Once per feature per batch in which it was used, with how often since the last batch.

| Prop | Values |
|---|---|
| `feature` | one of the list below |
| `count` | `1` \| `2-5` \| `6-20` \| `21+`: uses since the last batch |
| `source` | only, and always, with `import`: `markdown` \| `obsidian` \| `logseq` \| `notion` \| `evernote` \| `bear` \| `joplin` \| `standard_notes` \| `simplenote` \| `upnote` \| `apple_notes` \| `pouch` \| `other` |

| `feature` | Counted when |
|---|---|
| `note_created` | a note is created (by the person, not by sync or an import) |
| `voice_input` | dictation puts words in a note |
| `publish` | a note is published as a link |
| `import` | an import finishes |
| `lock` | a note or notebook is locked |
| `ai` | a question, an inline assistant request or a summary is sent to a model or agent |
| `quick_capture` | quick capture (desktop) or a share into Kanso (phones) saves a note |
| `shared_note` | a note is shared with another account |
| `export` | a note is exported (web page, PDF, Markdown) |
| `note_edited` | a note is edited; once per note per batch window, so it counts notes edited, not keystrokes |
| `note_deleted` | a note is deleted (moved to the trash) |
| `note_restored` | a note is restored from the trash or from a version |
| `note_pinned` | a note is pinned |
| `note_moved` | a note is moved to another notebook |
| `notebook_created` | a notebook is created |
| `tag_added` | a tag is added to a note |
| `template_used` | a note is made from a template |
| `daily_note` | the daily note is opened or created |
| `search` | a search is run |
| `wikilink_followed` | a `[[link]]` is followed |
| `backlinks_opened` | the backlinks view is opened |
| `graph_opened` | the graph view is opened |
| `task_toggled` | a task's checkbox is ticked or unticked |
| `attachment_added` | a file or image is attached to a note |
| `table_inserted` | a table is inserted |
| `diagram` | a diagram is inserted |
| `sticky_opened` | a sticky note is opened |
| `new_window` | a note opens in a new window |
| `focus_mode` | focus mode is turned on |
| `plugin_installed` | a plugin is installed |
| `plugin_command` | a plugin's command is run |
| `theme_changed` | the theme is changed |
| `sync_conflict_resolved` | a sync conflict is resolved |
| `review_prompt` | the app asks the system to show its review prompt (phones; the system decides whether it shows). Nothing about the review, or whether one was written, is known or sent |

Phones count only the features that exist there: see [Platform availability](#platform-availability).
Only the feature's name and a range are sent: never a title, text, tag name, notebook name, search
term, link target, theme name or plugin name.

No other event, prop or value exists. A new one is added here first. (`server_stats`, below, is
sent by the Worker itself; the Worker refuses it from an app.)

## Platform availability

What each platform sends of the optional `app_active` props and of the `feature_used` list. A
platform leaves out what it can't read cleanly rather than guess.

| Item | Desktop | Phones |
|---|---|---|
| `locale_lang` | `en` (the only language the desktop ships) | the app's language from the languages it ships, else `other` |
| `theme_mode`, `theme` | sent | sent |
| `editor_mode` | sent | left out (phones have no default editor mode) |
| `plugins` | sent | left out (phones have no plugins) |
| `notebooks` | sent | sent |
| `attachments_mb` | sent | sent |
| `sync_errors` | sent | sent |
| `channel` | `stable`, `beta` for a pre-release version, `dev` for Kanso Dev | `stable` |
| features not counted | `review_prompt` (the desktop never asks for a review) | `table_inserted`, `diagram`, `sticky_opened`, `new_window`, `plugin_installed`, `plugin_command` (they don't exist on phones) |

## The batch (JSON, version 1)

```json
{
  "v": 1,
  "id": "a_3f9c0d4e5b6a79881726354433221100",
  "events": [
    {
      "event": "app_active",
      "day": "2026-10-30",
      "props": {
        "platform": "macos",
        "form": "desktop",
        "app_version": "0.10.0",
        "os_version": "15.1",
        "sync": "kanso",
        "ai": false,
        "notes": "100-999",
        "locale_lang": "en",
        "theme_mode": "system",
        "theme": "kanso-dark",
        "editor_mode": "live",
        "plugins": "1-2",
        "notebooks": "5-19",
        "attachments_mb": "<10",
        "sync_errors": "0",
        "channel": "stable"
      }
    },
    {
      "event": "app_opened",
      "day": "2026-10-30",
      "props": { "platform": "macos", "form": "desktop", "app_version": "0.10.0" }
    },
    {
      "event": "session",
      "day": "2026-10-30",
      "props": { "platform": "macos", "form": "desktop", "app_version": "0.10.0", "duration": "15-60m" }
    },
    {
      "event": "feature_used",
      "day": "2026-10-30",
      "props": { "platform": "macos", "form": "desktop", "app_version": "0.10.0", "feature": "import", "source": "obsidian", "count": "1" }
    }
  ]
}
```

| Field | Pattern |
|---|---|
| `v` | `1` |
| `id` | `^(a\|i)_[0-9a-f]{32}$` |
| `events` | 1–32 events |
| `event` | `app_active` \| `app_opened` \| `session` \| `feature_used` |
| `day` | `^[0-9]{4}-[0-9]{2}-[0-9]{2}$`, the device's UTC day, within two days of the Worker's |
| `props` | exactly the props listed for the event: the required ones, and any of the optional `app_active` props; `count` and `source` as above; no other key |

A batch from an 0.9.0 client (no optional props, no `count` on `feature_used`, no `app_opened` or
`session`) is accepted unchanged. The whole body is at most 16 KiB. No other field is allowed, at
any level.

## Sending

`POST https://sync.kansoapp.app/v1/metrics` with `content-type: application/json` and the batch
as the body. This is Kanso's metrics Worker even when the person's notes sync with a self-hosted
server, so the `sync` property stays a property of the library, not the metrics destination. No
session, cookie, advertising ID or hardware identifier is sent.

| Status | Meaning | The app |
|---|---|---|
| `204` | accepted | clears what the batch carried from the ledger |
| `400` | not a valid batch (the body names the field, never echoing a value) | clears it too; it would fail again |
| `413` | over 16 KiB | as `400` |
| `429` | too many batches from this address | keeps them, tries again at the next send |
| other, or no answer | | keeps them, tries again at the next send |

Anything more than a day old is dropped, not sent late. The Worker limits each IP group (a keyed
hash, `metrics-ip` in the `limits` table) to 600 batches an hour, so many people behind one network
address are not dropped.

## The Worker and PostHog

The Worker checks the batch against this page, adds `country` (the two-letter code from
`request.cf.country`, left out when there is none) and sends it to PostHog's batch API
(`POST ${POSTHOG_HOST}/batch/`) with:

- `distinct_id`: the batch's `id`
- `timestamp`: the time the Worker received the batch; for an event whose `day` is before today
  (UTC), `<day>T23:59:59Z` so it stays on its day. Never later than receipt.
- `properties`: the props, `country`, `$process_person_profile: false` (no person profiles),
  `$geoip_disable: true`, `$lib: "kanso-worker"`

It answers `204` whether or not PostHog took it: a lost count isn't worth a retry. Without
`POSTHOG_KEY` it checks the batch and sends nothing. It stores nothing in D1 for a batch; the
sender is only counted by the `limits` table (`metrics-ip`, a keyed hash of its IP group, 600
batches an hour).

Configuration: `POSTHOG_HOST` is a plain var (`https://eu.i.posthog.com`); the project's API key
is a secret, `wrangler secret put POSTHOG_KEY`. In the PostHog project, turn on **Discard client
IP data** and leave autocapture, session replay, surveys and heatmaps off (they need a browser SDK,
which Kanso doesn't load). The key must be the project's API key (`phc_…`) from the same region
as `POSTHOG_HOST`.

`/batch/` answers `200` even for a key it doesn't know, and drops the events. So once per isolate
the Worker also asks `POST ${POSTHOG_HOST}/flags/?v=2` with the key, which does refuse an unknown
one, and logs `{"route":"posthog","outcome":"key_rejected"}` when it does. Each batch PostHog takes
logs `{"route":"posthog","outcome":"ok","events":n}`, and each failure its status or error name.
None of these lines carries an ID, a prop or the key.

## Server metrics

The sync server counts itself too, as aggregates only. On the existing `*/15` cron the Worker sends
one `server_stats` event to PostHog with `distinct_id` `kanso-sync-server`, for the window since
the last run. Requests are counted into D1 counter rows per window, batched off the request path
(`ctx.waitUntil`), so a request is never slowed by it. It counts:

- requests by route group and status class (`2xx`, `3xx`, `4xx`, `5xx`);
- p50 and p95 latency, as buckets;
- new accounts;
- distinct devices seen in the hour, counted from the existing tables (a number only);
- share and publish page views;
- the number of 5xx responses.

The event's props are flat. All are counts (numbers) or bucket labels, and every one is always
present (zero when unused):

| Prop | Value |
|---|---|
| `window_minutes` | `15` |
| `requests_total`, `requests_2xx`, `requests_3xx`, `requests_4xx`, `requests_5xx` | counts for the window |
| `errors_5xx` | the 5xx count (same as `requests_5xx`) |
| `req_<group>_<class>` | one count for each route group (`auth`, `account`, `sync`, `publish`, `publish_page`, `share_page`, `collab`, `telemetry`, `other`) and status class (`2xx`, `3xx`, `4xx`, `5xx`) |
| `latency_p50`, `latency_p95` | one of `<50ms`, `50-100ms`, `100-250ms`, `250-500ms`, `500-1000ms`, `1000ms+`, or `none` for an empty window |
| `new_accounts` | accounts created in the window |
| `devices_seen` | distinct devices whose session was seen in the hour ending with the window (counted, never sent). Present only on the event whose window ends on the hour, so events don't overlap; sessions are stamped at most hourly, so each device is counted about once an hour |
| `share_views` | successful share page views |
| `publish_views` | successful publish page views |
| `$process_person_profile`, `$geoip_disable`, `$lib` | as for client events (`false`, `true`, `kanso-worker`); no `country` |

Each isolate counts in memory and writes the counts to the `request_counts` table in D1 in one batch
at most once a minute (off the request path, after the response), counting the minute from its first
counted request; the cron also writes what its own isolate holds. Rows are keyed by 15-minute window,
route group, status class and latency bucket. A window is finished once a whole window has passed
since it ended. Each cron run sends every finished window that still has rows, oldest first and at
most 8 per run, plus the newest finished window even when empty (a quiet period reads as zeros), each
stamped where its window ended. A window's rows are deleted only after PostHog took its event, so a
missed cron or a refusal leaves it for the next run; a window still unsent after a day is dropped.
`devices_seen` reads the sessions seen in one hour through an index on `last_seen_at`. Residual loss:
an isolate that is recycled or evicted before its next flush loses what it held (up to a minute of
its requests), and an idle isolate is not flushed until a later request or the cron reaches it.

No IP address, account or device ID, token or path with a token is counted or sent: route groups
are fixed names, never paths.

## The notice

The first time Kanso opens with this version of the feature (a new install or an update), it
shows once:

> **Kanso counts, it doesn't watch**
>
> To know how many people use Kanso and on which devices, Kanso sends a small anonymous count every hour while it is open, when you open it and when you close it: the kind of device, Kanso's version, your language, theme and editor mode as a plain label, how many notes, notebooks and plugins you have and how much room attachments take as ranges, how long it was open, and how often features were used, as ranges. Never your notes, titles, tags, files, name or anything you type, and never the name of a theme or plugin. Devices signed in to the same account count as one person, with an ID that can't be traced back to the account.
>
> [What's sent, and what isn't](https://kansoapp.app/metrics) · [The code](https://github.com/iamngoni/kanso/blob/master/docs/METRICS.md)
>
> **OK**

The notice has a version, so it shows again when what's sent changes. Desktop remembers it in
`nudges.metricsNoticeVersion` (it shows until the value is at least `2`; the older
`nudges.metricsNoticeSeen` alone no longer counts); the phones keep `mobile.metrics.notice_seen`
in the library's SQLite metadata, holding the version (`2`; the older value `1` shows it again).
Nothing is sent until OK. The phones show the notice only in a production release, start the first
send 20 seconds after the notice was already seen (or immediately after OK), and then send hourly
while the app is open.

## Reading the numbers

In PostHog, a Trends insight on `app_active` with **Unique users** gives DAU, WAU and MAU (set the
interval); break it down by `platform` or `form` for desktop against phones, by `app_version` or
`channel` for adoption, by `sync` for how many people sync, by `locale_lang`, `theme_mode` or
`editor_mode` for how people set Kanso up. `app_opened` per user shows how often people come back
in a day, and `session` broken down by `duration` how long they stay. `feature_used` broken down by
`feature` shows what is used, as events per feature (each event is one batch's use, with `count` a
range, so counts can't be summed). `server_stats` shows the sync server's load and errors.
