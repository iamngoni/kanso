# Kanso for iPhone, iPad and Android

The Flutter app of [Kanso](../README.md), tracked in
[#48](https://github.com/iamngoni/kanso/issues/48). This folder holds the
design system (colours, type, icons, spacing, motion), the app shell (routes,
links, one to three panes, preferences, keyboard shortcuts), the screens for
browsing (the library, the note list, a note's text, search) and for settings
(theme, fonts, text size, sync status, About), the test and golden-image
tooling, CI, and the Rust core behind a bridge (`rust/`, `lib/core/bridge/`).
Writing and editing a note, and the rendered preview, come in later steps.

- Flutter 3.44.8 (stable), Dart 3.12, strict analysis.
- iOS and iPadOS 17 or later; Android 10 (API 29) or later; phones and tablets.
- The Rust toolchain (rustup; the versions are pinned in `rust/rust-toolchain.toml`)
  and, for Android, the NDK Flutter uses: the library is built by
  `hook/build.dart` during `flutter run`, `build` and `test`.
- iOS builds use Swift packages for plugins (no CocoaPods; `pubspec.yaml` turns
  it on for this project) and run on Apple silicon simulators only
  (`ios/Flutter/*.xcconfig` excludes x86_64).
- Bundle id `app.kanso.Kanso` on iOS, application id `app.kanso.kanso` on
  Android.

## Run

```sh
cd mobile
flutter pub get
dart run build_runner build --delete-conflicting-outputs
flutter gen-l10n
flutter run
```

`flutter run` starts the production entry point (`lib/main.dart` calls
`lib/main_production.dart`). `flutter run -t lib/main_development.dart`
starts the development one. There are no native flavours: one binary, two Dart
entry points.

Generated code (`*.freezed.dart`, `*.g.dart`, `injection.config.dart` and the
localisation classes in `lib/gen/l10n/`) is committed. CI regenerates it and
fails if the result differs, so run the two generators and commit after
changing a Freezed class, an injectable class or an ARB file.

## The Rust core

The notes, locked notes, sync and drawing are `crates/kanso-mobile`, the same
core the desktop app is built on. `rust/` wraps it for Dart with
[flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) (2.13.0,
pinned): `rust/src/api/` is what Dart sees, and the generated Dart is
`lib/core/bridge/` (committed). After changing `rust/src/api/`:

```sh
cargo install flutter_rust_bridge_codegen --version 2.13.0 --locked
flutter_rust_bridge_codegen generate   # from this folder
dart run build_runner build            # Freezed for the generated enums
```

`hook/build.dart` builds the library as a native asset, with the phones'
profile (`[profile.mobile]` in the root `Cargo.toml`) and the compilers of
`../scripts/mobile/env.sh`. `flutter test` runs it too; tests that call the
core open the library it built (`test/helpers/core_library.dart`). The app
opens its library at `kanso/library.db` in the application support folder
(`lib/core/configs/core_module.dart`); the background sync opens the same file.
`preferences.json` sits beside it (see below).

A note open for editing is a `KansoEditor` (`KansoLibrary.edit`): the text, its
undo history, the editor's commands, the colours of the text, completion and
saving are in Rust, and the field only hands over what changed. The calls it
makes on every keystroke are `#[frb(sync)]`, so they run on the UI isolate in
microseconds and a command and the field's text change in the same turn;
`lib/editor/models/repos/` wraps them (`EditorDocument`).

## Release

The release builds, signing, TestFlight and Play uploads, Shorebird patches and the store
text are in [docs/RELEASING-MOBILE.md](../docs/RELEASING-MOBILE.md) and `store/`. The version
of a release comes from the root `Cargo.toml`, not from `pubspec.yaml`'s `version:`, which is
only for local runs.

## Check

```sh
dart format --output=none --set-exit-if-changed lib test integration_test test_driver
flutter analyze
flutter test
```

## Integration tests and the scrolling check

`integration_test/` runs the app on a simulator or an emulator, and
`AppEnvironment.integration` (what `bootstrap` takes there) opens a library in
a new temporary folder, never the person's. `facade_test.dart` calls every
function of the bridge with the platform's own secret store; the three flows
use the app as a person does, on a library that `helpers/seed_library.dart`
fills through the app's repositories (two notebooks, notes with `#todo`, a
pinned note, a locked one):

| Flow | What it does |
| --- | --- |
| `browse_flow_test.dart` | All Notes under Pinned and Today; the library and a notebook's notes; a note opened and closed; a swipe pins; two notes chosen and moved; a note trashed and the Trash count; the locked note shows its title and its lock, and its text is nowhere |
| `search_flow_test.dart` | Type a word and see what matched highlighted; narrow to titles; open a hit |
| `new_note_flow_test.dart` | The Default notebook chosen in Settings; + makes a note in it, opened to write its title; a note made from a tag's list is tagged; ⌘N; a new notebook, and a note made in its own list |
| `open_flow_test.dart` | `kanso://open/note/…`, `…/notebook/…` and `…/search?q=…` as the platform hands them over; a note that is not there; a locked note |
| `icons_flow_test.dart` | Settings › Appearance › Icons lists the icon themes with Lucide in use; choosing Phosphor draws the app's icons from its SVGs at once and the Icons row names it; choosing Lucide again puts Kanso's own icons back |

```sh
flutter test integration_test/browse_flow_test.dart -d <simulator or emulator>
```

### On tablets

The flows are written for every layout class (see below): where the library is
a pane there is no drawer to open, where the note is beside its list there is
no page to go back from, and search is the field at the top of the library
rather than a button in the list's bar. `helpers/flow_driver.dart` has the
helpers (`inList`, `leaveNote`, `closeLibrary`, `openSearch`) that do the right
thing in each. `--dart-define=KANSO_ORIENTATION=portrait` or `landscape` turns
the device first (`SystemChrome.setPreferredOrientations`) and waits until the
window is that way round; without it the device keeps its own.

```sh
# An iPad simulator (boot it first: xcrun simctl boot <udid>)
xcrun simctl list devices available | grep iPad
flutter test integration_test/browse_flow_test.dart \
  -d <iPad simulator udid> --dart-define=KANSO_ORIENTATION=landscape

# A Pixel Tablet emulator (the AVD profile `pixel_tablet`, 2560x1600)
flutter test integration_test/browse_flow_test.dart \
  -d emulator-5556 --dart-define=KANSO_ORIENTATION=landscape
```

An iPad in portrait lays the app out in two panes (the list and the note),
in landscape in three; a Pixel Tablet is three panes in landscape (its natural
way round, 1280 x 800 dp) and two in portrait. On an Android emulator the flows
turn the screen themselves; set it first with
`adb shell settings put system accelerometer_rotation 0` and
`adb shell settings put system user_rotation 1` (portrait; 0 is landscape).
An iPad simulator cannot be turned from a script, and iPadOS ignores what an
app asks for while it runs in a window, so there the flow gives the window the
turned size (`tester.view.physicalSize`) and prints that it did: the panes
and the flow are those of a turned iPad, the screen is not. Turn the
simulator by hand (Device > Rotate Left) and the flow runs on the real turn.

CI runs them on an iPhone simulator and an Android 10 emulator, and again on an
iPad simulator and a Pixel Tablet emulator in both orientations. The Android job
then runs `scripts/mobile-backup-check.sh`, which asks the installed package
(`dumpsys package`) and fails if its flags list `ALLOW_BACKUP`.

`scroll_performance_test.dart` flings the note list of 2,000 notes in 8
notebooks up and down and records every frame's build and raster time
(`FrameTiming`) with the display's refresh rate. `lib/core/utils/frame_report.dart`
turns them into the numbers: the 50th, 90th and 99th percentile of each thread
and how many frames took longer than one refresh interval (8.33 ms at 120 Hz).
In debug mode, on a simulator or emulator at 60 Hz, it only checks that the
report was made. In **profile mode**, which is what a shipped app runs in, it
holds the numbers to a limit: p99 build and p99 raster under the interval, and
at most 1% of frames over it. Run it on the device, with `flutter drive`
(`test_driver/perf_driver.dart` prints the report and writes
`build/scroll_performance.json`):

```sh
flutter drive --profile \
  --driver=test_driver/perf_driver.dart \
  --target=integration_test/scroll_performance_test.dart \
  -d <device>
```

On a 120 Hz phone (ProMotion), add `--dart-define=KANSO_MIN_REFRESH_RATE=119`:
the run then also fails if the display ran slower, which would otherwise pass
at the looser 60 Hz interval. A simulator or emulator cannot measure 120 Hz;
the numbers are for a device. The iPhone's `Info.plist` has
`CADisableMinimumFrameDurationOnPhone`, which lets Flutter draw at 120 Hz.

### Typing into a long note

`editor_typing_performance_test.dart` types into a note of up to 50,000 words
the way a person does (and `editor_bench_app.dart` runs the same script as an
app, for a wireless iPhone). Long notes are edited a window at a time. How to
run it, the limits, and the results are in
[`docs/editor-gate.md`](docs/editor-gate.md); the manual device checks are in
[`docs/editor-device-checks.md`](docs/editor-device-checks.md).

## The app shell

**Routes** (`lib/core/router/`, go_router). `/` goes to `/notes`; the places
are `/notes`, `/pinned`, `/recent`, `/trash`, `/notebooks/:notebookId` and
`/tags/:tagId`; each of them and `/search?q=&scope=&in=` takes `/note/:noteId`
for a note opened from it. Ids are written with `Uri.encodeComponent` (note ids
contain `:`). `/settings` has `/settings/notebook`, `/settings/theme`,
`/settings/fonts?at=size` and `/settings/licences` inside it, and `/design` is the gallery of the design
proposal, in the development environment only (a row in its library). Places
and searches are `BrowseLocation` (`core/models/data/browse_location.dart`),
which writes and reads these addresses.

**Links.** `kanso://open/note/<id>`, `kanso://open/notebook/<id>` and
`kanso://open/search?q=…` open that (`core/router/deep_links.dart`; the scheme
is registered in `ios/Runner/Info.plist` and `AndroidManifest.xml`). The router
matches paths and ignores the scheme and host, so the redirect there turns a
link into an address of the app. Try one on a simulator:
`xcrun simctl openurl booted "kanso://open/notebook/<id>"`.

**Panes** (`core/configs/kanso_layout.dart`). One pane on a phone or a narrow
window, two when a tablet's window is 740 wide or more, three from 1020: the
note pane keeps at least 400. `BrowseShell` lays the panes out; in one the
library is a drawer (`KansoDrawer`: a button, a drag from the start edge on the
list, and the system's back close it). The window's size reaches the router
through `LayoutClassNotifier`, so the pages are built again for the new layout
and the open note stays open. Settings is a sheet on a tablet and a page on a
phone, whatever the width of the window.

**Preferences** live in `kanso/preferences.json` in the library folder, so they
stay on the phone and out of backups: the themes (`ThemeSelection`), fonts,
text size, list sort and notices seen. A save writes a temporary file and
renames it over the old one, one change at a time. A missing or unreadable file
is the defaults; a value the file has and this version does not know is that
value's default. `bootstrap` reads the saved themes before the first frame.

**Themes** come from the Rust core, which has the same theme files as the
desktop compiled in: `RendererRepository.builtinThemeTokens` gives the tokens
and `paletteFromTokens` turns them into a `KansoPalette` (the interface colours,
the syntax colours and styles, the preview's colours). `KansoPalette.kansoLight`
and `kansoDark` are the two Kanso themes written out, for the first frame and
for tests. `test/core/configs/theme_parity_test.dart` checks every theme's
palette against `crates/kanso-app/src/themes.golden.json`, which records what
the desktop draws.

**Backups.** The library is kept out of device backups on iOS by the core
(`kCFURLIsExcludedFromBackupKey`, set whenever it is opened) and on Android by
`android:allowBackup="false"` and two rule files in `res/xml/` that exclude
every domain from backups and from device transfers (Android 12 and later also
need `dataExtractionRules`). `test/platform/backup_exclusion_test.dart` reads
the manifest and both files.

## Settings, themes and the first launch

**Settings** (`lib/settings/`) has only rows that work: General (Default
notebook), Appearance (Theme, Fonts, Text size), Sync (the live status, "Off" while no account is signed in, and a
line that the notes are not in device backups) and About (the version, from
`package_info_plus`; the website, opened with `url_launcher`; Licences). Licences
is a page Kanso draws from Flutter's `LicenseRegistry`, never Flutter's own
Material one. Settings opens as a sheet on a tablet and a page on a phone, and
Fonts, Theme and Licences slide inside it. Everything the app does not do yet
(icon themes, locked notes, account, wake on change) is absent rather than
disabled.

**Default notebook** (`DefaultNotebookBloc`, `Preferences.defaultNotebookId`):
Settings › General › Default notebook lists "First notebook" and the notebooks
in the library's order, one checked; a locked notebook is dimmed and cannot be
chosen. It is where a new note goes unless it is made from a notebook's own
list; a notebook that is gone or locked reads as "First notebook".

**New notes** (`NoteCreationBloc`, shared by the buttons, ⌘N and a link in a
note to a note that is not there). Where a note goes is the Rust core's
(`kanso_engine::list::new_note_target`, which the desktop's New Note asks too):
the notebook of the list it is made from, else the Default notebook, else the
first. A tag's list tags the note and Pinned pins it. On a phone the button is
afloat over the list; where the list is beside the note it is in the list's bar
(a tablet without a keyboard has no ⌘N). The note opens in Edit with its title
to type; in a notebook that is locked and closed a line says to unlock it.
**New notebook** is the + of the library's Notebooks header and
secondary+shift+N: a sheet with a name (`NewNotebookBloc`) that makes a
notebook at the top and opens its list.

**Theme picker** (`lib/themes/`): a switch to follow the system's light and dark
mode, with the desktop's words, and the built-in themes as cards in a row, each
drawn from its own tokens (`themePreviewFromBuiltin`), the ones in use outlined
and first. Choosing one is `ThemeSelection.picked`, as on the desktop. The
gallery of themes to install needs the signed catalogue and a network, and is not
here.

**First launch.** A card says once that the notes on this phone are not in
device backups; OK is remembered in the preferences (`backupNoticeSeen`).

**Keyboard shortcuts** (`lib/shell/views/widgets/kanso_shortcuts.dart`, bindings in
`lib/shell/utils/kanso_shortcut_bindings.dart`). The secondary key is Command on
iOS and Control on Android, as the desktop's: secondary+`[` and `]` back and
forward through the notes opened, secondary+shift+F search all notes,
secondary+`,` settings, secondary+`\` show or hide the library (the pane of
three, or the drawer), secondary+option+down and up the next and previous note of
the list, secondary+shift+P pin the open note, secondary+`N` a new note in the place
being listed and secondary+shift+`N` a new notebook. Find in a note and the
command palette have none until what they act on is built.

## Crash reports

`lib/crash/` keeps a scrubbed crash report on the phone and sends it only when the
person taps Send. [docs/CRASH-REPORTS.md](../docs/CRASH-REPORTS.md) is the one
definition of a report; `lib/crash/utils/crash_grammar.dart` writes its patterns
once, and the scrubber (`crash_scrubber.dart` and one parser per source: Dart
stack traces, MetricKit JSON, Android tombstones (a small protobuf reader),
ANR traces and JVM exceptions) lets through only what matches. The native code
only drops raw files into `crash-reports/raw/` of the application support folder
(`ios/Runner/CrashDiagnostics.swift`, `android/.../KansoCrashes.kt`, also the
`app.kanso/crash` channel for the device model); the repository parses them,
keeps the reports in `crash-reports/pending/`, and deletes the raw files. After
a launch that finds reports a card asks; Settings › Privacy › Crash reports lists
them with the exact text.

## Design proposal

What is built is the real screens, with their goldens beside their tests
(`test/**/goldens/`). The mock screens (`lib/design/`) are those of the design
proposal that are not built yet, in a gallery for the owner to review, reached
from the library in the development environment: the note in edit, with the
keyboard's accessory bar and the completion of a link, and the signed-out card,
for a phone and for a tablet in landscape, with a light, dark or system switch.
The mock data is fixed (`lib/design/models/data/mock_data.dart`), and the images in
`test/design/views/pages/goldens/` are the proposal as pictures. The mocks of
the screens that have been built were removed with them; the approved images are
in the history.
The components the screens are made of (`lib/core/views/widgets/`) are the
ones the app uses.

## Windows and the viewport matrix

The panes the app shows come from the size of the window (`KansoLayout`):
one below 740 logical pixels wide or when the shortest side is under 600 (a
phone, even turned), two from 740, three from 1020. A tablet's window changes
with rotation, Split View, Slide Over and Stage Manager, so the tests use a
small, explicit set of windows, `KansoViewport` in
`test/helpers/kanso_viewport.dart`. Each is a size a real device reports, its
density and safe area, and the layout it must get; `kanso_viewport_test.dart`
checks that against `KansoLayout.classFor`, so the table cannot drift from the
code.

| Viewport | Size (logical px) | Density | Panes | Source |
| --- | --- | --- | --- | --- |
| `phone` | 393 x 852 | 3 | one | iPhone 17 (Apple HIG device sizes) |
| `iPadPortrait` | 820 x 1180 | 2 | two | iPad 10th gen, iPad Air 11-inch |
| `iPadLandscape` | 1180 x 820 | 2 | three | the same, turned |
| `iPad13Portrait` | 1032 x 1376 | 2 | three | iPad Pro 13-inch (M4, M5) |
| `androidTablet` | 1280 x 800 | 2 | three | Pixel Tablet, 2560 x 1600 px at xhdpi |
| `androidTabletPortrait` | 800 x 1280 | 2 | two | the same, turned |
| `splitViewTwoThirds` | 800 x 820 | 2 | two | iPad 11-inch landscape, two thirds |
| `splitViewHalf` | 590 x 820 | 2 | one | the same, half |
| `slideOver` | 375 x 820 | 2 | one | Slide Over, the width of an iPhone |

`test/shell/views/widgets/viewport_goldens_test.dart` draws the whole app (its
router, shell and panes, on the mock library) at `iPadPortrait`,
`iPadLandscape` and `androidTablet`, light and dark: the library as a drawer, a
list loading, empty, failed, choosing, its options and the move sheet and a
swipe; a note open, locked, not found, failed and loading; a search idle, with
results, none, failed, loading and finding a locked note; settings, fonts,
themes, licences and the backup notice. The same file draws the licences on a
phone, which had no picture. `viewport_resize_test.dart` walks a window through
the table (landscape, two thirds, half, Slide Over and back; a turn of an iPad)
and checks that the layout class follows, the open note, the list's scroll and
the search stay, the settings sheet fits every window, and nothing overflows.

## Golden images

Golden tests render a widget as iOS and as Android and compare both renders
with the same file, so any difference between the platforms fails. The helper
is `expectGoldenOnBothPlatforms` in `test/helpers/golden.dart`; the images sit
in a `goldens/` folder next to the test that wrote them. Text scale is 1 and
animations are off. The bundled fonts are loaded in `test/flutter_test_config.dart`.

The two renders are also compared with each other pixel by pixel, in memory,
with no tolerance. Against the stored file, anti-aliasing is ignored
(`test/helpers/kanso_golden_comparator.dart`): a pixel counts as changed only
when a channel moves by more than 32 of 255, and at most 0.3% of the pixels
may change. The images are generated on one Mac and compared on another, and
text rasterisation differs slightly between machines. Only one file is kept per screen.
Shadows are drawn blurred, as on a device (the test binding would otherwise
draw them as hard slabs).

Update the images after an intended visual change, look at them, and commit
them:

```sh
flutter test --update-goldens
```

Images are generated on macOS and compared on macOS in CI; do not update them
on another system.

## Conventions

Dart code follows the owner's Flutter rules (the `ngoni-flutter` skill):
BLoC only, one folder per BLoC under `state/<bloc_name>/`, Freezed for models,
events and states, GetIt with Injectable, one public widget per file,
relative imports, ARB localisations for all visible copy, and the lint
configuration in `analysis_options.yaml` (Very Good Analysis with strict
casts, inference and raw types).

Colours are not chosen here. `lib/core/configs/kanso_colors.dart` holds the
tokens of Kanso's own theme files, and `test/core/configs/kanso_colors_test.dart`
reads `../themes/kanso-dark.json` and `kanso-light.json` so the app and the
files cannot drift apart; every other theme is drawn from the core's tokens.

## Dependencies

Every dependency is MIT, BSD or Apache licensed.

| Package | Used for | Licence |
| --- | --- | --- |
| `bloc` | BLoC state machines | MIT |
| `go_router` | Routes, deep links and the back button | BSD-3-Clause |
| `bloc_concurrency` | `sequential` and `restartable` event transformers | MIT |
| `clock` | The time screens read ("Today", a row's hour), which tests fix | Apache-2.0 |
| `flutter_bloc` | Widgets for BLoCs | MIT |
| `freezed_annotation` | Annotations for generated models, events and states | MIT |
| `json_annotation` | Annotations for generated JSON conversion | BSD-3-Clause |
| `get_it` | Service locator | MIT |
| `injectable` | Registration annotations for GetIt | MIT |
| `flutter_svg` | Draws the Lucide icons | MIT |
| `intl` | Localisation (ARB) support | BSD-3-Clause |
| `flutter_localizations` | Flutter's own localisation delegates (Flutter SDK) | BSD-3-Clause |
| `flutter_rust_bridge` | Calls the Rust core (2.13.0, pinned) | MIT |
| `flutter_rust_bridge_hooks` | Builds the Rust core as a native asset (2.13.0, pinned); brings `native_toolchain_rust` (MIT) and `hooks` (BSD-3-Clause) | MIT |
| `code_assets` | The build hook reads the target platform | BSD-3-Clause |
| `path` | The build hook's file paths | BSD-3-Clause |
| `dartz` | `Either` for repository results | MIT |
| `path_provider` | The folder the library lives in | BSD-3-Clause |
| `package_info_plus` | The version and build in Settings › About | BSD-3-Clause |
| `shorebird_code_push` | Reads the number of the Shorebird patch the app runs (Settings › About and crash reports); it does not download patches | MIT |
| `share_plus` | Hands an attachment tapped in a preview to the system's share sheet | BSD-3-Clause |
| `url_launcher` | Opens the website from Settings › About | BSD-3-Clause |
| `webview_flutter` | Shows a note's preview page, with every tapped address handed to the Rust core | BSD-3-Clause |

The Rust library adds `core-foundation` and `core-foundation-sys` (Apple
targets only, MIT or Apache-2.0, already in the iOS tree through
`security-framework`) to mark the library's folder as excluded from backups.

Development only:

| Package | Used for | Licence |
| --- | --- | --- |
| `build_runner` | Runs the code generators | BSD-3-Clause |
| `freezed` | Generates models, events and states | MIT |
| `json_serializable` | Generates JSON conversion | BSD-3-Clause |
| `injectable_generator` | Generates the GetIt registrations | MIT |
| `very_good_analysis` | Lint rules | MIT |
| `xml` | Reads the Android backup rules in a test | MIT |
| `bloc_test` | BLoC tests | MIT |
| `mocktail` | Mocks | MIT |
| `glados` | Property-based tests | MIT |
| `integration_test` | Tests on a simulator or an emulator (Flutter SDK) | BSD-3-Clause |

Bundled files:

| Files | Used for | Licence |
| --- | --- | --- |
| Instrument Sans (`assets/fonts/InstrumentSans-*.ttf`) | Interface; the default note font | SIL Open Font Licence 1.1 (`assets/fonts/OFL-InstrumentSans.txt`) |
| Inter (`assets/fonts/Inter-Variable*.ttf`) | A note font (sans) | SIL Open Font Licence 1.1 (`assets/fonts/OFL-Inter.txt`) |
| Source Serif 4 (`assets/fonts/SourceSerif4-Variable*.ttf`) | A note font (serif) | SIL Open Font Licence 1.1 (`assets/fonts/OFL-SourceSerif4.txt`) |
| Literata (`assets/fonts/Literata-Variable*.ttf`) | A note font (serif) | SIL Open Font Licence 1.1 (`assets/fonts/OFL-Literata.txt`) |
| JetBrains Mono (`assets/fonts/JetBrainsMono-*.ttf`) | The default code font | SIL Open Font Licence 1.1 (`assets/fonts/OFL-JetBrainsMono.txt`) |
| IBM Plex Mono (`assets/fonts/IBMPlexMono-*.ttf`) | A code (and note) font | SIL Open Font Licence 1.1 (`assets/fonts/OFL-IBMPlexMono.txt`) |
| Lucide icons (`assets/icons/lucide/*.svg`) | Icons, the same set as the desktop app | ISC, with some icons MIT (`assets/icons/lucide/LICENSE`) |

Instrument Sans and JetBrains Mono are static instances (weights 400, 500,
600, 700 and italic 400 of Instrument Sans; 400 and 700 of JetBrains Mono) cut
from the variable fonts that Google Fonts publishes, at the default width.
Inter, Source Serif 4 and Literata are variable fonts, one file for the roman
and one for the italic, weight axis (`wght`) kept in full; the optical-size
axis was dropped with fonttools' instancer (it stays at the font's default)
to save about 2 MB. Flutter draws the weight a style asks for from the `wght`
axis, so styles use plain `FontWeight`s. IBM Plex Mono is published as static
files; 400, 500, 600, 700 and italic 400 are bundled. All are SIL Open Font
Licence 1.1, taken from `github.com/google/fonts` (`ofl/`); together the new
families add about 4.2 MB of font files.

Notes are set in the editor, preview and code fonts the person chooses
(`KansoFonts`), scaled by the text size (`KansoTextSize`: note text only, on
top of the device's own text size, which is never clamped); the interface
stays in Instrument Sans. A note font can also be "System", the device's own
font (San Francisco on iOS, the system font on Android): the one setting that
looks different per platform on purpose, so no golden uses it.

The licences are added to the registry the Licences page lists at start-up
(`lib/core/utils/license_registration.dart`).
