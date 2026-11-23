# Releasing Kanso

A release is a tag. `scripts/release.sh` cuts it, and the release workflow
builds every platform, signs the update feed, checks everything and
publishes the GitHub Release on its own. Publishing is the moment every
installed Kanso can see the update; if any check fails, nothing is
published.

The tag also starts the iPhone, iPad and Android builds, which go to
TestFlight and Play internal testing: see [RELEASING-MOBILE.md](RELEASING-MOBILE.md).

## How updates work

Every release carries, beside the downloads:

| Asset | What it is |
|---|---|
| `Kanso-<v>-macos-arm64.app.tar.gz` | `Kanso.app` alone, the macOS update (the `.dmg` is for first installs) |
| `Kanso-<v>-windows-x64.zip` | `kanso.exe`, the download and the update |
| `Kanso-<v>-windows-x64-setup.exe` | the per-user installer (`packaging/windows/kanso.iss`, Inno Setup 6.7.3), for first installs |
| `Kanso-<v>-linux-x64.tar.gz`, `…-linux-arm64.tar.gz` | the folder with `kanso`, the download and the update |
| `latest.json` | the feed: schema 1, version, date, release page, notes, and for each platform the update's URL, SHA-256 and size; optionally `whats_new` (the video's URL, SHA-256 and size) and `earlier` (the notes of up to 12 releases before) |
| `Kanso-<v>-whats-new.webp` | the release's What's new video ([below](#whats-new-video)) |
| `latest.json.sig` | an Ed25519 signature over the feed's exact bytes, base64 |
| `kanso_<v>-1_amd64.deb`, `…_arm64.deb` | the Debian packages, for apt ([apt repository](#the-apt-repository)) |
| `apt-stable-*` | the apt repository's signed index: `Packages` per architecture, `Release`, `InRelease`, `Release.gpg` (once the apt key is set up) |
| `SHA256SUMS` | every file's checksum |

The app (`crates/kanso-app/src/updates.rs`, with the logic in
`crates/kanso-update`) does this:

1. A few seconds after it opens, every half hour (five minutes after a
   failed check, and within a minute of waking from sleep), and on **Check for
   Updates…** (the app menu, the palette, Settings › Updates), it fetches
   `https://github.com/iamngoni/kanso/releases/latest/download/latest.json`
   and its `.sig`. GitHub answers `releases/latest` with the newest
   *published* release only, never a draft.
2. It checks the signature against the public keys compiled into it
   (`crates/kanso-update/keys/trusted-keys.txt`) before reading the JSON.
   Then it refuses a version that isn't newer than itself (no downgrade, no
   replay), a feed with no build for this system, and any URL that isn't
   `https://github.com/iamngoni/kanso/releases/…`. Every fetch, each hop of
   GitHub's redirects included, must be HTTPS on `github.com`,
   `objects.githubusercontent.com` or `release-assets.githubusercontent.com`.
3. With **Download and install updates automatically** on (the default), it
   downloads the update beside the install (in `.kanso-update/`, on the same
   volume, made afresh for each download and readable only by its owner),
   checks its size and SHA-256 against the signed feed, and unpacks it,
   taking only plain files that stay inside the folder. On Linux it also
   stages the runtime support files the archive installer puts in
   `~/.local/lib/kanso` (ONNX Runtime, PDFium, the XKB/XCB bridge and their
   licence folders) separately from the executable. It then takes a digest
   of what it unpacked and of the support folder, which it keeps in memory
   only. On macOS the staged `.app` must pass `codesign --verify --deep
   --strict`, be signed with the running Kanso's identifier and, when the
   running Kanso is signed by a developer, carry the same Team ID.
4. The sidebar shows **Update ready · Restart**. Its sheet has the release
   notes with their commit links, and **Restart now** or **Later**. Restart
   now, or any quit, checks that the staging folder and the digests are
   still the ones it took (anything changed or planted there is refused),
   then swaps the update in. On Linux the runtime support folder, plugin
   host and executable are moved into place as one transaction; if a later
   rename fails, the earlier ones are put back, so an old ONNX Runtime or
   XKB library cannot be left beside a new binary. The other platforms use
   the same two-rename swap for their executable or app bundle, removes
   macOS's quarantine flag, and starts the new Kanso with `--after-update`,
   which waits for the old one to let go of the data folder. On Windows the
   running `kanso.exe` becomes `kanso.exe.old`, which the next start
   deletes. An update staged but never swapped in (Kanso was force-quit,
   say) is deleted at the next start and downloaded again, never installed
   from disk.
5. The first start of the new version shows **Updated to Kanso <v>** once,
   with What's new. It reads the release's own feed by tag
   (`releases/download/v<v>/latest.json`, signature checked the same way)
   for its What's new video and the notes of any releases skipped.

With automatic updates off, the sidebar says **Update available ·
Download** and Download fetches and stages it the same way. A copy that
can't replace itself opens the release page instead, and Settings › Updates
says why: a debug or `cargo` build, Kanso Dev (never updated, see
[DEVELOPING.md](DEVELOPING.md)), an app macOS is running from a temporary
copy (App Translocation) or from its disk image, an AppImage, a folder
Kanso can't write to (Program Files, say), or an install that belongs to
another account on the computer. A copy apt installed (under `/usr`, not
`/usr/local`) is apt's: it still says a release is out, and its button
copies `sudo apt upgrade` rather than downloading or opening anything.

Failures stay quiet and in words: offline, rate limited, no release yet, an
update that failed a check, no space. The sidebar only mentions the ones that
need a person (a failed check, a full disk). Each step is logged to
`<data folder>/logs/updates.log` and to standard error.

`KANSO_UPDATE_FEED=<url>` points a **debug** build at another feed (it may be
`http://127.0.0.1:<port>/…`). Release builds ignore it, and the feed must
still be signed by a trusted key.

**Kanso Nightly** runs the same update code against
`https://github.com/iamngoni/kanso/releases/download/nightly/latest.json`,
with the nightly public key from
`crates/kanso-update/keys/nightly-keys.txt` and a prerelease version. Kanso
itself keeps using `releases/latest` and the keys in `trusted-keys.txt`; it
never reads the nightly feed. Because nightly versions are prereleases, the
first start of a new nightly does not fetch What's new (`refresh_installed`
only runs for a final version), and `whats_new` is not put in the nightly
feed.

## What's new video

From 0.10.7 a release carries a short video that leads the What's new
sheet (desktop and phones), above the notes. From 0.10.9 making it is part
of cutting the release: `scripts/release.sh` makes one whenever
`release-media/<v>/whats-new.webp` isn't there, and a release whose video
can't be made stops.

**Format: animated WebP, one file, at most 8 MB.** GPUI has no video
player, but its `img` decodes and loops animated WebP (and GIF) frames;
Flutter's `Image` plays the same file on iOS and Android, so there is no
second rendition (no MP4). WebP is far smaller than GIF at the same
quality, and has no sound track, so the video is always muted; it loops.
Both apps fetch it through the same checks, its size (at most 8 MB,
`kanso_update::whats_new::MAX_MEDIA_BYTES`) and SHA-256 against the signed
feed, and neither limits its length: the desktop sheet shows it in a 16:9
box (`ObjectFit::Cover`), the phones in a 16:9 `AspectRatio`.

**Memory sets the size.** Both apps decode every frame into memory as RGBA:
frames x width x height x 4 bytes. So the video is **640x360 at 10 fps, at
most 30 seconds and 300 frames**: 0.92 MB a frame, at most about 276 MB,
and less in practice, since WebP merges frames that don't change (a UI
video holds far fewer frames than its length suggests). The 960x540 at
12 fps used before would be up to 746 MB for the same 30 seconds.

### How it's made

The video shows nearly every user-facing change in the release, a scene
each, about 2.5 seconds apiece. `scripts/whats-new-video.sh <v>
[--brief "…"]` (run by `release.sh`, or by hand to try one):

1. reads the release's notes from `CHANGELOG.md` (`kanso-release notes`,
   the same notes the release page and the app show) and lists their
   user-facing entries: everything under New, Changed, Improved, Fixed and
   Breaking whose scope isn't internal (ci, refactor, tests, docs, site,
   release, build, chore, deps, bench);
2. has Claude Code, headless (`claude -p`), write the **video script**:
   a scene per change, closely related changes grouped into one ("Collab:
   shared properties" for two commits about it), each with a headline, a
   one-line description, what the UI mock shows and the commits it covers.
   A reviewed `release-media/<v>/script.md` already there is used instead.
   `--brief` (or `WHATS_NEW_BRIEF=… scripts/release.sh …`) steers it;
3. checks the script covers **every** user-facing entry's commit and has at
   most 12 scenes (12 x 2.5 s = 30 s). A release with more changes has its
   changes grouped further; one left out stops the script, never silently;
4. copies `release-media/template/` (a HyperFrames project; its
   `compositions/example-s*.html` are 0.10.6's three scenes) to
   `target/whats-new/<v>/` and has Claude Code, with the hyperframes skills,
   write a scene per script scene in that style: the "0N · v<v>" headline
   card on the left and a mock of Kanso's UI using the feature on the
   right, with text large enough to read at 640x360;
5. writes `index.html`, lints, renders with the pinned HyperFrames CLI and
   converts with ffmpeg (or, when ffmpeg has no `libwebp_anim`, as
   Homebrew's hasn't, ffmpeg's frames through `img2webp` with the same
   settings):

   ```sh
   ffmpeg -i whats-new.mp4 -vf "fps=10,scale=640:-2:flags=lanczos" -loop 0 \
     -c:v libwebp_anim -quality 70 -compression_level 6 -an whats-new.webp
   ```

6. checks it is an animated WebP, 640x360, at most 8 MB and 300 frames;
7. saves a still from each scene under
   `target/whats-new/<v>/.hyperframes/artifacts/snapshots/`, and writes
   `release-media/<v>/whats-new.webp` and `release-media/<v>/script.md`.

It needs cargo, node, ffmpeg (or img2webp) and the `claude` CLI with the
hyperframes skills. **Read the stills before pushing the tag**: `release.sh`
commits the video and its script with the release, so a scene with wrong
words or a broken layout would ship. If one is wrong, delete the tag and
release commit (`git tag -d v<v> && git reset --hard HEAD~1`), fix the
scene in `target/whats-new/<v>/` and re-render, or edit the script and run
the script again; commit `release-media/<v>/` on master and cut again: a
video that's already there is used as it is.

A video can also be made any other way (a screen recording, say) and
committed at `release-media/<v>/whats-new.webp` before cutting.

`scripts/release.sh <v> --no-video` ships without one. It is for
emergencies (a security fix that can't wait for a render) and says so
loudly; the release then has no video at all.

### Shipping it

1. `release.sh` checks `release-media/<v>/` holds `whats-new.webp` (and
   its `script.md`) and nothing else, refuses a file that isn't WebP or is
   over 8 MB, and commits them in `chore(release): v<v>`.
2. The feed job copies it to `Kanso-<v>-whats-new.webp`, and
   `kanso-release feed --whats-new` adds its URL, SHA-256 and size to
   `latest.json` as `whats_new`, under the feed's signature; the publish
   job uploads it with the other assets.

The apps fetch it only from this repository's releases, with every
redirect hop held to the same rules as an update, check its size and
SHA-256 against the signed feed before showing it, and keep it in
`<data folder>/whats-new/` (one release's at a time). A video that fails a
check is dropped and the notes show alone. Kanso up to 0.10.6 ignores the
`whats_new` and `earlier` fields: serde skips fields it doesn't know, and
the signature covers the bytes either way.

The feed also carries `earlier`, the app's notes of up to 12 releases
before (from `CHANGELOG.md`, `--changelog`), so the sheet can list the
releases someone skipped under a folded **Also since your last update**.

## Changelog

Commits follow [Conventional Commits](https://www.conventionalcommits.org).
`cliff.toml` turns them into `CHANGELOG.md` with git-cliff:

| Commit type | Section |
|---|---|
| any type marked breaking (`feat!:`, or a `BREAKING CHANGE` footer) | Breaking (first; needs an update notice, below) |
| `feat` | New |
| `perf`, `style`, `ui`, `refactor(ui)` | Improved |
| `fix`, `revert` | Fixed |
| `refactor`, `build`, `ci`, `chore`, `test`, `docs` | Under the hood (left out of the app's What's new) |

Every entry links its short sha to the commit, `(#n)` links the pull
request, and each release's heading links the compare view with the release
before. The release page and the feed's `notes` are the same section of
`CHANGELOG.md`; the app leaves out Under the hood, which the release page
keeps. Merges and commits that aren't conventional stay out.

**That last rule loses work silently when a contributor's subject isn't
conventional** — it bit v0.9.0, where a contributor's sticky notes feature
(+1,600 lines, PR #168) was dropped from the release notes and the changelog
because its subject read "add sticky notes functionality". Before publishing,
read the release's commits beside the generated section:

```sh
git log --no-merges --format='%h %an  %s' v0.8.4..v0.9.0
```

Every line should either appear in the section or be deliberately left out
(formatting-only subjects, release chores). Anything else is a real change
missing from the notes: add it by hand in the section it belongs to, with the
commit link and "by @login" credit. Rewording the commit is the alternative,
but that rewrites published history.

An entry whose commit someone else wrote ends "by @login" (from their GitHub
no-reply address; otherwise "by" and their name), so each feature is credited
to the person who made it. Keep contributors' commits as they wrote them
(merge, don't squash) for this to work.

## Breaking changes

A change that breaks something people rely on ships an **update notice**:
a banner and a How to fix sheet in the app, with the steps to fix what
broke (issue #243). The release tooling refuses a breaking change without
one.

1. Add the notice to `NOTICES` in `crates/kanso-core/src/notices.rs`: a
   never-reused `id` (`api-keys-0.10`), the `since` version, a title, what
   changed, numbered `steps`, and an `applies` check so only people the
   change affects see it.
2. Mark the commit breaking (`feat(api)!: …`, or a `BREAKING CHANGE:`
   footer) and add a footer naming the notice, in the message's last
   paragraph with any other footers:

   ```text
   feat(server)!: Drop the local API's username and password

   Notice: api-keys-0.10
   ```

   The ID is lower-case letters, digits, dots and hyphens; a footer with
   any other ID isn't printed, so the gate then finds no notice. A
   contributor's commit needs the same footer, and still ends "by @login".
3. git-cliff then lists it under **Breaking** and ends the entry with
   `(notice: api-keys-0.10)`, after the credit. For an entry written by
   hand, end it with that text yourself. A bullet nested under an entry is
   detail: the entry's own text is what has to end with the notice.

`kanso-release check-notices --changelog CHANGELOG.md --version <v>` holds
the release to this. It fails when a Breaking entry doesn't end with
`(notice: <id>)`, when the ID isn't in `NOTICES`, or when the notice starts
after this version (a prerelease comes before its release, so
`0.10.0-beta.1` can't ship a notice whose `since` is `0.10.0`, which the
beta wouldn't show). It also fails when a notice in the list has a
malformed or repeated ID, or no title, no explanation or no steps, and when
`<v>` has no section in the changelog. Without `--changelog` it checks only
the list. Each problem it prints says how to fix it.

`scripts/release.sh` runs it after writing the changelog section; when it
fails, the script puts `CHANGELOG.md`, `Cargo.toml` and `Cargo.lock` back
and stops before committing or tagging. The release workflow runs it again
in the feed job, before the feed is made or signed, so a tag pushed by hand
(or a section amended after `release.sh`) can't ship without a notice
either: the release isn't published.

**A breaking commit already on `master` without its footer** can't get one
without rewriting published history. Write the release's section ahead
instead: run `git cliff --unreleased --tag v<version> --strip header`, put
its output at the top of `CHANGELOG.md` (below the header), end the
Breaking entry with `(notice: <id>)`, commit that to `master` and push,
then run `scripts/release.sh <version>` again. It keeps a section written
ahead and only sets its date.

`scripts/test-cliff-notices.sh` checks all of this against fixture commits
in a throwaway repository (git-cliff and the gate together), as
`scripts/test-cliff-credit.sh` does for credit. Run both after changing
`cliff.toml`.

## Cutting a release

From a clean `master` that matches `origin/master`:

```sh
scripts/release.sh 0.2.0          # bump, changelog, commit, tag
# read the new section of CHANGELOG.md; amend it if it needs words
git push origin master && git push origin v0.2.0   # or pass --push above
# the workflow builds, checks and publishes it
```

`release.sh <version>`:

1. refuses anything but a clean `master` equal to `origin/master`, a tag
   that exists already, and a checkout with no trusted update key;
2. sets `[workspace.package] version` and updates `Cargo.lock`;
3. adds the release's section to `CHANGELOG.md` with
   `git cliff --unreleased --tag v<version>` (a section written ahead, as
   0.1.0's was, only gets today's date);
4. runs `kanso-release check-notices` on that section (above), and puts
   back what it changed if a breaking change has no notice;
5. commits `chore(release): v<version>` and tags `v<version>`, then prints
   the push commands. It pushes only with `--push`.

The tag starts `.github/workflows/release.yml`, which builds each platform,
signs and notarizes the Mac app when the Apple secrets are set, and makes
the update archives. Linux releases also include `Kanso-<version>-linux-x64-headless.tar.gz`
and `Kanso-<version>-linux-arm64-headless.tar.gz`, each with `kanso-headless` and
`install.sh --headless`. The Linux packaging job checks the binary with `ldd` for Wayland, X11,
Vulkan and libfido2, then exercises `serve`, account status and MCP initialization on the
display-less runner. The GUI archive also carries `libxkbcommon-x11.so.0` and `libxcb-xkb.so.1`; the
first is patched to have RUNPATH `$ORIGIN` so the loader finds the second beside it. Its `feed` job
then writes `latest.json` from them with
the release's section of `CHANGELOG.md` (a tag without one fails), signs it
with `KANSO_UPDATE_SIGNING_KEY`, and verifies it against the keys in the
checkout. A missing or untrusted signing key fails the release, and so does
a tag without the Developer ID secrets: the Mac build must be signed,
notarized and accepted by Gatekeeper (`spctl`). Last, the `publish` job
checks every file against `SHA256SUMS`, uploads everything as a draft, and
publishes it as the latest release once every file is up.

The signing key only reaches the `feed` job, which runs in the `release`
environment with a read-only token, no third-party actions, and the key in
one step that runs a `kanso-release` built in an earlier one. Only `publish`
can write to the repository, and it runs no code from it. Every action is
pinned to a commit; bump the SHAs deliberately.

`release.sh publish <version>` is only for a release left as a draft (by a
run stopped between upload and publish): it downloads the draft's
`latest.json` and `.sig`, verifies them, and publishes it.

## Nightly builds

`.github/workflows/nightly.yml` builds **Kanso Nightly** from `master` at
02:00 UTC. It builds only when master's latest CI run is green and the
moving `nightly` tag does not already point at that commit; run it by hand
(Actions › Nightly › Run workflow) to build the current master anyway.

The version is the Cargo version with a prerelease suffix,
`0.10.11-nightly.202611220200`, compiled in through
`KANSO_NIGHTLY_VERSION` so the app, its metrics and the feed agree. The
suffix is the UTC build time as one number, `YYYYMMDDHHMM`: it always
increases, so a later nightly is always newer, and it can't start with a
zero, which semver forbids in a number (a commit hash could, so it is not
used; the notes name the commit instead). The four
update archives, the macOS disk image, the Windows installer and the signed
`latest.json` go to the `nightly` tag, which stays a prerelease; each run
replaces the assets (the earlier nightly's are deleted) and, only once they
are uploaded, moves the tag. The notes are the commit the build is from and
the merged PR titles
since the last nightly (`kanso-release notes --commits`), not a
`CHANGELOG.md` section. The feed is signed
with `NIGHTLY_UPDATE_SIGNING_KEY`, which must be one of the keys in
`crates/kanso-update/keys/nightly-keys.txt` (`kanso-release sign --nightly`
refuses any other), and checked against those keys and the
`NIGHTLY_UPDATE_PUBLIC_KEY` repository variable before upload. A failed
nightly leaves the previous one in place.

On Linux, `install.sh --nightly` uses Kanso's layout under Kanso Nightly's
names: `~/.local/bin/kanso-nightly`, with its libraries, PDF helper and
plugin host in `~/.local/lib/kanso-nightly`. The updater swaps that folder
with the binary, and never touches `~/.local/bin/kanso-plugin-host`, which
is Kanso's.

Kanso Nightly is signed and notarized like a release (the workflow refuses
to publish an ad hoc Mac build), installs beside Kanso with its own bundle
ID, data folder and Keychain service, and updates itself from the nightly
feed. See [DEVELOPING.md](DEVELOPING.md) for its local build.

## One-time setup

Do these once, before the first release; the release workflow can't sign a
feed without the first.

### 1. The update signing key (required)

```sh
cargo run -p kanso-update --bin kanso-release -- keygen --secret-out ~/kanso-update-signing.key
```

It prints the public key. Add it on its own line to
`crates/kanso-update/keys/trusted-keys.txt` and commit that (`build: Trust
the update signing key`), so every build from then on checks updates with
it. Then give the secret to the workflow and put it somewhere safe:

First make the `release` environment (Settings › Environments › New
environment, `release`). Under **Deployment branches and tags** choose
**Selected branches and tags** and add the tag rule `v*`, so no branch, pull
request or hand-run workflow can reach the key; adding yourself as a
**Required reviewer** also makes every signing wait for your approval. Then
store the key as that environment's secret, not the repository's:

```sh
gh secret set KANSO_UPDATE_SIGNING_KEY --repo iamngoni/kanso --env release < ~/kanso-update-signing.key
# keep a copy in your password manager, then
rm ~/kanso-update-signing.key
```

A repository-wide secret also works, but any workflow on any branch can read
one.

Whoever holds this key can ship code to everyone running Kanso: keep it only
in the password manager and the environment secret. If it leaks, add a new
key to `trusted-keys.txt`, release with the old one one last time (installed
copies only trust the old one), switch the secret to the new key, remove the
old line, and release again. There is no revocation: a copy that hasn't
taken that release still trusts the leaked key, so whoever holds it can
update that copy too (given a release on this repository as well). Move fast,
and say so in the release notes.

Also, in Settings › Actions › General › **Fork pull request workflows from
outside collaborators**, choose **Require approval for all outside
collaborators**. CI runs on the self-hosted Macs, and a pull request's
workflow is the one in the pull request, so the `if:` that keeps forks off
those Macs can be edited away by the fork itself; only the approval stops
it.

To check the key and the secret match, sign a test feed the same way the
workflow does: `kanso-release sign` refuses a key whose public half isn't in
`trusted-keys.txt`.

### 2. The nightly signing key (required for Kanso Nightly)

Kanso Nightly has its own key, separate from the release key; never reuse
one. Generate it the same way:

```sh
cargo run -p kanso-update --bin kanso-release -- keygen --secret-out ~/kanso-nightly-signing.key
```

Add the public key it prints on its own line to
`crates/kanso-update/keys/nightly-keys.txt` and commit that. Only Kanso
Nightly compiles that file in. Then make the `nightly` environment (Settings
› Environments), restrict **Deployment branches** to `master`, and store the
secret there:

```sh
gh secret set NIGHTLY_UPDATE_SIGNING_KEY --repo iamngoni/kanso --env nightly < ~/kanso-nightly-signing.key
# keep a copy in your password manager, then
rm ~/kanso-nightly-signing.key
```

Set the public half as a repository variable, which the workflow verifies
the signed feed against before publishing:

```sh
gh variable set NIGHTLY_UPDATE_PUBLIC_KEY --repo iamngoni/kanso --body '<the public key>'
```

The `nightly` environment also needs the Apple Developer ID and
notarization secrets from step 4, because every nightly Mac build is signed
and notarized. A missing key or secret fails the nightly before anything is
published.

### 3. The theme gallery signing key (set up)

The theme gallery has its own key, separate from the update key; never reuse
one for the other. The key and the `gallery` environment already exist, so
there is nothing to generate:

- The public half `gAD6MZmqRxzGkbeLh0ofxcVn5UlW+AaA7157L15Qd4Y=` is in
  `crates/kanso-gallery/keys/themes-trusted-keys.txt` here and in
  `gallery/trusted-keys.txt` in
  [`iamngoni/kanso-themes`](https://github.com/iamngoni/kanso-themes). Keep
  the two lists identical.
- `kanso-themes` has a `gallery` environment that deploys only from `main`
  and requires the owner's approval. Its only secret is
  `KANSO_THEMES_SIGNING_KEY`.
- That secret was made with `kanso-release keygen`, which writes the base64
  of a 32-byte Ed25519 seed. `gallery-sign` reads that format as is; no
  conversion and no `gallery-sign keygen` is needed. Do not generate a new
  key unless rotating: a shipped Kanso would not trust it.

The signing job builds `tools/gallery-sign` with `--locked` in a step with no
secret, then runs that binary with the key only in the signing step's
environment. The separate publish job uses the workflow's built-in
`GITHUB_TOKEN` with `contents: write` and holds no secret; its pushes do not
start another workflow, which is wanted. A push cannot forge a signature, so
a compromise can only change files whose hashes no longer match, delete
files, or replay an older index (which the serial floor refuses).

To rotate: generate a new key with `kanso-release keygen`, add its public
half to both lists, ship a Kanso release trusting both, switch
`KANSO_THEMES_SIGNING_KEY` to the new secret, then remove the old line from
both lists once the oldest supported Kanso trusts the new key. Never reuse
the update key.

### 4. Developer ID signing and notarization (optional)

Without these, the Mac app is signed ad hoc as before: it updates itself,
but a first install needs right-click › Open. With them, it is signed with
your Developer ID and the hardened runtime, notarized and stapled, and the
updater also requires every update to carry the same Team ID (4973Y692JU).

**The certificate.** In Keychain Access, under login › My Certificates, find
"Developer ID Application: Ngonidzashe Mangudya (4973Y692JU)". Open its
disclosure arrow to check the private key is there, select the certificate,
choose File › Export Items…, save as `DeveloperID.p12` and set a password.
Then:

```sh
base64 -i DeveloperID.p12 | gh secret set MACOS_CERT_P12_BASE64 --repo iamngoni/kanso --env release
gh secret set MACOS_CERT_PASSWORD --repo iamngoni/kanso --env release   # paste the password
rm DeveloperID.p12
```

**The App Store Connect API key**, for `notarytool`. On
[App Store Connect](https://appstoreconnect.apple.com), go to Users and
Access › Integrations › App Store Connect API › Team Keys, press **+**,
name it "Kanso notarization" with the **Developer** role, and Generate.
Note the **Key ID** beside it and the **Issuer ID** above the list, and
download `AuthKey_<KEYID>.p8` (Apple lets you download it once). Then:

```sh
base64 -i AuthKey_<KEYID>.p8 | gh secret set APPLE_API_KEY_P8_BASE64 --repo iamngoni/kanso --env release
gh secret set APPLE_API_KEY_ID --repo iamngoni/kanso --env release --body <KEYID>
gh secret set APPLE_API_ISSUER --repo iamngoni/kanso --env release --body <issuer-uuid>
```

Keep the `.p8` in the password manager and delete the download. The
workflow signs and notarizes only when all five Apple secrets are set.

**The provisioning profile**, for native passkeys (docs/ACCOUNTS.md › Where
the ceremony runs). Associated Domains (`webcredentials:sync.kansoapp.app`)
is a restricted entitlement: macOS launches the app with it only when
`Kanso.app/Contents/embedded.provisionprofile` authorizes it. The profile is
"Kanso Developer ID" in the developer account: a Developer ID profile for
`4973Y692JU.app.kanso.Kanso` with Associated Domains, made for the Developer
ID Application certificate above (it expires 2031-01-16; renew the profile
with the certificate). Then:

```sh
base64 -i Kanso_Developer_ID.provisionprofile | gh secret set MACOS_PROVISION_PROFILE_BASE64 --repo iamngoni/kanso --env release
```

The profile isn't secret, but it lives beside the certificate. With it and
the signing secrets, the workflow passes it to `scripts/bundle-macos.sh` as
`KANSO_PROVISION_PROFILE`, which embeds it and keeps the entitlement. Without
it, the script builds without Associated Domains and says so: the app runs
as before, and platform passkeys say they aren't available. To build a
release with passkeys locally, set both `KANSO_SIGN_IDENTITY` and
`KANSO_PROVISION_PROFILE` (for example
`~/.config/kanso/Kanso_Developer_ID.provisionprofile`).

### 5. The built-in themes

This release keeps every built-in color and icon theme compiled in: they are
the offline, always-there set, and they keep every existing selection
working without a migration. A later release may shrink to Kanso Light,
Kanso Dark and Lucide, but only with a migration that installs gallery copies
of any selected theme before removing the built-in one. Mark the gallery
entries for those ids as **Built in** until then; the gallery never installs
over a built-in id.

### 6. The apt repository (optional)

Until this is done, releases carry the `.deb`s and the release job logs
"apt signing key not configured; skipping apt metadata". Once:

1. Run `scripts/apt-setup.sh` from the repository's root, signed in to `gh`
   as an admin of the repository. It makes an ed25519 signing key in a
   temporary GnuPG home, stores it and a random passphrase as
   `APT_SIGNING_KEY` and `APT_SIGNING_PASSPHRASE` in the `release`
   environment (only `v*` tags reach them), offers a backup to a folder
   outside the repository, and writes `site/public/apt/kanso.gpg` and
   `packaging/linux/apt.conf`. It never writes the private key or the
   passphrase into the repository.
2. Commit those two files.
3. Deploy the site once (`cd site && npm run deploy`), so
   `https://kansoapp.app/apt/kanso.gpg` exists. The site needs no deploy per
   release.
4. Tag a release: it publishes the repository.

The key expires after the time you chose (2 years by default). Before then,
extend it from the backup (`gpg --quick-set-expire`), store it again with
`gh secret set APT_SIGNING_KEY --env release`, and commit the re-exported
`kanso.gpg`. `scripts/apt-setup.sh --rotate` replaces the key outright;
everyone who installed with apt must then download `kanso.gpg` again, so
avoid it unless the key is lost or leaked.

## The apt repository

`kansoapp.app/apt` is an apt repository whose files all live on the GitHub
Release; the site only redirects ([APT.md](APT.md) has the design):

| URL under `https://kansoapp.app/apt/` | Served as |
|---|---|
| `kanso.gpg` | `site/public/apt/kanso.gpg`, a static file |
| `dists/stable/InRelease`, `Release`, `Release.gpg`, `main/binary-{amd64,arm64}/Packages(.gz)` | 302 to `releases/latest/download/apt-stable-…`, never cached |
| `pool/<tag>/kanso_<v>_<arch>.deb` | 302 to `releases/download/<tag>/…` |

Anything else under `/apt` is a 404 (`site/src/lib/apt.ts`). The redirects
can't change what apt gets: `InRelease` is checked against the key, and
`Packages` pins each `.deb`'s SHA-256. Since `releases/latest` is the newest
published release, the index holds the latest release only; older versions
aren't installable through apt.

In the release workflow, each Linux build runs `scripts/deb-build.sh` after
the tarball (cargo-deb 3.8.0, `[package.metadata.deb]` in
`crates/kanso-app/Cargo.toml`); a failure there leaves the release without
`.deb`s rather than stopping it. The `apt` job then runs
`scripts/apt-release.sh`, which imports the key and runs
`scripts/apt-index.sh`: `apt-ftparchive packages` over `pool/<tag>/`, then
`Release` over the index, then the signatures, nothing touched after. Its
files reach the feed job's `SHA256SUMS` and the release like every other.

To try it without GitHub: build the `.deb` (`scripts/deb-build.sh 0.9.0
/tmp/debs`), make a throwaway key in a temporary `GNUPGHOME`, run
`APT_SIGNING_FINGERPRINT=<fpr> scripts/apt-index.sh /tmp/debs v0.9.0
/tmp/apt`, and serve `/tmp/apt` behind redirects shaped like the Worker's
([APT.md](APT.md) › Local testing).

## Testing an update

`cargo test -p kanso-update` runs the property tests and an end-to-end test
that serves a signed feed from a local HTTP server on an ephemeral port and
updates a fake Kanso in a temporary folder: check, download, verify, stage,
swap. Nothing outside the temporary folders is touched.

To watch the app do it, run a debug build with `KANSO_UPDATE_FEED` pointing
at a feed signed with a key in `trusted-keys.txt`. A debug build never
replaces itself; it shows the update and offers the release page.

The theme gallery's end-to-end test is in `crates/kanso-gallery/tests/loopback.rs`:
it serves a signed catalog over loopback, installs and updates a theme, sees
it revoked, and refuses a replayed index. A debug build can point Browse at a
local catalog with `KANSO_GALLERY_INDEX=<url>` (loopback is allowed there);
the index must still verify against a compiled-in themes key.
