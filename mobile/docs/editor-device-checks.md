# The editor's device checks

What was run against the editor, where, and what is left for a person. The
automated rows run in CI (`integration_test/editor_ime_test.dart` on an iPhone
simulator and an Android emulator; the semantics tests in `flutter test`).

Date of every row: 2026-10-26. Who: the automated run (Claude), unless it says
otherwise. Simulator: iPhone 18 Pro, iOS 27.0. Emulator: Android 16 (API 36).

| Check | Device and OS | Steps | Expected | Result |
| --- | --- | --- | --- | --- |
| Japanese: romaji, kana, kanji | iOS simulator; Android emulator | `editor_ime_test`: a composing region grows and is converted | The text is never replaced during a composition; Rust's text matches after each step | Passed on both |
| Chinese after an emoji | iOS simulator; Android emulator | Pinyin to hanzi after a surrogate pair | Offsets are UTF-16 units | Passed on both |
| Latin words composed as Gboard does | iOS simulator; Android emulator | Each word composed, then committed | Same | Passed on both |
| A composition does not hold the save back | iOS simulator; Android emulator | Compose, wait past autosave | The text is saved | Passed on both |
| Return commits the composition, then continues the list | iOS simulator; Android emulator | Compose in a list item, press Return | One new item | Passed on both |
| Autocorrect is one undo step | iOS simulator; Android emulator | Replace a word range | One undo restores it | Passed on both |
| Dictation: a placeholder, then a phrase | iOS simulator; Android emulator | `insertTextPlaceholder`, a multi-word insert | The text is the phrase | Passed on both |
| Handwriting placeholder shows no errors | iOS simulator; Android emulator | Placeholder active | No misspelling marks | Passed on both |
| Spell check marks, keeps colours, skips code and addresses | iOS simulator; Android emulator | Inject results | Underlines on words only | Passed on both |
| Semantics: multiline field, key labels, Redo action, live regions | `flutter test` | `editor_semantics_test.dart` | As planned | Passed |
| Typing and selection: handles, magnifier, menu, floating cursor | iPhone 17 Pro | By hand | Handles and menu where the caret is | Not run: needs a person with the phone |
| Real Japanese and Chinese keyboards | iPhone 17 Pro | By hand | As the automated row | Not run: needs a person |
| Dictation on the phone | iPhone 17 Pro | By hand | The spoken text is inserted | Not run: needs a person |
| Autocorrect and spell suggestions on the phone | iPhone 17 Pro | By hand | Suggestions replace the word; one undo | Not run: needs a person |
| Scribble | iPad with Apple Pencil | By hand | Handwriting becomes text | Not run: no iPad with a Pencil here (owner decision) |
| VoiceOver | iPhone 17 Pro | Swipe through a note, edit a word | Field announced as multiline text with its value; the window's edge is readable | **For the owner**: needs a human |
| TalkBack | Android emulator with Accessibility Suite | By hand | As VoiceOver | Not run: needs a person (and the Accessibility Suite on the emulator) |
| Hardware keyboard | iPad | By hand | Shortcuts of the plan | Not run: needs a person |
| System undo gestures | iPhone 17 Pro | Shake, three-finger swipe | Rust's undo and redo | Not run: needs a person |
| External change while typing | Simulator | `conflict_flow_test.dart` | Keep mine and Use theirs | Passed (part 2) |
| Lock while typing | Simulator; emulator | `locked_edit_flow_test.dart` | Text gone, stored body sealed | Passed (part 2) |
| 120 Hz typing | iPhone 17 Pro | `editor-gate.md` | Gate numbers | Not measured cleanly: see `editor-gate.md` |
