# Crash reports from the mobile apps

Kanso's iPhone, iPad and Android apps keep a crash report on the device after a crash and
send it only when the person taps **Send** (#48, step 13, provisional decision 1 of
2026-10-26). There is no third-party SDK. This page is the one definition of what a report
may hold: the app's scrubber (`mobile/lib/crash/`) produces exactly this, and the sync
Worker (`cloud/src/crash_reports.ts`) refuses anything else.

## The two rules

1. **A report never contains note text, titles or file names.** It holds where in Kanso's
   code the failure happened and nothing about what the person was doing. Everything is
   whitelisted: a value that doesn't match its pattern below is dropped (a frame) or
   replaced (`unknown`), never passed through or "cleaned".
2. **Nothing is sent without the person's choice.** The app shows the exact text of the
   report before it is sent.

## Sources

| `kind` | Where it comes from | What is read from it |
|---|---|---|
| `dart` | `FlutterError.onError`, `PlatformDispatcher.onError` | the error's runtime type name, the stack trace's frames |
| `ios-native` | MetricKit `MXCrashDiagnostic` (delivered at the next launch) | the exception type / signal (and, for a kill by the system, its termination code), the crashed thread's frames (binary name, offset, binary UUID) |
| `android-native` | `ApplicationExitInfo` with `REASON_CRASH_NATIVE`, its tombstone (protobuf) | the signal name, the crashing thread's backtrace (file basename, relative pc, function name, build ID) |
| `android-anr` | `ApplicationExitInfo` with `REASON_ANR`, its text trace | the main thread's frames |
| `android-jvm` | an uncaught-exception handler in Kotlin | the throwable's class name, its frames (class, method, line) |

Never read: messages (`toString()`, `exceptionReason`, `abort_message`), registers, memory
dumps, logs (`log_buffers`, logcat), open files, memory maps, command lines, paths beyond a
binary's base name, thread names.

## The report (JSON, version 1)

```json
{
  "v": 1,
  "kind": "dart",
  "type": "StateError",
  "app": { "version": "0.9.0", "build": "57", "patch": 2 },
  "os": { "name": "ios", "version": "18.1" },
  "device": "iPhone17,1",
  "day": "2026-10-26",
  "frames": [
    "dart _NoteEditorState._save package:kanso/note/views/widgets/note_editor.dart:120:7",
    "dart <closure> dart:async/zone.dart:1525:47"
  ]
}
```

| Field | Pattern | Notes |
|---|---|---|
| `v` | `1` | |
| `kind` | `dart` \| `ios-native` \| `android-native` \| `android-anr` \| `android-jvm` | |
| `type` | `^[A-Za-z_$][A-Za-z0-9_$.]{0,127}$`, else `unknown` | a type, signal or exception class name; generic arguments are cut at the first `<`. A MetricKit `SIGKILL` that carries a termination code in its `terminationReason` is `SIGKILL.<code>`, the code being `0x` and eight lower-case hex digits and nothing else of the text, for example `SIGKILL.0xdead10cc` (the app slept holding a file lock), `SIGKILL.0x8badf00d` (a watchdog timeout) or `SIGKILL.0xc00010ff` (thermal) |
| `app.version` | `^[0-9]{1,4}(\.[0-9]{1,4}){2}$` | |
| `app.build` | `^[0-9]{1,10}$` | |
| `app.patch` | integer 0–100000, or `null` | Shorebird's patch number; `null` without Shorebird or before the first patch. The key is always present: a report without it is a 400 |
| `os.name` | `ios` \| `android` | |
| `os.version` | `^[0-9A-Za-z._-]{1,32}$`, else `unknown` | |
| `device` | `^[A-Za-z0-9 ,._()-]{1,64}$`, else `unknown` | iOS's machine identifier (`iPhone17,1`), Android's `Build.MANUFACTURER` and `Build.MODEL` |
| `day` | `^[0-9]{4}-[0-9]{2}-[0-9]{2}$` | the UTC day of the crash; no time of day |
| `frames` | 1–128 strings, each one of the frame patterns below, at most 300 characters | outermost frames past 128 are dropped |

The whole body is at most 32 KiB. No other field is allowed, at any level.

### Frames

Each frame is one line, a keyword and space-separated parts:

| Frame | Pattern |
|---|---|
| Dart, with a symbol | `dart <symbol> <uri>:<line>[:<column>]` |
| Dart, without symbols (AOT with `--split-debug-info`) | `dart-pc +0x<hex>` (an offset into the isolate's instructions) |
| Native (iOS, Android) | `native <binary> +0x<hex>[ <symbol>+0x<hex>][ <build-id>]` |
| JVM | `jvm <class>.<method>:<line>` |

| Part | Pattern |
|---|---|
| `<symbol>` (Dart) | `^[A-Za-z_$<][A-Za-z0-9_$.<>]{0,199}$`, where `<anonymous closure>` is written `<closure>`; a symbol with anything else is replaced by `?` |
| `<uri>` | `^(package:[a-z0-9_]{1,64}/[A-Za-z0-9_/.-]{1,200}\.dart\|dart:[a-z0-9_/.-]{1,100})$`; a frame with any other URI (a `file://` path, an `org-dartlang-sdk:` one) keeps only `dart <symbol> ?` |
| `<line>`, `<column>` | `^[0-9]{1,7}$` |
| `<hex>` | `^[0-9a-f]{1,16}$` |
| `<binary>` | `^[A-Za-z0-9_.+-]{1,64}$`: a base name only (`libkanso_mobile.so`, `Flutter`, `Runner`); anything else is `?` |
| `<symbol>` (native) | `^[A-Za-z0-9_$.]{1,256}$` (mangled names as they are); anything else is left out |
| `<build-id>` | `^[0-9a-fA-F-]{8,64}$` (a Mach-O UUID or an ELF build ID) |
| `<class>` | `^[A-Za-z_$][A-Za-z0-9_$.]{0,199}$` |
| `<method>` | `^[A-Za-z_$<][A-Za-z0-9_$<>]{0,99}$` |
| JVM `<line>` | `^-?[0-9]{1,7}$` |

## Sending

`POST https://sync.kansoapp.app/v1/crash-reports` with `content-type: application/json` and
the report as the body. No session, cookie or device ID is sent. Answers:

| Status | Meaning | The app |
|---|---|---|
| `204` | stored | deletes the report |
| `400` | not a valid report (the body says which field, never echoing it) | keeps it, says it couldn't be sent |
| `413` | over 32 KiB | as `400` |
| `429` | too many reports from this address | keeps it, offers to try later |

The Worker checks the report against this page again, stores it with the day it was
received, and deletes reports after 90 days. It stores no IP address and no account.
Reading them: `npx wrangler d1 execute kanso-sync --remote --command "SELECT …"` from `cloud/`.
