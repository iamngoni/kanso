# Capture (#48 step 10)

How things shared into Kanso from other apps, the home-screen widgets and the
system's shortcuts reach the library, on iOS and Android.

## The library's folder

| Platform | Folder | Who opens it |
|---|---|---|
| iOS | `<App Group container>/Library/Application Support/kanso/` (group `group.app.kanso.Kanso`) | the app, the share extension, App Intents |
| Android | `<filesDir>/kanso/` | the app, the share activity (same process) |

The folder holds `library.db`, `preferences.json`, `inbox/` (below) and
`widget.json` (below). On iOS the app moves an older folder
(`<app container>/Library/Application Support/kanso/`) into the group
container once, on its first launch with this build, and keeps it excluded
from backups.

## Drops: what a share leaves for the library

A share is written as a *drop*: a folder in `<library folder>/inbox/`.

1. The writer makes `inbox/<id>.partial/`, where `<id>` is 32 lowercase hex
   characters from a random source.
2. It copies each shared file in, under a name of its own (`0.png`,
   `1.pdf`…), and writes `drop.json` last.
3. It renames the folder to `inbox/<id>/`. A folder still ending in
   `.partial` is never read, and a reader removes ones older than a day.

`drop.json` (UTF-8 JSON; every field but `version` may be missing):

```json
{
  "version": 1,
  "createdAt": 1793000000000,
  "subject": "The page's title, or the share's subject",
  "text": "Shared text",
  "url": "https://example.com/a",
  "files": [
    {"path": "0.png", "name": "IMG_0042.png", "type": "image/png"},
    {"path": "1.pdf", "name": "Quarterly report.pdf", "type": "application/pdf"}
  ]
}
```

- `path` is a file name inside the drop's folder; any other path (with `/`,
  `..` or absolute) is refused.
- `type` is what the sharing app said; the library decides by the file's
  extension and first bytes, not by this alone.
- At most 20 files per drop. Each file must fit the attachment limit
  (`MAX_ATTACHMENT_SIZE`, 10 MB).

## Voice drops

A recording started outside the app leaves a *voice drop*. It has the same
folder rules as any drop (`<id>.partial` while it is written, `drop.json`
last, then the rename), but `drop.json` says what it is:

```json
{"version": 1, "kind": "voice", "createdAt": 1700000013000,
 "started_ms": 1700000000000, "duration_ms": 12345, "audio": "rec.m4a"}
```

- `audio` names the recording in the folder: AAC in an `.m4a`, mono, about
  64 kbps. It is the only file. Times are milliseconds since 1970.
- A recording under about a second is thrown away, and one runs at most 30
  minutes.
- The folder and the audio are `NSFileProtectionCompleteUntilFirstUserAuthentication`,
  so a capture started on the lock screen is written without unlocking.
- Whatever writes it never opens the library, reads keys or calls the core.
  The app transcribes the drop (on the device) the next time it is open.
- Readers other than the app's voice pass (the share drain, `captureInbox`)
  leave a `kind: "voice"` drop alone: it has no text, URL or files for them.

On iOS:

- **Intent.** `CaptureVoiceIntent` (`openAppWhenRun = false`, an
  `AudioRecordingIntent` from iOS 18) starts recording at once; running it
  again, tapping Stop, or 30 minutes stops it. The "recording" state is kept in
  the App Group's `UserDefaults`, so a second run finds it.
- **Live Activity.** `VoiceCaptureAttributes` (in `Shared/`) is started with
  the recording: a red dot, the time and Stop (`StopVoiceCaptureIntent`) on the
  lock screen and in the Dynamic Island. On stop it shows "Saved to Inbox" for a
  few seconds and the system dismisses it. `UIBackgroundModes` has `audio`.
- **Control Center (iOS 18).** *Voice Note* runs the intent; *Quick Note* opens
  `kanso://open/new`.
- **Widgets.** The lock-screen circle is Voice Note. From iOS 18 the Voice note
  button runs the intent in place; on iOS 17 it opens `kanso://open/new?voice=1`
  (the intent would run in the widget extension, which can't hold the
  microphone).
- **Siri and Spotlight.** "Capture in Kanso", "Voice note in Kanso", "Record a
  note in Kanso".

On Android:

- **Service.** `VoiceCaptureService` is a foreground service of type
  `microphone`. It records with `MediaRecorder` (AAC in MPEG-4, mono, 64 kbps)
  into `<filesDir>/kanso/inbox/<id>.partial/rec.m4a` and, on stop, writes
  `drop.json` and renames the folder (`VoiceCapture.finish`). It stops on the
  notification's Stop, on a second start request (a toggle) and at 30 minutes.
  The ongoing notification reads "Kanso is recording · 00:14" and updates each
  second; tapping it opens the app. It never opens the library or calls
  `KansoNative.captureInbox`.
- **Quick Settings tile.** *Voice note* (`VoiceTileService`) is active while
  recording; a tap starts or stops the service, also from the lock screen.
- **Widget.** The widget's Voice note button starts the same service in place
  when the microphone permission is granted, and opens
  `kanso://open/new?voice=1` otherwise. New note and Today are links, as before.
- **Assistant.** The `voice_note` launcher shortcut carries the
  `actions.intent.CREATE_NOTE` capability ("Hey Google, voice note in Kanso"),
  which opens `kanso://open/new?voice=1`.
- **Refusals.** Android 14 only lets a visible action (tile or widget tap)
  start a microphone service. When Android refuses, or the permission is
  missing, the tile opens `kanso://open/new?voice=1` so the app's permission
  flow runs.
- **Transcribing a file.** `kanso_dictation`'s `transcribeFile` decodes the
  `.m4a` to 16 kHz mono PCM and gives it to `SpeechRecognizer`
  (`EXTRA_AUDIO_SOURCE`, Android 13 and later), on the device's own recognizer.
  A server-backed recognizer is used only when `allowServer` is true. Errors:
  `unsupported`, `notOnDevice`, `denied`, `unreadable`, `failed`; empty text
  with no error means nothing was heard.

## Saving drops

The Rust core saves each finished drop, oldest first, as one note in the
Inbox (`kanso_engine::capture`), then removes its folder:

- The note's ID is `note:cap-<id>`, and its attachments' `file:cap-<id>-<n>`,
  so a drop saved twice (a crash between the note and removing the folder)
  makes one note.
- Images and PDFs become attachments, linked in the note's body
  (`![](inkdrop://file:…)`, `[name.pdf](inkdrop://file:…)`). Anything else,
  or a file over the limit, is left out and named in the result's problems.
- A locked Inbox refuses the drop: its folder is removed and the result says
  so. Nothing is ever written into a locked notebook.
- A drop that fails for another reason (the database busy, a disk error) is
  kept and tried again the next time drops are saved.

### The C entry

```c
// Saves every finished drop in `inbox_path` into the library at
// `library_path` (its `library.db`). Returns a JSON string the caller frees
// with kanso_string_free, or NULL when the arguments aren't UTF-8 paths:
//   {"notes": ["note:cap-…"], "problems": ["…"], "kept": 0}
char *kanso_capture_inbox(const char *library_path, const char *inbox_path);
void kanso_string_free(char *text);
```

It blocks, may be called from any thread, opens no window and needs no
Flutter. The share extension, App Intents and the Android share activity
call it straight after writing a drop; the app calls the same code through
the bridge as it starts and whenever it comes back to the foreground.

- **iOS:** the library is `kanso_mobile_bridge.framework` in the app's
  `Frameworks/`. The share extension (in `PlugIns/`) opens it with `dlopen`
  from `../../Frameworks/` and finds the two functions with `dlsym`; the app
  itself finds them with `dlsym(RTLD_DEFAULT, …)`.
- **Android:** `KansoNative.captureInbox(libraryPath, inboxPath): String?`,
  a JNI method in `mobile/rust/src/android.rs`.

## Kanso's links for capture

| Link | Opens |
|---|---|
| `kanso://open/new` | a new note in Edit (the preferred notebook, as the + button) |
| `kanso://open/new?voice=1` | the same, for a voice note |
| `kanso://open/daily` | today's daily note, made if it isn't there |
| `kanso://open/note/<id>` | that note (already there) |

## `widget.json`

The app writes the widget's colors beside the library whenever the theme or
the appearance changes, so the widgets (SwiftUI, Glance) look like the app:

```json
{"version": 1,
 "light": {"background": "#FFFFFF", "surface": "#F4F4F5", "text": "#18181B", "muted": "#71717A", "accent": "#2563EB"},
 "dark":  {"background": "#0B0B0C", "surface": "#18181B", "text": "#FAFAFA", "muted": "#A1A1AA", "accent": "#60A5FA"}}
```

A widget with no file yet uses the colors of Kanso Light and Kanso Dark.
