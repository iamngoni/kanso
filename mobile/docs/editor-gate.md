# The editor's performance gate

The Markdown editor of the phone app is a Flutter `EditableText`. This is the
gate it has to pass before it ships, and what has been measured so far.

## What is measured

`integration_test/helpers/typing_script.dart` types into a note of N words
(`long_note.dart`: headings, lists, tasks, tables, fenced code, links,
wiki-links, a CJK paragraph, emoji, and one broken table three quarters down,
which sends the highlighter down its slow path). The script:

- 300 characters at the end of the note and 300 in the middle, at 10 per second
  and in bursts of 30 per second;
- Return at the end of a list, 20 times;
- one paste of 2 KB;
- 20 undos;
- two minutes of steady typing, for the memory measure.

Input goes through `EditableTextState.updateEditingValue`, the method the
platform channel calls. `lib/core/utils/typing_report.dart` turns the frames
and the timed spans into a `TypingReport`.

| Measure | Limit, in profile mode on a device |
| --- | --- |
| Build and raster time | p99 under one refresh interval, at most 1% of frames over it |
| Keystroke to frame | p95 at most 2 intervals, p99 at most 4 |
| Fresh highlight (a change to its new colours) | p95 at most 100 ms |
| `kanso.apply` (the core's call for a change) | p99 at most 1 ms |
| Memory growth over the two minutes | at most 50 MB |

## How it runs

```sh
# With a cable (or a phone that keeps its Dart VM service reachable):
flutter drive --profile \
  --driver=test_driver/typing_driver.dart \
  --target=integration_test/editor_typing_performance_test.dart \
  -d <device> [--dart-define=KANSO_TYPING_WORDS=20000] \
  [--dart-define=KANSO_WINDOW_UNITS=100000000]   # the whole note in the field

# Without the VM service (a wireless iPhone): the same script as an app.
flutter build ios --profile -t integration_test/editor_bench_app.dart \
  --dart-define=KANSO_TYPING_WORDS=50000
xcrun devicectl device install app --device <iPhone> build/ios/iphoneos/Runner.app
xcrun devicectl device process launch --device <iPhone> app.kanso.Kanso
xcrun devicectl device copy from --device <iPhone> \
  --domain-type appDataContainer --domain-identifier app.kanso.Kanso \
  --source Documents/editor_typing.json --destination editor_typing.json
```

The phone must stay unlocked and awake for the whole run (a few minutes):
when the screen locks, iOS suspends the app and the run stalls or its tail
percentiles are nonsense. In debug mode, or on a simulator or emulator, the
test runs a short script on a small note and only checks that the report was
made (CI does this).

`--dart-define=KANSO_WINDOW_UNITS=<n>` sets the size of the window (below);
without it the window is 8,192 UTF-16 units.

## Window mode

Flutter lays out and paints an `EditableText`'s whole text as one paragraph on
every change, and the text input channel sends the whole text with each update.
So a note longer than the window (8,192 UTF-16 units by default, about 1,400
words) is edited a window of whole lines at a time around the selection; the
text before and after is drawn as read-only slivers in the same scroll view.
The window moves when the selection comes within 1,024 units of an edge and no
composition is open. Select All makes the window the whole note (one slow
layout, for something deliberate). A note shorter than the window has exactly
one window, so ordinary notes behave as before. The Rust core never knows
about windows. Spell check is only given the window, so there is no longer a
character limit for it (the old limit of 200,000 characters is gone).

Costs, accepted by the owner: VoiceOver and TalkBack announce the field as the
window and read the rest as static text; a selection is limited to about two
windows except with Select All.

## Results

Fixture: `longNote(words)`; 5,000 words = 24,667 units, hash
`d72fb80f4399584d`. Flutter 3.44.8 (framework `058e0af2c2`, engine
`0cd610717b`), Dart 3.12.2.

### iPhone 17 Pro (the proxy for an iPhone 12-class phone), iOS 27.2, profile, 120 Hz, wireless

The budget is 8.33 ms. The run was wireless: `flutter drive` reached the app
only now and then (some attempts lost the VM service, one installed in ten
minutes), and the full script could not be completed under supervision. Two
runs gave numbers; neither is a clean gate run.

| Run | Mode | Frames | Build p50 / p99 (ms) | Raster p99 | Frames over | Keystroke to frame p50 / p95 / p99 (ms) | Fresh highlight p50 / p95 (ms) | apply p99 | Memory |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 5k words, quick script (167 keystrokes), `flutter drive` | whole note in the field | 542 | 0.29 / 39.4 | 2.5 | 22.1% | 34 / 114 / 177 | 338 / 1,657 | 0.42 ms | 215 to 219 MB |
| 5k words, full script (2,462 keystrokes), bench app | whole note in the field | 919 | 15.1 / 22.1 | 2.1 | 94.1% | 42 / 389 / (tail unusable) | 1,288 / (tail unusable) | 0.05 ms | 193 to 209 MB (+16.9) |

The second run was suspended by the screen locking for most of its length, so
its p99 keystroke latency and p95 fresh highlight (506 s, 538 s) are the
suspension, not the editor. Its frame build times are real: a median frame
that builds in 15 ms against a budget of 8.33 ms, with a note of only 5,000
words. Raster is not the problem (p99 about 2 ms); the time is in the build,
which is the paragraph being laid out again. `kanso.apply` is far inside its
limit in both runs.

The 20k and 50k whole-note runs, and every window-mode run on the iPhone,
were **not measured**: the phone's screen locked during each attempt.

### Android emulator (Pixel-class AVD, Android 16, 60 Hz, profile): functional numbers only

An emulator's timings come from the host's software GPU and say nothing about
a phone; there are no thresholds here. What they show is that the script runs
and that size matters.

| Words | Mode | Script | Frames | Build p50 (ms) | Keystroke to frame p50 (ms) | Note |
| --- | --- | --- | --- | --- | --- | --- |
| 5,000 | window 8,192 | quick | 136 | 7.4 | 1,141 | runs |
| 50,000 | window 8,192 | quick | 44 | 49.1 | 1,724 | runs; opening the note took five minutes |
| 50,000 | whole note | full | | | | stalled: frames of 9 s and 2,300 skipped frames; the driver lost the app |

### Mid-range Android

**Not measured: no device.** The gate stays open (owner decision: ship with
the Android gate recorded as open).

## Highlight latency (2026-10-26)

**These are host and simulator numbers, not the gate.** The gate is the
iPhone 17 Pro in profile mode, which has not been run since this change. The
host is the Mac (Apple silicon, release build of the Rust core); the simulator
is an iPhone 18 Pro in **debug** mode (profile is not available on a
simulator), so its Dart is far slower than a phone's and only the shape of the
change means anything.

The problem: on the phone, whole-note editing at 5,000 words had a fresh
highlight (a keystroke to its new colours) of p50 338 ms and p95 1,657 ms,
against a limit of p95 at most 100 ms.

### Where the time went

Two things, both in Kanso's code, neither in Flutter's layout. The first is
the large one.

1. **Colours for a text a few keys old were thrown away.** The controller
   asked Rust for colours once at a time, and showed them only if the text
   had not changed since. A call into Rust and back takes longer than a key at
   thirty a second (a burst), so during a burst no colours were shown at all
   until the typing paused: the delay grew with the length of the burst. Run
   on the simulator at 5,000 words (window mode, debug), the same script gave a
   fresh highlight p50 of 44 ms and p95 of **756 ms**, with the tail made of
   exactly those bursts. The same happened to the colours of the text outside
   the window, which were thrown away and asked for again after 300 ms for as
   long as there was typing.
2. **A syntax error made every request parse from the error to the end of the
   note.** `highlight::assemble` highlights a note block by block until a
   block has an error, then that block and all after it as one parse, which
   no longer matches anything in the cache once a key is typed after the
   error. For a note with the error three quarters of the way down (the
   fixture's broken table), a request for the 8,192-unit window at the caret
   took the whole parse of the rest: 23 ms at 50,000 words on the host
   (about 3 to 4 times that on a phone). The hashing of the blocks before the
   range, which the plan feared, is small: a request for a window before the
   error takes 1.4 ms at 50,000 words, and that is the cost of cutting and
   hashing the note.

The number of runs sent over the bridge is not the problem: a window request
sends 380 to 800 runs, against 2,400 to 24,000 for the whole note. The
controller already asked for the window's range only, one request at a time,
at most once a frame, so superseded requests were never queued.

### The fix

- **Dart.** `EditLog` keeps the changes made since colours were asked for; a
  batch for an older text is moved along them (`shiftRuns`, as the colours on
  screen already were) and shown, and keystrokes up to its version count as
  fresh. Only a window that moved or a text that came from outside (the log is
  reset) leaves a batch behind. Colours for the text outside the window are
  taken whatever the text in the window has done since. `shiftRuns` also had a
  bug that this exposed: a change over the edge of two runs gave the new text to
  both, leaving runs that overlapped. The new text is now in one run.
- **Rust.** The parse after an error stops at the end of the block that holds
  the end of the requested range and the one after it (the plan's cap), and,
  when the range is less than half of that parse, only what touches the range is
  captured and parsed again (inline text, fenced code); the block grammar still
  parses the text from the error to the range. A range that is most of its
  parse is highlighted whole and cached, as before.

The incremental-equals-full property is limited to the requested range, as
the plan said: the spans inside the range equal those of the parse to the end of
the note (`spans_uncapped`, property H6, 3,000 cases with errors of three
shapes); the spans of a whole-note request are unchanged (H4).

What remains is the grammar's parse from the error to the caret, which grows
with the distance between them (10.6 ms at 50,000 words with the error a
quarter of the note behind the caret). Reusing the old tree
(`Tree::edit`) would remove it; it is not built, because its result for broken
text is not guaranteed to equal a fresh parse and the host numbers are inside the
limit with room to spare. Look at it again if the phone says otherwise.

### Host (Rust, release), time of `styled_runs` after a keystroke, p50 in ms

`crates/kanso-mobile/tests/highlight_latency.rs`, run with
`cargo test -p kanso-mobile --release --test highlight_latency -- --ignored --nocapture`.
Generated notes with the Dart fixture's broken table three quarters of the
way down; 24 keystrokes at each place, the window is 8,192 units around the
caret. "Before" is commit 69118e2.

| Words | Request | Caret | Before | After |
| --- | --- | --- | --- | --- |
| 5,000 | window | end, after the error | 2.5 | 2.5 |
| 5,000 | window | 90%, after the error | 2.7 | 2.7 |
| 5,000 | window | 40%, before the error | 0.6 | 0.6 |
| 20,000 | window | end | 9.4 | 4.9 |
| 20,000 | window | 90% | 9.6 | 5.1 |
| 20,000 | window | 40% | 0.8 | 0.8 |
| 50,000 | window | end | 23.5 | 10.6 |
| 50,000 | window | 90% | 23.4 | 9.1 |
| 50,000 | window | 40% | 1.4 | 1.4 |
| 50,000 | whole note | end | 38.7 | 38.7 |
| 50,000 | whole note | 40% | 16.5 | 16.1 |

The first request of a session (cold) takes 5 to 41 ms for a window (5.2 ms at
5,000 words, 41 ms at 50,000 with the caret after the error) and 14 to 147 ms for the whole
note. The whole-note request is not used while typing in window mode.
At 5,000 words the host figure is 2.5 ms against the phone's 338 ms: the
phone's delay was not the Rust call, which is the finding above.

### Simulator (iPhone 18 Pro, iOS 27.0, debug, 60 Hz), quick script, window mode

`flutter test integration_test/editor_typing_performance_test.dart -d <simulator>
--dart-define=KANSO_TYPING_WORDS=<n>` (167 keystrokes). Indicative only.

| Words | Fresh highlight p50 / p95, before (ms) | Fresh highlight p50 / p95, after (ms) |
| --- | --- | --- |
| 5,000 | 44 / 756 | 33 / 54 |
| 50,000 | 71 / 920 | 46 / 68 |

Keystroke to frame is not changed by this (20 ms p50, 35 to 40 ms p95 in
debug mode in every run).

### What is still needed

The iPhone 17 Pro in profile mode at 5,000, 20,000 and 50,000 words, with the
full script, for the real fresh-highlight figure against the 100 ms limit. The
device gate is not passed by anything here.

## Memory and Flutter's undo history

`EditableText` keeps a copy of the whole text in an unbounded undo stack every
500 ms (Kanso's own history does the undo; Flutter's is never read). For a
note held whole this is about 600 KB every 500 ms at 50,000 words, about
70 MB over two minutes of typing. With window mode the field holds at most
about 8,000 units, so the same two minutes cost about 2 MB; that is the
mitigation. Swapping the controller at each autosave is therefore not built.
The iPhone's 5k-word run grew by 16.9 MB over the two minutes (a 25 KB note;
the suspension makes the figure unreliable), so the 50 MB limit is **not
confirmed** for 50k words in either mode.

## Findings that need a clean run

1. **Whole-note editing misses the budget at 5,000 words** on the proxy phone
   (median frame build 15 ms against 8.33 ms). At 50,000 words it cannot be
   better, and the emulator's whole-note run stalled. Window mode is built for
   this reason.
2. **Fresh highlight is slow at 5,000 words** (median 338 ms in the quick run
   on the phone, against a limit of 100 ms at p95). That is Kanso's code, or the
   core's, not Flutter's layout: the time between a change and its new colours
   includes the coalescing to one request per frame and the call into Rust.
   The escape-hatch table says to fix this within step 6. It was fixed there
   (see "Highlight latency"): the time was in Kanso's code, in colours that
   were thrown away while typing and in the parse after a syntax error. Not yet
   measured on the phone.

## Decision

Provisional. Window mode stays in step 6 (it was built, has unit, widget and
golden tests, and works on the emulator). **The gate is not measured on the
iPhone in window mode, so it is not passed.** Step 6b (a native text view) is
not triggered by these numbers: the only time seen on a phone is in the build
of one whole paragraph, which is what window mode removes, and the highlight
delay is in Kanso's own code. A clean run on the iPhone at 5k, 20k and 50k
words (window mode, then whole note for the record) decides it. What is needed
is a cable (`flutter drive` over USB), or the phone unlocked with Auto-Lock
set to Never for ten minutes per run.

## Summary for #48

Step 6, part 3: long notes and the gate.

- The editor types into a window of whole lines (8,192 units) around the
  caret for long notes, with the text before and after drawn read-only in the
  same scroll view; a note shorter than the window is one window. Spell check
  needs no size limit any more.
- The gate (`integration_test/editor_typing_performance_test.dart`, and the
  self-contained `editor_bench_app.dart` for a phone `flutter drive` cannot
  reach) is built and documented in `mobile/docs/editor-gate.md`.
- On the iPhone 17 Pro (the proxy, 120 Hz), whole-note editing already misses
  the build budget at 5,000 words (median frame 15 ms against 8.33 ms), and the
  fresh-highlight delay is too long (median 338 ms). The wireless phone's
  screen locked during every other attempt, so no clean full-script run exists
  yet, and window mode is **not measured on the phone**. The gate is therefore
  not passed. Needed: a USB cable, or an unlocked phone with Auto-Lock off.
- The Android emulator runs the script (functional only). The mid-range
  Android gate stays open (no device).
- Composition (Japanese, Chinese, Gboard-style), autocorrect, dictation, spell
  check and handwriting placeholders pass on an iPhone simulator and an Android
  emulator. VoiceOver on the iPhone needs the owner; Scribble was not run.
