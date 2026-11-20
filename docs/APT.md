# Kanso on apt

How Kanso is published as an apt repository at `kansoapp.app/apt`: the
handoff this work was built from, with its open questions answered, and
[As built](#as-built) at the end for where the code differs from the plan
and why. [RELEASING.md](RELEASING.md) has the short version for cutting
releases.

Once the maintainer has run the one-time setup, Debian and Ubuntu users do:

```bash
curl -fsSL https://kansoapp.app/apt/kanso.gpg | sudo tee /usr/share/keyrings/kanso.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/kanso.gpg] https://kansoapp.app/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/kanso.list
sudo apt update && sudo apt install kanso
```

**Repo:** `iamngoni/kanso`, upstream, Rust/GPUI, AGPL-3.0. The work was done
on the fork `Zawala/kanso`, as one PR against `iamngoni/kanso:master`.
**Contributor:** Kelvin (the fork's owner). **Maintainer:** Ngoni
(iamngoni), who owns the signing key, the secrets and the site's deploys.

---

## 1. Decisions (settled)

| # | Decision | Value |
|---|---|---|
| 1 | Hosting | `kansoapp.app/apt`, served by the existing site Worker in `site/` |
| 2 | Signing key | The maintainer generates it with `scripts/apt-setup.sh`. The private key never enters the repo. |
| 3 | Package name and maintainer | `kanso`; `Maintainer: Ngonidzashe Mangudya <imngonii@gmail.com>` (confirmed by Kelvin) |
| 4 | Updates for apt installs | Left to `apt upgrade`. Kanso still checks for updates and tells the user, but never replaces itself. |
| 5 | Channels | Only `stable` is implemented. The suite name is a parameter, so `unstable` can be added without restructuring. |
| 6 | Copy button | The apt-installed updater offers a button that copies `sudo apt upgrade` (confirmed by Kelvin). |

---

## 2. Facts about the repo (as of 2026-11-01)

- Linux shipped only `Kanso-<v>-linux-{x64,arm64}.tar.gz` and
  `assets/linux/install.sh` (which installs to `~/.local/bin`). There was no
  `.deb`.
- The binaries are `kanso` (crate `kanso-app`) and `kanso-plugin-host`
  (crate `kanso-plugin-host`). The plugin host must sit beside `kanso`.
- Linux assets:
  - `assets/linux/kanso.desktop`: `Exec=kanso %U`, `Icon=kanso`, the `kanso://` handler in `MimeType`
  - `assets/icon/kanso.png`: 512 px
- `.github/workflows/release.yml`:
  - It runs on self-hosted runners, so forks can't run it; test locally ([section 6](#6-local-testing)).
  - Every action is pinned by commit SHA, and every new tool is pinned to an exact version.
- `scripts/linux-deps.sh` installs build dependencies. The pattern is: root runs apt-get, otherwise sudo, otherwise a clear error.
- The self-updater's `locate()` lives in `crates/kanso-update/src/install.rs`; the app side is in `crates/kanso-app/src/updates.rs`.
- The site (`site/`) is a Cloudflare Worker:
  - Redirects are answered in `site/src/lib/redirects.ts` before TanStack Start sees the request.
  - Static files live in `site/public/`.
  - CI tests the site but doesn't deploy it.
  - Worker static assets are capped at 25 MiB per file, so the `.deb`s can't be served from the site.

---

## 3. Design

**Every apt file lives on the GitHub Release. The site only redirects.**
That keeps large files off Cloudflare, needs no Cloudflare credentials in CI,
and means the site needs one code change and one deploy, ever.

```
https://kansoapp.app/apt/kanso.gpg                        → static file: site/public/apt/kanso.gpg
https://kansoapp.app/apt/dists/stable/InRelease           → 302 github.com/iamngoni/kanso/releases/latest/download/apt-stable-InRelease
https://kansoapp.app/apt/dists/stable/Release             → 302 …/latest/download/apt-stable-Release
https://kansoapp.app/apt/dists/stable/Release.gpg         → 302 …/latest/download/apt-stable-Release.gpg
https://kansoapp.app/apt/dists/stable/main/binary-amd64/Packages(.gz)  → 302 …/latest/download/apt-stable-main-binary-amd64-Packages(.gz)
https://kansoapp.app/apt/dists/stable/main/binary-arm64/Packages(.gz)  → 302 …/latest/download/apt-stable-main-binary-arm64-Packages(.gz)
https://kansoapp.app/apt/pool/<tag>/<file>.deb            → 302 github.com/iamngoni/kanso/releases/download/<tag>/<file>.deb
```

- **Asset names.** The path after `dists/` is flattened: its segments are joined with `-` and prefixed `apt-`. This is a pure function with unit tests (`site/src/lib/apt.ts`).
- **Pool paths.** Each `Packages` entry's `Filename:` is `pool/<tag>/<deb>`. It's produced by running `apt-ftparchive packages` over a temporary `pool/<tag>/` tree.
- **Stable means latest.** `releases/latest` excludes drafts and prereleases, which is what `stable` wants.
- **Integrity.** apt checks `InRelease` against the pinned key, and `Packages` pins each `.deb` by SHA-256, so a redirect can't substitute content.
- **Only the latest version** is in the index. Older versions aren't installable through apt.
- **Caching.** Redirects for `dists/*` are `cache-control: no-store`, so `apt update` sees a new release at once. Redirects for `pool/*` are cached for a day, since a tag's `.deb` never changes.
- **No open redirect.** Only the exact paths above redirect; the tag and file name are checked with regular expressions. Everything else under `/apt/` is a 404.

---

## 4. Tasks

### 4.1 The `.deb` (cargo-deb)

`[package.metadata.deb]` in `crates/kanso-app/Cargo.toml` packs:
- `target/release/kanso` and `kanso-plugin-host` into `/usr/bin`
- the desktop file into `/usr/share/applications`
- the icon into `/usr/share/icons/hicolor/512x512/apps`

Dependencies:
- `Depends: $auto, libvulkan1`. `$auto` includes the direct shared-library packages, including
  `libxkbcommon-x11-0`, `libfido2-1`, `libasound2` and `libdbus-1-3`, so apt installs them with Kanso.
  `ldd` confirms Vulkan is loaded at run time, so dpkg-shlibdeps can't find it.
- `Recommends: mesa-vulkan-drivers, gnome-keyring | kwalletmanager`. Kanso runs without a keyring, but doesn't stay signed in.

`scripts/deb-build.sh <version> <out-dir>` runs cargo-deb 3.8.0 with:
- **An explicit version.** A release's `0.9.0` becomes `0.9.0-1`. A build run by hand, `0.9.0-dev.abc1234`, becomes `0.9.0~dev.abc1234-1`, which sorts below the release.
- **A one-entry Debian changelog**, which lintian requires.

No maintainer scripts are needed: dpkg triggers update the desktop database and the icon cache. The `.desktop` file's `MimeType` registers `kanso://` system-wide.

In `release.yml`, the Linux builds run it in a `Package (Debian)` step after the tarball, so `dist/kanso_<debver>_{amd64,arm64}.deb` go into the existing upload.

### 4.2 Updater: apt installs leave updates to apt

`NotInstallable::PackageManaged` (`crates/kanso-update/src/install.rs`):
- **Message:** "Kanso was installed by your package manager. Update it with sudo apt upgrade."
- **When:** `locate()` returns it on Linux when the canonicalised path of the executable is under `/usr`. It comes after the dev and AppImage checks and before the writability probe.

The app (`crates/kanso-app/src/updates.rs`):
- still checks the feed and says a new version is out
- its button reads **Copy Command** and copies `sudo apt upgrade`, then says so in a notification; it never downloads or opens the release page
- the pill reads "Update available · apt upgrade"

Tests:
- `/usr/bin/kanso` on Linux is `PackageManaged`.
- `~/.local/bin/kanso` and `/usr/local/bin/kanso` behave as before.
- macOS and Windows are unaffected.

### 4.3 Publishing the apt index in the release

The `apt` job of `release.yml` (tags only, in the `release` environment):

1. Without `APT_SIGNING_KEY`, it logs `::notice::apt signing key not configured; skipping apt metadata` and publishes nothing. Merging before the key exists doesn't break releases.
2. Otherwise `scripts/apt-release.sh`:
   - imports the key into a temporary `GNUPGHOME` (loopback pinentry, passphrase from `APT_SIGNING_PASSPHRASE`)
   - checks the key is the one `packaging/linux/apt.conf` names
   - runs `scripts/apt-index.sh` for each suite there, failing clearly on any suite but `stable`
3. `scripts/apt-index.sh` builds the `pool/<tag>/` tree from the `.deb`s, then, in this order and touching nothing afterwards:
   1. runs `apt-ftparchive packages` per architecture and gzips the output
   2. runs `apt-ftparchive release` with `Origin=Kanso`, `Label=Kanso`, `Suite=stable`, `Codename=stable`, `Components=main`, `Architectures=amd64 arm64`
   3. signs `InRelease` (clearsigned) and `Release.gpg` (detached, armored)
   4. copies everything out under the flattened `apt-…` names
4. An `always()` step removes the GnuPG home.
5. `scripts/apt-tools.sh` installs `apt-utils` and `gnupg` the way `linux-deps.sh` does.
6. The files are uploaded as the `apt-metadata` artifact. The feed job's `SHA256SUMS` covers them, and the publish job checks and uploads them with everything else.

### 4.4 The site

- `site/public/apt/kanso.gpg` is written by the setup script; until then a README holds the folder.
- `site/public/_headers` serves the key as `application/pgp-keys`.
- `site/src/lib/apt.ts`, called from `redirectFor`, holds the whitelist:
  - **Tag:** `^v\d+\.\d+\.\d+$`
  - **File:** `^kanso_[0-9A-Za-z.~+-]+_(amd64|arm64)\.deb$`
  - Its tests (`apt.test.ts`) cover every index path, the pool, and a list of paths that must answer 404.
- The repository slug is the existing `REPOSITORY` constant.

### 4.5 `scripts/apt-setup.sh`: the maintainer's one-time setup

1. **Checks** that `gpg`, `gh`, `openssl` and `git` exist, `gh` is signed in, the user is an admin of `iamngoni/kanso`, and the script runs from the repository's root on a clean tree.
2. **Asks for:**
   - confirmation of the maintainer string, which is read from `Cargo.toml`
   - the key's expiry: `2y` (default), `5y` or `never`
   - the suites: only `stable`
   - a backup folder, which must be outside the repository
3. **Stops** if `APT_SIGNING_KEY` already exists, unless run with `--rotate`.
4. **Makes the key:** `gpg --quick-gen-key "Kanso APT Signing Key <email>" ed25519 sign <expiry>` in a temporary `GNUPGHOME`, with a passphrase from `openssl rand -base64 32`.
5. **Stores the secrets:** pipes the armored secret key straight into `gh secret set APT_SIGNING_KEY --env release`, and the passphrase into `APT_SIGNING_PASSPHRASE`. Neither is written anywhere else or echoed.
6. **Writes the public files:** `site/public/apt/kanso.gpg` (binary) and `packaging/linux/apt.conf` (`suites=` and `fingerprint=`).
7. **Backs up** the key and passphrase to the chosen folder, readable only by the owner, or warns loudly when no folder was given.
8. **Prints** the fingerprint, the `git add … && git commit` to run, the one site deploy, and that the next tag publishes the repository.

`.gitignore` ignores `kanso-apt-signing-key-BACKUP.*` in case a backup ever ends up in the repository.

### 4.6 Docs

- README › Download: the apt commands, the requirements, and the `install.sh --uninstall` note (`~/.local/bin` comes before `/usr/bin` in `PATH`).
- RELEASING.md: one-time setup step 5 (the apt key, its expiry and rotation), and "The apt repository".

---

## 5. Out of scope

- An `unstable` channel or dev builds in apt
- Older versions in the index
- RPM, Flatpak, AUR
- Changes to `install.sh`
- Deploying the site from CI

---

## 6. Local testing

Forks can't run `release.yml`, so everything was tested locally.

```bash
git submodule update --init
scripts/linux-deps.sh
cargo build --release --locked -p kanso-app -p kanso-plugin-host
cargo install cargo-deb --version 3.8.0 --locked
scripts/deb-build.sh 0.9.0 /tmp/debs
lintian /tmp/debs/kanso_*.deb
```

lintian leaves three warnings, all expected:
- `initial-upload-closes-no-bugs`: this isn't an upload to Debian.
- `no-manual-page` for `kanso` and `kanso-plugin-host`: a GUI app and its helper. `kanso --help` documents the commands.

End to end, without GitHub:

1. Make a throwaway key in a temporary `GNUPGHOME`.
2. Run `APT_SIGNING_FINGERPRINT=<fpr> APT_SIGNING_PASSPHRASE_FILE=<file> scripts/apt-index.sh /tmp/debs v0.9.0 <release-dir>`.
3. Serve the release folders behind a small HTTP shim that redirects exactly as `site/src/lib/apt.ts` does.
4. In a container: add the source with `signed-by` set to the throwaway key, then `apt update && apt install kanso`.
5. Index a 0.9.1 `.deb` as the new latest release, and check that `apt upgrade` installs it.
6. Change a `Packages` file after signing, and check that apt refuses it ("Hash Sum mismatch").

The repository's own checks:

```bash
PROPTEST_RNG_SEED=20261017 cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
(cd site && npm ci && npm test)
shellcheck scripts/apt-setup.sh scripts/apt-index.sh scripts/apt-release.sh scripts/apt-tools.sh scripts/deb-build.sh
```

---

## 7. Acceptance criteria

- [ ] A tagged release has `kanso_<ver>-1_amd64.deb` and `_arm64.deb`, covered by SHA256SUMS.
- [ ] Without `APT_SIGNING_KEY`, a release behaves as before, plus the `.deb`s, and logs a notice.
- [ ] With the key, the release also carries the signed `apt-stable-*` index, and `apt update && apt install kanso` works on Ubuntu 22.04/24.04 and Debian 12, amd64 and arm64.
- [ ] An apt-installed Kanso reports new versions and points to `sudo apt upgrade`; it never downloads or replaces itself. Tarball installs are unchanged.
- [ ] The site serves `/apt/kanso.gpg`, redirects only whitelisted `/apt/*` paths (unit tested), and answers 404 to everything else under `/apt/`.
- [ ] `scripts/apt-setup.sh` passes shellcheck, never writes the private key or passphrase into the repository, and won't replace an existing key without `--rotate`.
- [ ] Workspace tests, clippy (`-D warnings`) and site tests pass.
- [ ] New CI tools are pinned to exact versions, and any new actions are pinned by SHA.

---

## As built

Where the code differs from the plan above, and why:

- **A separate `apt` job, not a step in the publish job.** `publish` is the only job that can write to the repository, and it deliberately runs no repository code. `feed` holds the update-signing key. Signing in its own job keeps the apt key and the update key out of each other's reach, and keeps `publish` free of repository code. `feed` now needs `apt`: it downloads every artifact, so its `SHA256SUMS` covers the apt files. `publish` downloads `apt-metadata` when it was published.
- **Environment secrets.** `APT_SIGNING_KEY` and `APT_SIGNING_PASSPHRASE` are secrets of the `release` environment, not of the whole repository, so only `v*` tags reach them, as with `KANSO_UPDATE_SIGNING_KEY`.
- **`/usr/local` isn't package-managed.** A binary under `/usr` but not `/usr/local` is `PackageManaged`; `/usr/local` is the administrator's, so it keeps the old behaviour.
- **The `.deb` step never stops a release.** If dpkg-shlibdeps can't map a library to a package (libfido2 unpacked without root by `linux-deps.sh`, say), cargo-deb only warns and leaves every shared library out of `Depends`. `scripts/deb-build.sh` therefore refuses a `.deb` that doesn't depend on `libc6`, `libxkbcommon0`,
`libxkbcommon-x11-0` and `libdbus-1-3`. The step is `continue-on-error`, so the release then ships without `.deb`s and the apt job skips with a warning.
- **The backup is asked for first.** `apt-setup.sh` asks for the backup folder before it makes or stores anything, so a folder inside the repository is refused before any secret exists. The backup is two files, `kanso-apt-signing-key-BACKUP.asc` and `.passphrase`, both mode 600.
- **The minimum glibc follows the runner.** `$auto` takes library versions from the build machine. A `.deb` built on Ubuntu 26.04 needs glibc 2.43, while the release runners (Ubuntu 22.04) give 2.35. The local tests ran on 26.04, so the 22.04 and Debian 12 install checks wait for the first release built on the runners.
