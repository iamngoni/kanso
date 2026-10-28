# Store listings

The words and pictures for the App Store and Google Play, written once here so they
are reviewed like code. Nothing in this folder is uploaded by CI except
`play/whatsnew/whatsnew-en-US`, which goes to Play with each internal release.
The rest is pasted or uploaded by the owner into the consoles
([docs/RELEASING-MOBILE.md](../../docs/RELEASING-MOBILE.md), "What the owner does").
`scripts/mobile-store-check.sh` (run by the Mobile release workflow) holds every text
to its store's limit and keeps other notes apps out of them.

All text is English (US). Kanso's voice: plain, calm, no hype.

## Files

| File | Where it goes | Limit |
|---|---|---|
| `app-store/name.txt` | App Store Connect › App Information › Name | 30 |
| `app-store/subtitle.txt` | Subtitle | 30 |
| `app-store/promotional-text.txt` | Version › Promotional Text (can change without a review) | 170 |
| `app-store/description.txt` | Description | 4000 |
| `app-store/keywords.txt` | Keywords, comma-separated, no spaces | 100 |
| `app-store/whats-new.txt` | Version › What's New (0.9.0) | 4000 |
| `app-store/support-url.txt`, `marketing-url.txt`, `privacy-policy-url.txt` | URLs | |
| `app-store/primary-category.txt`, `secondary-category.txt`, `copyright.txt` | | |
| `app-store/privacy.md` | App Privacy answers (draft), with the open points | |
| `app-store/review-notes.md` | App Review Information › Notes | |
| `play/title.txt` | Main store listing › App name | 30 |
| `play/short-description.txt` | Short description | 80 |
| `play/full-description.txt` | Full description | 4000 |
| `play/whatsnew/whatsnew-en-US` | Release notes (sent by CI) | 500 |
| `play/category.txt`, `support-url.txt`, `website-url.txt`, `privacy-policy-url.txt` | | |
| `play/data-safety.md` | Data safety form (draft), with the open points | |

The support URL is the project's GitHub issues page: it works, but asks people for a
GitHub account. A page on kansoapp.app (with the account deletion link both stores want)
is better; change the files when it exists.

## What each claim depends on

The descriptions are written for the app as designed in
[#48](https://github.com/iamngoni/kanso/issues/48). Before submitting, strike anything that
has not shipped.

| Claim | Build step |
|---|---|
| Notebooks, tags, pinned, recent, library; search with highlights; themes; three panes on iPad | 5 (merged) |
| Markdown editor, styling as you type, the bar above the keyboard, dictation, autocorrect | 6 |
| Preview, math and diagrams, wiki-links followed | 7 |
| End-to-end encrypted sync, sign-in methods, device approval, recovery key | 8 |
| Locked notes, Face ID / fingerprint, the cover in the app switcher | 9 |
| Share sheet, widgets | implemented; device QA pending |
| Ask your notes | implemented; keep out of public copy until live-model QA |
| Not claimed (drop-in later): canvases and ink (11), recordings (12) | |
| Kept out of iCloud and device backups | 5 (merged) |
| No ads, trackers or analytics | true while crash reporting stays undecided |

If Ask your notes ships in 0.9.0 after live-model QA, add it to the descriptions and to the
privacy answers.

## Screenshots

Apple needs screenshots for the largest iPhone (6.9-inch, or 6.5-inch if there is no 6.9
set) and, because Kanso runs on iPad, for the 13-inch iPad; smaller sizes are scaled from
them. Play takes 2 to 8 per device type, JPEG or 24-bit PNG without alpha, each side between
320 and 3,840 px, the long side at most twice the short one; a phone set is needed, and 7-inch
and 10-inch tablet sets (1,080 px or more on the short side) for tablet featuring. Play also
requires a 1024 × 500 feature graphic and a 512 × 512 icon.
Sources: [App Store Connect screenshot specifications](https://developer.apple.com/help/app-store-connect/reference/screenshot-specifications),
[Play Console: add preview assets](https://support.google.com/googleplay/android-developer/answer/9866151).

The rendered pictures are the default for the listings. They are drawn from the golden-test
harness (`test/store/store_screenshots_test.dart`): the real app over the mock library, in
the real fonts, at each set's exact size, as iOS for the App Store sets and as Android for
the Play sets, with no simulator, so CI can make them. The status bar and home indicator
are left as blank safe area (the pictures carry no device frame or clock).

| Set | Pixels (portrait) | Layout | Used for |
|---|---|---|---|
| `ios-iphone-6.9` | 1320 × 2868 | one pane | App Store, 6.9-inch iPhone |
| `ios-iphone-6.5` | 1284 × 2778 | one pane | App Store, 6.5-inch iPhone |
| `ios-ipad-13` | 2064 × 2752 | three panes | App Store, 13-inch iPad |
| `android-phone` | 1080 × 2160 | one pane | Play phone (2:1, the tallest Play accepts) |
| `android-tablet-7` | 1200 × 1920 | one pane (tablet display) | Play 7-inch tablet |
| `android-tablet-10` | 1600 × 2560 | two panes | Play 10-inch tablet |

Each set holds six pictures: **01-notes** (the list, with the pinned note), **02-note** (an
open note), **03-library** (notebooks and tags, a drawer on one and two panes, a pane on
three), **04-search** (a highlighted match), then the same list and note in dark
(**05-notes-dark**, **06-note-dark**). Add the editor with the keyboard bar, a locked note
and the iPad in three panes as steps 6 and 9 land. The pictures show the mock library, not
real notes. Play's extras are written to `play/`: `feature-graphic.png` (1024 × 500) and
`icon-512.png` (512 × 512, the App Store icon scaled down). The 1024 × 1024 icon for the App
Store is `ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-1024x1024@1x.png` (opaque).

```sh
scripts/mobile-store-render.sh            # writes mobile/store/screenshots/<set>/NN-name.png
scripts/mobile-store-render.sh some/dir   # or somewhere else
```

The script runs the tagged test (`flutter test --tags store-screenshots`, which does nothing
unless `KANSO_STORE_SHOTS_DIR` is set) and then checks every PNG's size against its set and
that none has an alpha channel. Look at each picture before it goes to a store.

For pictures on real device frames and status bars, the simulator script remains:

| Set | Device to use | Pixels |
|---|---|---|
| `ios-iphone-6.9` | iPhone 17 Pro Max simulator | 1320 × 2868 |
| `ios-ipad-13` | iPad Pro 13-inch (M5) simulator | 2064 × 2752 |

```sh
scripts/mobile-store-screenshots.sh ios-iphone-6.9 "iPhone 17 Pro Max"
scripts/mobile-store-screenshots.sh ios-ipad-13 "iPad Pro 13-inch (M5)"
scripts/mobile-store-screenshots.sh android-phone emulator-5554
scripts/mobile-store-screenshots.sh android-tablet emulator-5554
```

It runs `integration_test/store_screenshots_test.dart` with
`test_driver/store_screenshot_driver.dart` (`flutter drive`), writes PNGs to
`mobile/store/screenshots/<set>/` (git-ignored) and fails if a picture isn't the set's
size. Boot the simulator or emulator first, at the set's size; set the status bar to 9:41
and full battery for the iPhone (`xcrun simctl status_bar booted override --time 9:41
--batteryState charged --batteryLevel 100 --cellularBars 4`). It needs the simulators and a
full build of the Rust library, so it is not run in CI.
