# Releasing Kanso for iPhone, iPad and Android

The mobile apps ship with every Kanso release: the desktop's `v*` tag also starts
`.github/workflows/mobile-release.yml`, which builds the signed iPhone/iPad app and
the Android App Bundle and sends them to testers (TestFlight and Play internal
testing). The desktop side is in [RELEASING.md](RELEASING.md); this page is the
mobile side, tracked in [#48](https://github.com/iamngoni/kanso/issues/48) (step 13).

The pipeline is built so that **nothing in it needs an account, key or paid action
to be green**. Every step that does is skipped, with a notice in the log, until its
secret is added; add them one at a time and each part becomes real. Section
["What the owner does"](#what-the-owner-does) is the one list of everything that
is left to do by hand.

## Contents

- [Versions](#versions)
- [What the workflow does](#what-the-workflow-does)
- [Signing](#signing)
- [Shorebird (over-the-air patches)](#shorebird-over-the-air-patches)
- [Export compliance](#export-compliance)
- [Privacy](#privacy)
- [Crash reports](#crash-reports)
- [Store listings](#store-listings)
- [What the owner does](#what-the-owner-does)
- [Cutting a mobile release](#cutting-a-mobile-release)
- [Known gaps](#known-gaps)

## Versions

One source of truth: **`[workspace.package] version` in the root `Cargo.toml`**,
which `scripts/release.sh` bumps. Kanso 0.9.0 is 0.9.0 on the desktop, the iPhone
and Android.

| | Value | From |
|---|---|---|
| Version name (iOS `CFBundleShortVersionString`, Android `versionName`) | `X.Y.Z`, the Cargo version | `scripts/mobile-version.sh` |
| Build number (iOS `CFBundleVersion`, Android `versionCode`) | the workflow's **run number** | `github.run_number` |

Both stores want the build number to grow with every upload of a version, and a run
number only ever grows. The tag must match Cargo's version (as the desktop's does),
and a version with a pre-release suffix is refused: the App Store and Play take only
dotted numbers. `scripts/mobile-version.sh` prints them and `scripts/test-mobile-version.sh`
tests it.

`mobile/pubspec.yaml`'s own `version:` is for local runs and is **ignored by every
release build**: the workflow passes `--build-name` and `--build-number` to Flutter
(and to Shorebird, which passes them on). Do not edit it for a release.

Re-running a *failed job* of a run reuses its run number, and a store refuses a build
number it has seen. If an upload got as far as the store, start a fresh run from
Actions › Mobile release › Run workflow instead. Renaming the workflow file
resets the run number: don't.

Shorebird names a release `version+number` (for example `0.9.0+57`); that is what a
patch asks for.

## What the workflow does

| Trigger | What happens |
|---|---|
| A `v*` tag | Both apps are built at the tag's version, checked, kept as artifacts for 30 days and uploaded |
| Run by hand on `master` | The same, for the version Cargo.toml says: a build for testers before the release |
| Run by hand on another branch | Built and checked only; nothing signed, kept or uploaded |
| Run by hand on `master` with `action: shorebird-patch` | A Shorebird patch, for the `release_version` you give, on the staging track unless `stage` is turned off |

Jobs (`.github/workflows/mobile-release.yml`):

**`version`** (Ubuntu, no secrets). Tests `scripts/mobile-version.sh`, checks the store
listing text against the stores' limits (`scripts/mobile-store-check.sh`), derives the
version and number, checks the tag, decides whether this run may upload (tag or `master`).

**`ios`** (macOS, environment `mobile-release` on tags and `master`).

| Step | Without its secrets | With them |
|---|---|---|
| Archive: `flutter build ipa --release --no-codesign` (or `shorebird release ios --no-codesign`) | Runs; the library in the archive and the version are checked | Same |
| Export compliance code into Info.plist | Notice | `IOS_EXPORT_COMPLIANCE_CODE` variable added |
| Export the IPA: `xcodebuild -exportArchive` with `ios/ExportOptions.plist` and the App Store Connect API key | Skipped, with a notice | Xcode signs for the App Store, and the IPA is checked (bundle id, version, number, iOS 17, team 4973Y692JU, Apple Distribution, no `get-task-allow`, the library) |
| Keep the debug symbols (`kanso-ios-dsyms-…`, 90 days), to read crash reports | Kept on tags and `master` | Same |
| Upload to TestFlight: `xcrun altool --upload-app` with the API key | Skipped | Uploaded; the IPA is also kept as an artifact |

**`android`** (Ubuntu, environment `mobile-release` on tags and `master`).

| Step | Without its secrets | With them |
|---|---|---|
| `flutter build appbundle --release` (or `shorebird release android --artifact aab`) | Signed with a **throwaway key made in the run** (never kept or uploaded) so the build and the Gradle signing config are exercised | Signed with the upload key from `android/key.properties`, written from the secrets and deleted at the end |
| Check the bundle (`scripts/mobile-release/aab-check.sh`) | Runs: the library for `arm64-v8a`, `armeabi-v7a` and `x86_64`, R8's mapping file, not signed with the debug key | Same, and the bundle and `mapping.txt` are kept as an artifact |
| Render the store pictures (`scripts/mobile-store-render.sh`) | Runs, and the pictures are kept as `kanso-store-pictures-…` | Same |
| Upload to Play internal testing (`r0adkll/upload-google-play`) with the mapping file and the release notes from `mobile/store/play/whatsnew/` | Skipped | Uploaded to the `internal` track |

Play serves each phone the library of its own ABI from the bundle (that is what "per-ABI
native libs in the bundle" means here); the check fails if one ABI is missing.

**`patch`** (macOS and Ubuntu, environment `mobile-patch`; only a manual run on `master`).
See [Shorebird](#shorebird-over-the-air-patches).

Every action is pinned to a commit SHA, as the rest of the repository's workflows are.
Two actions are new: `shorebirdtech/setup-shorebird` v1.4.1 and `r0adkll/upload-google-play`
v1.1.5. `r0adkll/upload-google-play` is the only third-party code that sees a store
credential (the Play service account). To remove that, replace it with a short script that
calls the Play Developer API; the rest of the job does not change.

## Signing

### iOS: the App Store Connect API key signs (3 secrets)

The iOS app is archived **unsigned** and exported with
`xcodebuild -exportArchive -allowProvisioningUpdates -authenticationKeyPath/ID/IssuerID`.
With *automatic* signing, Xcode uses the key to create or fetch an Apple Distribution
certificate and an App Store profile for `app.kanso.Kanso` (cloud-managed signing). The
same key then uploads the build. So the whole iOS side needs **three secrets** (the key,
its ID and the issuer ID) and no certificate (`.p12`) or provisioning profile to export,
renew or leak.

The alternative, manual signing, needs the key for the upload *and* a `.p12` with its
password *and* an App Store profile: six secrets, and two files that expire. It is the
fallback if cloud signing fails (below).

Two things to know:

- **The key needs the Admin role.** Cloud-managed distribution signing is refused to
  keys with less ("Cloud signing permission error"; reports in Apple's developer
  forums, [thread 698117](https://developer.apple.com/forums/thread/698117)). An Admin
  key can do most things in the team's App Store Connect, so it lives only in the
  `mobile-release` environment (tags and `master`, with a required reviewer), as the
  desktop's signing key does. Make it a key used for nothing else.
- **I couldn't run this path without an account.** Exporting an *unsigned* archive with
  cloud signing is the documented Shorebird flow for `--no-codesign` and is what
  `exportArchive` does (it re-signs), but the first run is the test. If it fails with a
  signing error, switch to manual signing: add the three secrets
  `IOS_DISTRIBUTION_CERT_P12_BASE64`, `IOS_DISTRIBUTION_CERT_PASSWORD` and
  `IOS_APPSTORE_PROFILE_BASE64`, import them into a temporary keychain as the desktop job
  does (`release.yml`, "Package (macOS)"), and change `ios/ExportOptions.plist` to
  `signingStyle` `manual` with `signingCertificate` `Apple Distribution` and
  `provisioningProfiles` `{ "app.kanso.Kanso": "<the profile's name>" }`.

The team is `4973Y692JU` and the bundle id `app.kanso.Kanso`, as in the Xcode project.
Extension targets (share extension, widgets: step 10) will need their own bundle ids
registered and their profiles are created by the same cloud signing; add them to
`ExportOptions.plist` only if you move to manual signing.

**Upload method.** `xcrun altool --upload-app -f <ipa> -t ios --apiKey <id> --apiIssuer <issuer>`
(the API key is read from `~/.appstoreconnect/private_keys/AuthKey_<id>.p8`, which the job
writes and removes). It is one of the four ways Apple lists in App Store Connect Help
([Upload builds](https://developer.apple.com/help/app-store-connect/manage-builds/upload-builds)),
and `altool` is only deprecated for *notarization*, which is `notarytool`'s job and not this
one. Apple's current `altool --help` also offers `--upload-package`; both take an IPA, and
`--upload-app` is what Apple's page shows. Xcode 26's `altool` can exit 0 after an error, so
the step also fails on any `ERROR` in its output. Alternatives if altool ever goes: Transporter
(`iTMSTransporter`), or `destination` `upload` in `ExportOptions.plist`, which makes
`xcodebuild -exportArchive` upload with the same key.

### Android: an upload key, Play App Signing (4 secrets + the Play key)

Google re-signs what Play serves with the **app signing key** it holds
([Play App Signing](https://support.google.com/googleplay/android-developer/answer/9842756));
CI signs with a separate **upload key**, which Google can reset if it leaks. Enrol the
app in Play App Signing when creating it (the default for new apps).

`mobile/android/app/build.gradle.kts` signs the release build with `android/key.properties`
if the file exists (CI writes it from the secrets; it and every `*.jks` are git-ignored) and
with the debug key otherwise, so `flutter run --release` still works on a checkout with no
key. `aab-check.sh` fails the job if a release bundle came out signed with the debug key.

R8 is on in release builds (`isMinifyEnabled`, `isShrinkResources`); `proguard-rules.pro`
keeps what the Rust library calls by name. The mapping file is uploaded with the bundle so
Play and the Play Console's crash reports de-obfuscate.

## Shorebird (over-the-air patches)

Decision 10 of #48: Dart-only fixes ship as `shorebird patch` without a store review; a
change to the Rust core, a plugin or `ios/` / `android/` ships as a store release. A patch is
a second way to deliver code to everyone's notes, so:

- **Patches are made only here**, from a manual run on `master`, in the `mobile-patch`
  environment (deployment branch `master` only, with a required reviewer). Nobody patches from a
  laptop; the `SHOREBIRD_TOKEN` and the patch signing key live only in these environments.
- **Patches are signed.** Shorebird's [patch signing](https://docs.shorebird.dev/code-push/guides/patch-signing/):
  an RSA key pair; the releases are built with the *public* key
  (`shorebird release … --public-key-path`), so the app installs only patches signed with the
  matching *private* key (`shorebird patch … --public-key-path … --private-key-path`). The
  private key exists only as the `mobile-patch` secret and in the owner's password manager.
- **A release is built with Shorebird only when it is set up**: `SHOREBIRD_TOKEN`,
  `SHOREBIRD_PATCH_PUBLIC_KEY` and `mobile/shorebird.yaml` all present. Otherwise the
  store build is plain Flutter (a notice says so). A build without the public key would
  accept *unsigned* patches, so the workflow never makes a Shorebird release without it.
- Patches go to the **staging track** by default (`stage`); promote one yourself after
  trying it: `shorebird patches promote --release-version 0.9.0+57 --patch-number 1`. Native
  and asset differences are refused (the workflow never passes `--allow-native-diffs`).
- The patch number appears in Settings › About and in crash reports: the app reads it from
  Shorebird's updater (`shorebird_code_push`), and shows none in a build made without Shorebird.

**Why `mobile/shorebird.yaml` isn't committed.** It holds the `app_id` that `shorebird init`
makes on Shorebird's server for the owner's account. It is not secret, but it is an account
value this change can't have. The one command, run once in `mobile/` after logging in:

```sh
cd mobile
shorebird login            # or export SHOREBIRD_TOKEN
shorebird init             # creates shorebird.yaml and adds it to pubspec.yaml's assets
git add shorebird.yaml pubspec.yaml && git commit -m "build(mobile): Set up Shorebird"
```

Commit both files. After that the workflow's Shorebird steps turn on by themselves once the
secrets are set.

**Credentials.** `shorebird login:ci` no longer works (its tokens stopped working on 1 October
2026): create an **API key** in the Shorebird Console (Account › API Keys, "Release and patch
only" is enough, with an expiry) and use it as `SHOREBIRD_TOKEN`. See
[CI](https://docs.shorebird.dev/code-push/ci/generic/).

**Patch signing key pair** (on your computer; never in the repository):

```sh
openssl genrsa -out shorebird-private.pem 2048
openssl rsa -in shorebird-private.pem -outform PEM -pubout -out shorebird-public.pem
gh secret set SHOREBIRD_PATCH_PRIVATE_KEY --repo iamngoni/kanso --env mobile-patch < shorebird-private.pem
gh secret set SHOREBIRD_PATCH_PUBLIC_KEY  --repo iamngoni/kanso --env mobile-patch < shorebird-public.pem
gh secret set SHOREBIRD_PATCH_PUBLIC_KEY  --repo iamngoni/kanso --env mobile-release < shorebird-public.pem
# keep both files in the password manager, then delete the private one from disk
rm shorebird-private.pem
```

**Risks to check at the first run** (none could be tried without an account):

- Shorebird builds with its own fork of Flutter. The workflow passes `--flutter-version=3.44.8`
  (the repository's pinned release). If Shorebird doesn't offer that version yet, `shorebird
  release` fails; pin the nearest version Shorebird has, here and in `mobile.yml`, or wait.
  `shorebird release` prints the versions it supports.
- The app builds its Rust library through a Dart build hook (`mobile/hook/build.dart`, native
  assets). Confirm `shorebird release` and `patch` run it, and that a patch compiled from
  the same sources reports no native differences.
- The Shorebird CLI is unpinned (`setup-shorebird` installs the latest; its servers refuse old
  CLIs over time). Pin `shorebird-version` only if a new CLI breaks the job.

## Export compliance

Kanso uses its own end-to-end encryption, so App Store Connect's export compliance
questions apply to the iOS app. Facts from the code (`docs/ACCOUNTS.md`): Argon2id,
HKDF, XChaCha20-Poly1305 and AES-256-GCM, X25519 and Ed25519, in Rust crates in the app
(not Apple's CryptoKit), sealing note text, titles, names and files before they are synced,
beside TLS through the platform's certificate verifier.

**What Apple says** ([Complying with Encryption Export Regulations](https://developer.apple.com/documentation/security/complying-with-encryption-export-regulations),
[Export compliance documentation for encryption](https://developer.apple.com/help/app-store-connect/reference/export-compliance-documentation-for-encryption),
[Determine and upload app encryption documentation](https://developer.apple.com/help/app-store-connect/manage-app-information/determine-and-upload-app-encryption-documentation)):

- `ITSAppUsesNonExemptEncryption` is `NO` if the app uses no encryption, "or if it only uses
  forms of encryption that are exempt from export compliance documentation requirements";
  otherwise `YES`. Encryption built into the OS (HTTPS through `URLSession`, say) is the
  typical exempt case.
- Apple's table: encryption **limited to the Apple OS** needs no documentation; an
  **industry-standard algorithm not provided within the OS** needs the **French encryption
  declaration**, and only if the app is distributed on the App Store in France; a
  **proprietary** algorithm needs a CCATS (US classification) as well.
- If the app uses exempt encryption, a **year-end self-classification report** to the US
  government might still be required; with non-exempt encryption and documentation provided
  to Apple it isn't. After Apple clears the documentation it gives a code to put in Info.plist
  as `ITSEncryptionExportComplianceCode`, so the questions aren't asked for every build.

**What applies to Kanso, and what this repo does.** Kanso uses standard (IETF) algorithms that
are not Apple's own, so it is in Apple's second row, not the first. Read literally,
"only forms of encryption exempt from documentation requirements" is then not true when the
app is available in France, so `mobile/ios/Runner/Info.plist` says
`ITSAppUsesNonExemptEncryption` = **`true`**. That is the conservative answer: it never claims
an exemption that isn't clear. It has a cost: until the owner answers, App Store Connect shows
the build as "Missing Compliance" and holds it back from testers.

**The answers are drafted** in [`mobile/store/app-store/export-compliance.md`](../mobile/store/app-store/export-compliance.md)
(provisional decision 2 of #48 step 13): standard algorithms not provided by the OS; the
mass-market exemption and publicly available source code for the US, which since BIS's 2021
rule needs nothing filed; and **France left off the app's availability** until the owner has
filed the ANSSI declaration. The owner submits the answers in App Store Connect and answers
for them. When Apple returns a code, put it in the `mobile-release` variable
`IOS_EXPORT_COMPLIANCE_CODE`; the workflow adds it to Info.plist as
`ITSEncryptionExportComplianceCode` before the build. If the owner ends up with `NO`, change
the value in `Info.plist` and say why in the commit.

Play has no export questionnaire; Google's Play Console declares US export laws in the
developer agreement.

## Privacy

`mobile/ios/Runner/PrivacyInfo.xcprivacy` (the app's privacy manifest) and the draft answers
in `mobile/store/app-store/privacy.md` and `mobile/store/play/data-safety.md` say the same
thing: no tracking and no analytics or crash SDK. **Only when the person signs in to sync**,
the sync server holds the account's username (a User ID). **Only when the person taps Send**
on a crash report, the server keeps that report (Crash Data, not linked to the person) for
90 days. Notes leave the phone only as ciphertext, which Apple's definition of "collect"
doesn't count. Open points are marked **Check** in those files.

## Crash reports

Provisional decision 1 of #48 step 13: no crash SDK and no crash-reporting account. The app
keeps a report on the device when it crashes, from iOS's MetricKit, Android's
`ApplicationExitInfo` (native crashes and ANRs), an uncaught JVM exception, or a Dart error.
It builds the report from a whitelist of fields: stack frames that match a strict grammar,
versions, the kind of device and the day, so note text can't reach it.
The person sees the exact text and sends it with **Send**; it goes to
`POST https://sync.kansoapp.app/v1/crash-reports`, which stores it in D1 for 90 days with no
IP address or account. [`docs/CRASH-REPORTS.md`](CRASH-REPORTS.md) defines every field.

The release keeps what reading a report needs: the iOS debug symbols (`kanso-ios-dsyms-…`,
90 days) and R8's mapping (in `kanso-android-…`). Before the first build that sends reports,
the Worker needs migration `0012_crash_reports.sql` (`cloud/README.md`).

## Store listings

Plain text for both stores is in `mobile/store/` (`mobile/store/README.md` explains each
file, the limits and the screenshot plan). The sizes are checked on every run of this
workflow (`scripts/mobile-store-check.sh`). The pictures for every required size, the Play
feature graphic and the 512 × 512 icon are rendered from the golden-test harness by
`scripts/mobile-store-render.sh`, in every iOS build job (macOS, where the goldens are made); they are the
`kanso-store-pictures-…` artifact. The text describes the app as designed in #48; trim
it to what has shipped before submitting (the table in `mobile/store/README.md` ties each claim
to its build step).

## What the owner does

Everything below needs an account, a console or a key, so none of it is done. Do it in this
order; the workflow works at every point in between.

**A. GitHub (repository settings)**

1. Settings › Environments › **New environment** `mobile-release`: deployment branches and
   tags → *Selected branches and tags* → add the tag rule `v*` and the branch rule `master`;
   add yourself as a *Required reviewer*.
2. **New environment** `mobile-patch`: deployment branch `master` only; a *Required reviewer*.

**B. Apple (App Store Connect, developer account, team `4973Y692JU`)**

3. Register the App ID `app.kanso.Kanso` (Certificates, Identifiers & Profiles › Identifiers)
   if it isn't (the Xcode project already uses it on the owner's iPhone), and create the app
   record in App Store Connect (Apps › + › New App, iOS, bundle `app.kanso.Kanso`, SKU of your
   choice).
4. Users and Access › Integrations › App Store Connect API › **Team Keys** › **+**: name it
   "Kanso mobile release", role **Admin**; note the **Key ID** and the **Issuer ID**; download
   `AuthKey_<KEYID>.p8` (once).
5. Secrets of `mobile-release`:
   ```sh
   base64 -i AuthKey_<KEYID>.p8 | gh secret set IOS_ASC_API_KEY_P8_BASE64 --repo iamngoni/kanso --env mobile-release
   gh secret set IOS_ASC_API_KEY_ID --repo iamngoni/kanso --env mobile-release --body <KEYID>
   gh secret set IOS_ASC_API_ISSUER_ID --repo iamngoni/kanso --env mobile-release --body <issuer-uuid>
   ```
   Keep the `.p8` in the password manager and delete the download.
6. Export compliance: the four steps above. Then `gh variable set IOS_EXPORT_COMPLIANCE_CODE --repo iamngoni/kanso --env mobile-release --body <code>`.
7. Fill the App Store page from `mobile/store/app-store/` (text, URLs, categories, privacy
   answers from `privacy.md`, App Review notes). Add the screenshots from the
   `kanso-store-pictures-…` artifact. Leave France off the availability until step 16 is done. Set up TestFlight
   internal testers (Users and Access, then TestFlight › Internal Testing).

**C. Google (Play Console, a developer account: a one-time registration fee)**

8. Create the app (name Kanso, package `app.kanso.kanso`), keep **Play App Signing** on.
9. Make the upload key on your computer and keep it in the password manager. If it is lost,
   Google can reset it, but that takes days:
   ```sh
   keytool -genkeypair -v -keystore upload-keystore.jks -alias upload -keyalg RSA -keysize 2048 -validity 10000
   ```
10. Upload the **first** bundle by hand (Testing › Internal testing › Create new release): the
    Play API refuses an app that has never had a release. A bundle from this workflow's
    artifacts (`kanso-android-…`) works once the key is set. While the app has no release,
    set the variable `PLAY_RELEASE_STATUS` to `draft` (`gh variable set PLAY_RELEASE_STATUS --repo iamngoni/kanso --env mobile-release --body draft`) and delete it after the first
    release.
11. Play Console › Setup › API access: link or create a Google Cloud project, create a **service
    account** with a JSON key, and invite its email in *Users and permissions* with permission to
    **release to testing tracks** for this app. Never create the key in the repository.
12. Secrets of `mobile-release`:
    ```sh
    base64 -i upload-keystore.jks | gh secret set ANDROID_UPLOAD_KEYSTORE_BASE64 --repo iamngoni/kanso --env mobile-release
    gh secret set ANDROID_UPLOAD_KEYSTORE_PASSWORD --repo iamngoni/kanso --env mobile-release
    gh secret set ANDROID_UPLOAD_KEY_ALIAS --repo iamngoni/kanso --env mobile-release --body upload
    gh secret set ANDROID_UPLOAD_KEY_PASSWORD --repo iamngoni/kanso --env mobile-release
    gh secret set PLAY_SERVICE_ACCOUNT_JSON --repo iamngoni/kanso --env mobile-release < play-service-account.json
    ```
13. Fill the Play listing from `mobile/store/play/` (text, URLs, category, the Data safety form
    from `data-safety.md`, content rating, and the screenshots, 512 × 512 icon and 1024 × 500
    feature graphic from the `kanso-store-pictures-…` artifact). Create the internal testers list.

**D. Shorebird (optional; releases work without it, but can't be patched later)**

14. Create the Shorebird account, then `cd mobile && shorebird init` and commit `shorebird.yaml`
    and `pubspec.yaml` (see above). **Do this before the first store build you want to patch**:
    a build made without Shorebird can never be patched.
15. Create an API key in the Shorebird Console, make the patch key pair, and set
    `SHOREBIRD_TOKEN` in both environments, `SHOREBIRD_PATCH_PUBLIC_KEY` in both and
    `SHOREBIRD_PATCH_PRIVATE_KEY` in `mobile-patch` only (commands above).

**E. Decisions and pages**

16. Confirm or overrule the export compliance answers (`export-compliance.md`); file the
    ANSSI declaration if Kanso should be on sale in France, then add France back.
17. Crash reporting needs no account. Deploy the Worker with migration `0012` before the
    first build that sends reports, if it isn't deployed yet (`cloud/DEPLOYED.md`).
18. Add the account-deletion web page Play asks for (the mobile app shows the link). The
    privacy, help and release-notes pages exist on kansoapp.app.
19. Register the secrets' owners and rotation dates in your password manager.

All secrets, in one table (names are exact):

| Name | Kind | Environment | What it is |
|---|---|---|---|
| `IOS_ASC_API_KEY_P8_BASE64` | secret | `mobile-release` | the App Store Connect API key (Admin role), base64 of the `.p8` |
| `IOS_ASC_API_KEY_ID` | secret | `mobile-release` | its Key ID |
| `IOS_ASC_API_ISSUER_ID` | secret | `mobile-release` | the team's Issuer ID |
| `IOS_EXPORT_COMPLIANCE_CODE` | variable | `mobile-release` | the code Apple returns once the export documentation is approved (optional) |
| `ANDROID_UPLOAD_KEYSTORE_BASE64` | secret | `mobile-release` | base64 of `upload-keystore.jks` |
| `ANDROID_UPLOAD_KEYSTORE_PASSWORD` | secret | `mobile-release` | the keystore password |
| `ANDROID_UPLOAD_KEY_ALIAS` | secret | `mobile-release` | the key's alias (`upload`) |
| `ANDROID_UPLOAD_KEY_PASSWORD` | secret | `mobile-release` | the key's password |
| `PLAY_SERVICE_ACCOUNT_JSON` | secret | `mobile-release` | the service account's JSON key, as text |
| `PLAY_RELEASE_STATUS` | variable | `mobile-release` | `draft` until the app's first release; unset after |
| `SHOREBIRD_TOKEN` | secret | `mobile-release`, `mobile-patch` | a Shorebird API key |
| `SHOREBIRD_PATCH_PUBLIC_KEY` | secret | `mobile-release`, `mobile-patch` | PEM public key; releases require patches signed by its pair |
| `SHOREBIRD_PATCH_PRIVATE_KEY` | secret | `mobile-patch` only | PEM private key; signs patches |

## Cutting a mobile release

1. Cut the desktop release: `scripts/release.sh 0.9.0` and push the tag (RELEASING.md). The
   tag starts both workflows; the mobile one is independent, and a mobile failure doesn't
   hold back the desktop release.
2. Approve the `mobile-release` environment when the run asks.
3. After the run: the build appears in TestFlight once App Store Connect has processed it
   (the "Missing Compliance" question, until step 6 above is done) and in Play's internal
   testing track a few minutes after the upload.
4. By hand, on real devices, before opening the build to more than the owner (#48, "Device
   tests before each release"): IME composition (Japanese, Chinese), dictation and
   autocorrect; VoiceOver and TalkBack; Pencil latency; 120 Hz scrolling and typing; background
   sync; a fresh install and an upgrade.
5. Promote: TestFlight external testing and App Store review from App Store Connect;
   Play internal → closed → production from the Play Console. (A new Play app made with a
   personal developer account must run a closed test with enough testers for 14 days before
   it can go to production; check Google's current rule.)
6. A Dart-only fix afterwards: merge to `master`, then Actions › Mobile release › Run
   workflow on `master`, `action: shorebird-patch`, `release_version: 0.9.0+<number>`.

## Known gaps

- **Attachment work is not finished:** sealed attachments (#44), PDF attachments (#43), and
  recordings (#38) are still outstanding. The existing app screens and the custom app icon
  are in place; do not describe either as unfinished or as Flutter placeholders.
- **Physical-device QA, TestFlight/Play uploads, and store submission are unverified.** Do
  the device checks in step 4 above before claiming the build is ready for testing.
- **iOS cloud signing and the Shorebird steps are untested** until the accounts exist (see
  above).
- **Mac Catalyst / "iPhone apps on Apple silicon Macs":** App Store Connect makes the iPhone
  app available on Apple silicon Macs by default. Turn that off under the app's Pricing and
  Availability if you don't want it (the desktop app is the Mac app).
