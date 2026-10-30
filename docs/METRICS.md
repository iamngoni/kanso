# Usage metrics

Kanso's apps send a small anonymous count once a day, so we know how many people use Kanso and
on which devices. There is no third-party SDK in the apps. This page is the one definition of
what may be sent: the shared client (`crates/kanso-metrics`) produces exactly this, and the sync
Worker (`cloud/src/metrics.ts`) refuses anything else before it passes the count on to PostHog.

What people read in the app is the note at [kansoapp.app/metrics](https://kansoapp.app/metrics)
(its text is [METRICS-NOTE.md](METRICS-NOTE.md)). It must change when this page does.

## The rules

1. **Never anything a person wrote.** No note text, titles, notebook or tag names, file names,
   usernames, email addresses, or anything typed. Every value is from a closed list or matches a
   pattern below. A value that doesn't fit is replaced with `unknown` on the device, and the
   Worker refuses the whole batch if one still arrives.
2. **No address, no account.** The Worker reads the country Cloudflare gives a request and never
   passes the IP address on. PostHog gets no account ID, username, device ID or session.
3. **Said before it starts.** Nothing is sent until the person has seen the notice (below) and
   pressed OK. Metrics can't be turned off; the notice and this page say so.
4. **Releases only.** Debug builds, Kanso Dev, the mobile development entry point and tests never
   send.

## Who is counted

Each batch carries one `id`:

| `id` | When | What it is |
|---|---|---|
| `a_` + 32 hex | the device is signed in to an account and holds its data key | the first 16 bytes of HKDF-SHA256(data key, no salt, info `"kanso/v1/metrics-id"`). Every device of the account computes the same value, so they count as one person. The sync server never holds the data key, so neither it nor PostHog can tell which account an ID belongs to. |
| `i_` + 32 hex | otherwise | 16 random bytes made the first time it is needed and kept on this device (desktop: `metrics.installId` in the config; phones: `metricsInstallId` in `preferences.json`). Never sent anywhere else. |

Signing in changes a device's ID from its `i_` to the account's `a_`; that day it can be counted
under both. Signing out goes back to the same `i_`.

## Events

### `app_active`

At most once per UTC day per `id`, when Kanso is open (at launch, and when the day changes while it
is open).

| Prop | Values |
|---|---|
| `platform` | `macos` \| `windows` \| `linux` \| `ios` \| `android` |
| `form` | `desktop` \| `phone` \| `tablet` (desktop platforms are always `desktop`) |
| `app_version` | `^[0-9]{1,4}(\.[0-9]{1,4}){2}$` |
| `os_version` | `^[0-9A-Za-z._-]{1,32}$`, else `unknown` |
| `sync` | `off` \| `kanso` (Kanso's own server: `sync.kansoapp.app`, `kanso.antonlabs.cc`) \| `self_hosted` |
| `ai` | `true` \| `false`: AI is turned on |
| `notes` | `0` \| `1-9` \| `10-99` \| `100-999` \| `1000+`: notes in the library, as a range only |

### `feature_used`

At most once per feature per UTC day per `id`, the first time that day the feature is used.

| Prop | Values |
|---|---|
| `platform`, `form`, `app_version` | as above |
| `feature` | `note_created` \| `voice_input` \| `publish` \| `import` \| `lock` \| `ai` \| `quick_capture` \| `shared_note` \| `export` |
| `source` | only with `import`: `markdown` \| `obsidian` \| `logseq` \| `notion` \| `evernote` \| `bear` \| `joplin` \| `standard_notes` \| `simplenote` \| `upnote` \| `apple_notes` \| `pouch` \| `other` |

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

No other event, prop or value exists. A new one is added here first.

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
        "app_version": "0.9.0",
        "os_version": "15.1",
        "sync": "kanso",
        "ai": false,
        "notes": "100-999"
      }
    },
    {
      "event": "feature_used",
      "day": "2026-10-30",
      "props": { "platform": "macos", "form": "desktop", "app_version": "0.9.0", "feature": "import", "source": "obsidian" }
    }
  ]
}
```

| Field | Pattern |
|---|---|
| `v` | `1` |
| `id` | `^(a\|i)_[0-9a-f]{32}$` |
| `events` | 1–32 events |
| `event` | `app_active` \| `feature_used` |
| `day` | `^[0-9]{4}-[0-9]{2}-[0-9]{2}$`, the device's UTC day, within two days of the Worker's |
| `props` | exactly the props listed for the event, every one present (`source` only and always with `import`) |

The whole body is at most 16 KiB. No other field is allowed, at any level.

## Sending

`POST https://sync.kansoapp.app/v1/metrics` with `content-type: application/json` and the batch
as the body, whichever sync server the device uses (a self-hosted server isn't sent counts). No
session, cookie or device ID is sent.

| Status | Meaning | The app |
|---|---|---|
| `204` | accepted | marks those events sent for their day |
| `400` | not a valid batch (the body names the field, never echoing a value) | drops the batch; it would fail again |
| `413` | over 16 KiB | as `400` |
| `429` | too many batches from this address | keeps them, tries again later the same day |
| other, or no answer | | keeps them, tries again later the same day |

Unsent events from an earlier day are dropped, not sent late.

## The Worker and PostHog

The Worker checks the batch against this page, adds `country` (the two-letter code from
`request.cf.country`, left out when there is none) and sends it to PostHog's batch API
(`POST ${POSTHOG_HOST}/batch/`) with:

- `distinct_id`: the batch's `id`
- `timestamp`: the event's `day` at 12:00 UTC
- `properties`: the props, `country`, `$process_person_profile: false` (no person profiles),
  `$geoip_disable: true`, `$lib: "kanso-worker"`

It answers `204` whether or not PostHog took it: a lost count isn't worth a retry. Without
`POSTHOG_KEY` it checks the batch and sends nothing. It stores nothing in D1; the sender is only
counted by the `limits` table (`metrics-ip`, a keyed hash of its IP group, 60 batches an hour).

Configuration: `POSTHOG_HOST` is a plain var (`https://eu.i.posthog.com`); the project's API key
is a secret, `wrangler secret put POSTHOG_KEY`. In the PostHog project, turn on **Discard client
IP data** and leave autocapture, session replay, surveys and heatmaps off (they need a browser SDK,
which Kanso doesn't load).

## The notice

The first time Kanso opens with this feature (a new install or an update), it shows once:

> **Kanso counts, it doesn't watch**
>
> To know how many people use Kanso and on which devices, Kanso sends a small anonymous count once a
> day: the kind of device, Kanso's version and which features were used. Never your notes, titles,
> tags, files, name or anything you type. Devices signed in to the same account count as one
> person, with an ID that can't be traced back to the account.
>
> [What's sent, and what isn't](https://kansoapp.app/metrics) · [The code](https://github.com/iamngoni/kanso/blob/master/docs/METRICS.md)
>
> **OK**

Desktop remembers it in `nudges.metricsNoticeSeen`, the phones in `metricsNoticeSeen` in
`preferences.json`. Nothing is sent until OK.

## Reading the numbers

In PostHog, a Trends insight on `app_active` with **Unique users** gives DAU, WAU and MAU (set the
interval); break it down by `platform` or `form` for desktop against phones, by `app_version` for
adoption, by `sync` for how many people sync. `feature_used` broken down by `feature` shows what
is used.
