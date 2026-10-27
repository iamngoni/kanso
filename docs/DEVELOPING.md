# Developing Kanso

## Kanso and Kanso Dev

`/Applications/Kanso.app` only ever comes from a published release, and
updates itself from there ([RELEASING.md](RELEASING.md)). To try a change in
a real app without touching it, build **Kanso Dev**:

```sh
scripts/bundle-macos.sh --dev             # target/kanso-dev/release/Kanso Dev.app
scripts/bundle-macos.sh --dev --install   # and copy it to ~/Applications/Kanso Dev.app
```

The flavour is chosen when Kanso is compiled: the `dev-flavour` feature of
`kanso-app` (`cargo run -p kanso-app --features dev-flavour` works too).
`crates/kanso-app/src/flavour.rs` holds everything that differs:

| | Kanso | Kanso Dev |
|---|---|---|
| Name | Kanso | Kanso Dev, with a DEV badge in the sidebar and an amber DEV band on its icon |
| Bundle ID, login item | `app.kanso.Kanso` | `app.kanso.Kanso.dev` |
| Data folder | `~/Library/Application Support/Kanso`, `%APPDATA%\Kanso`, `~/.local/share/kanso` | `…/Kanso Dev`, `%APPDATA%\Kanso Dev`, `~/.local/share/kanso-dev` |
| Keychain service | `Kanso` | `Kanso Dev` |
| Local API port (new config) | 19840 | 19841 |
| Updates | from releases | off: Settings › Updates says "Development build — updates are off" |

So Kanso Dev never opens or writes the release's library, config or account,
and never replaces itself or `/Applications/Kanso.app`. `--install` is only
for `--dev`, and installs to `~/Applications`; the release workflow builds
the normal flavour. `KANSO_DATA_DIR` or `--data-dir` still point either
flavour at another folder.

A plain `cargo run -p kanso-app` is the normal flavour, but a debug build: it
checks for updates and shows them, and never installs one (it offers the
release page).

## Tests

```sh
PROPTEST_RNG_SEED=20261017 cargo test --workspace
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

New logic gets property tests alongside example tests; the seed is pinned so
a failure in CI reproduces here. The views are checked as PNGs without
capturing the screen:

```sh
KANSO_SNAPSHOT_DIR=/tmp/shots cargo test -q -p kanso-app snapshots -- --ignored --nocapture
KANSO_SNAPSHOT_ONLY=dark-update-ready,light-settings-updates ...   # a few
```

Voice input's real transcription test needs the Parakeet model (about 670 MB)
in a folder of its own; it is skipped otherwise:

```sh
KANSO_VOICE_MODEL_DIR=~/Library/Application\ Support/Kanso\ Dev/models/parakeet-tdt-0.6b-v3 \
    cargo test --release -p kanso-voice --test model -- --ignored --nocapture
```

ONNX Runtime is linked into the app statically; its build script downloads
pyke's prebuilt archive once and checks it. On macOS the bundle carries the
microphone entitlement (`packaging/macos/Kanso.entitlements`), without which
the hardened runtime keeps the microphone from Kanso and macOS never asks.

## Mobile targets

The core the mobile app shares (`kanso-core`, `kanso-store`, `kanso-engine`,
`kanso-render`, `kanso-collab`, `kanso-import`, `kanso-sync` and `kanso-ai`,
the last two without their desktop features, plus `kanso-tls`), the facade the
app calls (`kanso-mobile`) and the library the app loads
(`kanso-mobile-bridge`, in `mobile/rust`) build for
`aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`,
`armv7-linux-androideabi` and `x86_64-linux-android`. Add the targets once
with `rustup target add`; Android needs the NDK (`ANDROID_NDK_HOME`, else the
newest in `~/Library/Android/sdk/ndk`), which `scripts/mobile/env.sh` points
the C compilers and bindgen at.

```sh
scripts/mobile-check.sh                  # cargo check for the five targets, the banned-dependency look, clippy
scripts/mobile-test.sh ios-sim           # the tests, in a booted iPhone simulator
scripts/mobile-test.sh android           # the tests, on the one device or emulator adb lists
scripts/mobile-size.sh ios|android       # what the release app and its library weigh (builds it; --no-build measures the last build)
scripts/mobile-artifact-check.sh ios|android <app>  # a built app has the library, its exported symbols, and (Android) the classes R8 must keep
scripts/mobile-parity.sh                 # the mobile profile in hook/build.dart equals [profile.mobile]; both toolchain files name one channel
scripts/mobile-background-check.sh       # the background sync entry, called from a plain C process
```

- `mobile-test.sh` runs each crate's library and integration tests (not
  doctests) through a Cargo runner: `KANSO_IOS_SIM_UDID` picks a simulator,
  `ANDROID_SERIAL` a device, and `KANSO_TEST_CRATES="kanso-sync kanso-ai"`
  narrows the run. On Android the test binaries and the fixtures they read are
  copied to `/data/local/tmp/kanso`, so fixture paths are read from
  `CARGO_MANIFEST_DIR` at run time, not when the test is built. An Apple-silicon
  Mac runs `aarch64-linux-android` in an arm64 emulator; CI runs an x86-64 one
  at API 29 (`.github/workflows/mobile-core.yml`, only for changes that can
  affect it).
- `cargo build --profile mobile` is the phones' release build: `opt-level =
  "z"`, fat LTO, one codegen unit, and `panic = "unwind"`, so the facade
  catches a panic and returns an error. The desktop's `[profile.release]` is
  unchanged. The Flutter app's hook builds the library with the same settings
  (`mobile/hook/build.dart`, as `CARGO_PROFILE_RELEASE_*`), and
  `scripts/mobile-parity.sh` fails when the two differ.
- What the app weighs (`scripts/mobile-size.sh`, release build, last measured
  with Flutter 3.44.8; MB are 10^6 bytes). The library is `kanso_mobile_bridge`
  as the app embeds it, built with `[profile.mobile]` (symbols already
  stripped, so `strip` saves nothing more); Zip is the library alone under
  `zip -9`:

  | Platform | Library (raw) | Library (zip) | App package | Under gzip again |
  |---|---:|---:|---:|---:|
  | iOS device, arm64 (`.ipa` equivalent) | 42.24 MB | 8.24 MB | 17.08 MB | not measured |
  | Android arm64-v8a (APK) | 44.75 MB | 8.89 MB | 64.07 MB | 18.71 MB |

  The library is half the iOS download and, uncompressed, more than two thirds
  of the Android APK (which stores libraries uncompressed so Android can map
  them; the Play Store compresses its transfers). The rest of the iOS app is
  Flutter.framework 4.44 MB and App.framework 4.26 MB zipped; of the APK,
  libflutter.so 11.58 MB, libapp.so 4.65 MB and assets 4.98 MB. The script
  fails when the library zipped is over 12 MB (`KANSO_MAX_LIB_ZIP`): the bare
  library of #93 was 8.66 MB (iOS) and 9.56 MB (Android) zipped, so more means
  the profile stopped applying. CI (`.github/workflows/mobile.yml`) prints the
  tables in the job summary of `build-ios` and `build-android`, for every
  Android ABI. The earlier measurement put about three quarters of the bare library in the tree-sitter grammars; dropping some
  on phones is a separate decision.
- `kanso-mobile` is the facade: plain Rust with no bridge generator, tested on
  a computer and on the simulators. `mobile/rust` (`kanso-mobile-bridge`) is
  the library the app loads: it wraps the facade for flutter_rust_bridge and
  defines the C entry the background jobs call, `kanso_background_sync`
  (`src/background.rs`), and on Android the native method that hands over the
  app's `Context` (`src/android.rs`). `mobile-background-check.sh` builds the
  library for this computer and calls the entry from a C program
  (`mobile/rust/tests/c_abi/main.c`), as the system's job calls it, with no
  Dart in the process.
- A note open for editing is a `kanso_mobile::EditorSession`
  (`Library::edit`, `crates/kanso-mobile/src/editing.rs`). The text, its undo
  history (`kanso_engine::edit::history`), the commands (Enter in a list, Tab,
  formats), the colours, completion and saving are Rust's; the app's text field
  draws the text and hands over each change. Typing is saved 600 ms after the
  last change (`kanso_engine::saving`, which the desktop editor follows too) by
  the library's `kanso-editors` thread, and a note changed elsewhere meanwhile
  asks which version to keep, as on the desktop. A library made by
  `open_for_test` has no autosave thread: a test saves with
  `EditorSession::tick(now)`. When the lock closes, sessions on locked notes
  are saved sealed, then wiped. `tests/editor.rs` and `tests/locked.rs` hold
  the properties.
- Secrets (the session, the data key) are kept by Rust in the platform's
  store: the Keychain on iOS, the Keystore on Android through a Kotlin class
  (`kanso_mobile::secrets::android` lists its three methods), memory on a
  computer. The tests never touch your own Keychain.
- The Rust library has its own `mobile/rust/rust-toolchain.toml`, which must
  name the same channel as the root's (the native-assets build runs cargo in
  that folder).
- rquickjs ships no bindings for phones, so `kanso-render` builds them with
  libclang (Xcode's, or the NDK's) on iOS and Android. A bindgen setting for
  your own Mac in `BINDGEN_EXTRA_CLANG_ARGS` would be passed to the phones'
  targets too; the scripts unset it.
- Phones check servers' certificates with the system (`kanso-tls`). On
  Android that needs the app's `Context` first; until the app hands it over,
  requests over HTTPS fail with an error. A bare `adb shell` test binary has no
  JVM, so the real certificate check on Android is tested in the app.
