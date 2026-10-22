# Mobile spike: gpui-pre-mobile or React Native

One question: can Kanso's phone app be built on
[`gpui-pre-mobile`](https://github.com/longbridge/gpui-mobile) (the longbridge fork,
commit `c24a130`, 14 Oct 2026) instead of React Native over the Rust core with
uniffi-bindgen-react-native?

The spike is [`mobile/kanso-mobile`](../mobile/kanso-mobile). It is its own Cargo workspace,
so the desktop workspace, CI and release workflows never build it. It runs a note list, then a
note in the gpui-base editor or the Markdown preview, over a `kanso-store` database in the app
container, with the GPUI Kit theme (light and dark).

Build and run (iOS Simulator, XcodeGen):

```sh
cd mobile/kanso-mobile
SIM=<simulator udid> ./build.sh [--release]
```

Tested on: iPhone 18 Pro simulator, iOS 27.0, Xcode 27 (macOS host). **Everything below is
simulator-only.** Nothing was run on a device.

## Verdict

**Not yet. Keep React Native as the plan of record; run one more, device-based spike before
deciding.**

The rendering, the store, selection and scrolling are better than expected, and the editor and
preview are the same widgets as the desktop app. But the decision hangs on text input, and
that is the weakest part:

- The fork turns autocorrect **off** in code, and the keyboard shows no predictions. I did not
  verify that this can be changed without patching the fork.
- IME marked text and dictation were not tested (the simulator can't drive them here).
- Accessibility is experimental per the fork's README, and I did not test it.

If a device test shows marked text and dictation work, and autocorrect can be turned on
(likely a one-line change in the fork's `configure_keyboard_traits`; I did not try it), the balance shifts towards
gpui-pre-mobile: one UI codebase, no bridge.

## What worked

| Area | Result |
|---|---|
| Dependencies | `gpui-pre =0.3.9`, gpui-kit `=0.7.3` (`tree-sitter-markdown` grammars only), `kanso-store` and `kanso-core` all compile for `aarch64-apple-ios-sim` with no patches. |
| Store | `kanso-store` (bundled SQLite) opens in the app container and seeds 5 notes in 2–18 ms. |
| Editor | `EditorState` / `Editor` render with Markdown highlighting and soft wrap, edit, and save back to the store on change. |
| Preview | `TextView::markdown` renders headings, lists, inline code, links and code blocks. |
| Theme | `Theme::change` light and dark both apply to the list, editor and preview. |

## Measurements

### Typing and IME

- Typing works. The fork uses a hidden `UITextView` as the composition buffer, so UIKit sends
  the text, and text typed with the keyboard reached the editor and the store.
- **The keyboard does not appear by itself.** Focusing the editor only gives GPUI focus; the
  app must call `gpui_mobile::show_keyboard()` on `InputEvent::Focus` (and `hide_keyboard()` on
  `Blur`). The spike does this.
- **Autocorrect is off.** `text_input.rs` calls `setAutocorrectionType: 1` (None) and
  `setAutocapitalizationType: 0`. Typing `teh recieve ` left it as typed, and the QuickType
  bar stayed empty.
- The mic (dictation) key shows on the keyboard. **Dictation not tested**: it can't be driven
  in the simulator.
- **Marked text not tested.** The fork overrides `setMarkedText` and its README says multistage
  IME composition is "not complete"; it needs a CJK keyboard on a device.
- Tapping empty space below the last line does not focus the editor; a tap has to land on
  text.

### Selection and caret with touch

- Tap places the caret.
- Long-press selects a word and shows the native iOS handles and the Cut / Copy / Paste /
  Select All menu (`04-selection.png`).
- Dragging a handle extends the selection, snapping to words (`05-selection-extended.png`).
- A long-press issued while a scroll fling was still moving did nothing; the same press on a
  still view worked.
- Not tested: magnifier loupe, double-tap, cut/paste round trips, selection across a scroll.

### Keyboard avoidance

- The fork publishes the keyboard height (`gpui_mobile::keyboard_height()`, 328 pt here); the
  spike pads the root by it, so the editor viewport ends at the keyboard.
- The height is a global, not an event, so the spike polls it every 60 ms to re-lay out.
  There is no animation and I expect a visible jump.
- Not verified: the caret scrolling into view when the keyboard hides it, and rotation.
- Safe-area insets come from `gpui_mobile::safe_area_insets()`.

### Scrolling feel

- Momentum scrolling works in the editor (`03-scroll.png`) and the preview, and a scroll bar
  shows. Done by swipe injection on the simulator; **I did not measure frame rate**, and a
  simulator is not representative of a phone's GPU.
- The editor scrolled a 30-section note without visible hitching.

### Startup and size

| | Debug | Release |
|---|---|---|
| App binary | 86 MB | 62.5 MB |
| `.app` directory | n/a | 61 MB |
| Zipped `.app` (not an `.ipa`: simulator build, unsigned) | n/a | 17.8 MB |
| App callback to window open | about 3.0 s cold, 0.3–0.5 s after | 286 ms first launch, 87–90 ms after |

Release profile: `opt-level = "z"`, fat LTO, `strip = true`, `panic = "abort"`. The `.app` is a
simulator build; a device `.ipa` was not built (no signing set up), so expect different sizes.
The startup numbers are from the app's own log line and exclude process launch time.

## What kanso-app code could be reused

- **As they are:** `kanso-core`, `kanso-store` (used unchanged here), and from the crate list
  `kanso-sync`, `kanso-collab`, `kanso-import` and `kanso-render` should compile for iOS.
  I only built `kanso-core` and `kanso-store`; the rest, including `kanso-render`'s QuickJS
  (MathJax), is untried.
- **Reusable only after extraction:** `kanso-app` is a binary crate (about 93k lines, no
  `lib.rs`). `editor.rs`, `preview.rs`, `theme.rs`, `themes.rs`, `note_list.rs`,
  `markdown_edit.rs`, `list_edit.rs` and `table_format.rs` are the pieces a phone wants, but
  they depend on desktop-only pieces (`global-hotkey`, AppKit, macOS PDF/OCR code) and on the
  app's global state. They would have to move into a shared library crate first.
  I did not do this; the spike builds its own small views from the same gpui-kit widgets.
- **Needs touch variants:** sidebar and window chrome (`title_bar`, `workspace`, `split`),
  command palette and menus (`palette`, `note_menu`), anything keyboard-shortcut driven (`keys`,
  `editor_vim`, `commands`), settings windows, and the context menus.
- Built-in themes come from the `themes/` submodule, which was not checked out here, so the
  spike used GPUI Kit's own light and dark themes, not Kanso's.

## What gpui-kit lacks, or what I could not confirm

From reading the crate sources and from the spike, not from building them:

- **Navigation:** no navigation stack or transitions; the spike swaps views by state.
- **Sheets:** `gpui-component` has a `sheet.rs`; I did not try it, so I can't say whether it
  does bottom sheets with detents.
- **Swipe actions on rows:** none found (only carousel and scroll code mentions swipe).
- **Long-press:** no long-press handler found in the component library; the selection
  long-press above is done by UIKit's text view, not by gpui-kit.
- **Pull to refresh, haptics, native share and document pickers:** not provided by gpui-kit.
  The fork has packages for some (share, haptics-like vibration, file selector), untested here.
- **Safe area, keyboard height and the keyboard itself** come from the fork, not the kit, and
  need app glue (see above).

## Fork and platform problems found

1. The fork's example host crashes on launch with the iOS 27 SDK ("UIScene life cycle is
   required"). I worked around it in `ios/main.m` with a scene delegate and a swizzle that gives
   the fork's `UIWindow` a scene. This needs fixing upstream.
2. Autocorrect is hardcoded off (see above).
3. The status bar text stays dark in the dark theme (the host sets no status bar style).
4. `gpui-pre-mobile` 0.1.0 on crates.io targets gpui-pre 0.3.4; use the git revision for
   0.3.9.
5. Linker warnings: libraries built for iOS 27.0 linked against a 16.0 target unless
   `IPHONEOS_DEPLOYMENT_TARGET` is set (`build.sh` sets it).
6. Debug builds are slow on first start (about 3 s to create the wgpu renderer).

## Recommendation

| | gpui-pre-mobile | React Native + uniffi-bindgen-react-native |
|---|---|---|
| UI code | One: the desktop widgets (editor, preview) | A second UI to write and keep in step |
| Rust core | Linked directly | Through a generated bridge |
| Text editing | Works; autocorrect off, IME and dictation unverified | Native `TextInput`; known quality, but no shared Markdown editor |
| Selection, keyboard, a11y | Selection good; a11y experimental; keyboard glue by hand | Native |
| Maturity | One fork, experimental per its README | Mainstream |
| Size | 17.8 MB zipped simulator build | Not measured |

**Recommendation:** keep React Native for the phone app for now. Before deciding, spend a
second short step on a physical iPhone to check three things: marked text with a CJK keyboard,
dictation, and autocorrect with that change. If all three pass, gpui-pre-mobile
becomes the better choice, because the same editor and preview would run on all platforms.

Screenshots: [list](mobile-spike/01-list.png), [editor](mobile-spike/02-editor.png),
[scrolled editor](mobile-spike/03-scroll.png), [selection](mobile-spike/04-selection.png),
[extended selection](mobile-spike/05-selection-extended.png),
[preview](mobile-spike/06-preview.png), [dark preview](mobile-spike/07-dark-preview.png),
[release build](mobile-spike/08-release-list.png).
