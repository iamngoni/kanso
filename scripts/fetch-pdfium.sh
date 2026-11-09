#!/bin/sh
# Fetches the PDFium library Kanso's PDF worker loads (issue #43), from the
# prebuilt, non-V8 archives of bblanchon/pdfium-binaries, and checks each
# against a pinned SHA-256 before it is unpacked.
#
#   scripts/fetch-pdfium.sh                 this machine's platform
#   scripts/fetch-pdfium.sh win-x64         another one (for packaging)
#   scripts/fetch-pdfium.sh ios-device-arm64   a phone's (the app's build hook
#                                           runs this; ios-simulator-arm64,
#                                           ios-simulator-x64, android-arm64,
#                                           android-arm and android-x64 are
#                                           the others)
#   PDFIUM_DIR=/some/dir scripts/fetch-pdfium.sh
#
# The result is $PDFIUM_DIR/<platform>/ (default vendor/pdfium/<platform>/):
# the library (lib/libpdfium.dylib, bin/pdfium.dll or lib/libpdfium.so), the
# archive's LICENSE and its licenses/ folder, which ship in the app's notices.
#
# Only the non-V8 build is ever installed: PDF JavaScript runs in V8, and
# without it no PDF's script can run. The archive's name, its args.gn and the
# library itself are checked (scripts/check-pdfium-no-v8.sh), and a V8 build
# stops the script.
#
# PDFIUM_BUILD and the checksums below are a pair with the `pdfium-render`
# version and its `pdfium_<build>` feature in crates/kanso-pdf/Cargo.toml.
# Bump all of them together: a wrapper built for one PDFium API against a
# binary of another can fail at runtime.
set -eu

cd "$(dirname "$0")/.."

PDFIUM_BUILD=8138
REPO=bblanchon/pdfium-binaries

sha256_for() {
    case "$1" in
    mac-arm64) echo 21a1567b397b571f4f77f21ca91eac7b892774219c3c715a5127a20950ccb18b ;;
    mac-x64) echo 72662ac8eff7bf5381ef82ceebd0a0ad5bddb051b303981115114e6972fd201c ;;
    linux-x64) echo d6b1e28950a0392590d79bbdbb7eb19686872b688f8a36f170acb19e41473113 ;;
    linux-arm64) echo c70b1ce4d13257f741af19e0aeee575890875995eb81370a65be942e601a1862 ;;
    win-x64) echo 8d48973e3eb472dda0f5f2435e595ac043f9f2219dd71883392f549445afa8a5 ;;
    win-arm64) echo 5e04abdb36ba9b4b5c8630123eaf042036f8a62abda6e5529708fee3d35fbf4f ;;
    ios-device-arm64) echo d2acc4d9c24fa3bd002415e89c8a52a1936da823215283315dfab2070e465698 ;;
    ios-simulator-arm64) echo 7e2d50b1dbe1d4f9e4b28a575a0f69423b5f2150f30b3303455a39669dd70c61 ;;
    ios-simulator-x64) echo 5696940a4e3adf1e2bca4010d69ce500d3ffadc1e740bc4df957349fb8fdbd98 ;;
    android-arm64) echo fc73c8020d7204e52f7a9d8153489b3cac2e9c5dc5d70d3a37a976a7398c1a86 ;;
    android-arm) echo ee004a2ced1b321b7d9875f71ef9470692d00155b86262312b8c4c1b787c2124 ;;
    android-x64) echo cf71ceb527d72a64b3e8bf36e9b15b326984f56d33bb8b74350a92ab8706ba87 ;;
    *) return 1 ;;
    esac
}

platform=${1:-}
if [ -z "$platform" ]; then
    arch=$(uname -m)
    case "$(uname -s)-$arch" in
    Darwin-arm64) platform=mac-arm64 ;;
    Darwin-x86_64) platform=mac-x64 ;;
    Linux-x86_64) platform=linux-x64 ;;
    Linux-aarch64) platform=linux-arm64 ;;
    MINGW* | MSYS* | CYGWIN*) platform=win-x64 ;;
    *)
        echo "fetch-pdfium: no PDFium build for $(uname -s) $arch" >&2
        exit 1
        ;;
    esac
fi
expected=$(sha256_for "$platform") || {
    echo "fetch-pdfium: unknown platform $platform (mac-arm64, mac-x64, linux-x64, linux-arm64, win-x64, win-arm64, ios-device-arm64, ios-simulator-arm64, ios-simulator-x64, android-arm64, android-arm, android-x64)" >&2
    exit 1
}

dest=${PDFIUM_DIR:-vendor/pdfium}/$platform
# An iOS copy fetched before the minimum was restamped (below) is fetched again.
stale_ios_minimum() {
    case "$platform" in ios-*) ;; *) return 1 ;; esac
    for lib in "$dest"/lib/*.dylib; do
        [ -f "$lib" ] || return 0
        vtool -show-build "$lib" | grep -q 'minos 13.0' || return 0
    done
    return 1
}
if [ -f "$dest/BUILD" ] && [ -f "$dest/args.gn" ] && [ "$(cat "$dest/BUILD")" = "$PDFIUM_BUILD $expected" ] && ! stale_ios_minimum; then
    # Looked at again, so a library swapped in after the fetch is caught.
    scripts/check-pdfium-no-v8.sh "$dest" >/dev/null
    echo "PDFium $PDFIUM_BUILD ($platform) is already in $dest."
    exit 0
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
archive="pdfium-$platform.tgz"
# The V8 archives are named pdfium-v8-<platform>.tgz; that is never fetched.
case "$archive" in
*v8*)
    echo "fetch-pdfium: $archive is a V8 build; Kanso only ships the non-V8 build." >&2
    exit 1
    ;;
esac
url="https://github.com/$REPO/releases/download/chromium/$PDFIUM_BUILD/$archive"
curl --retry 5 --retry-all-errors --retry-delay 5 --fail --silent --show-error --location --output "$work/$archive" "$url"

if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$work/$archive" | cut -d' ' -f1)
else
    actual=$(shasum -a 256 "$work/$archive" | cut -d' ' -f1)
fi
if [ "$actual" != "$expected" ]; then
    echo "fetch-pdfium: $archive does not match its pinned checksum." >&2
    echo "  expected $expected" >&2
    echo "  got      $actual" >&2
    exit 1
fi

mkdir "$work/unpacked"
tar -xzf "$work/$archive" -C "$work/unpacked"

# Kanso ships the build without V8: PDF JavaScript can't run in it. The
# checksum above pins the exact archive; this is the second lock, on what is
# inside it.
if ! scripts/check-pdfium-no-v8.sh "$work/unpacked" >/dev/null; then
    echo "fetch-pdfium: $archive was built with V8; Kanso only ships the non-V8 build." >&2
    exit 1
fi

rm -rf "$dest"
mkdir -p "$dest"
[ -d "$work/unpacked/lib" ] && cp -R "$work/unpacked/lib" "$dest/lib"
[ -d "$work/unpacked/bin" ] && cp -R "$work/unpacked/bin" "$dest/bin"
cp "$work/unpacked/LICENSE" "$dest/LICENSE"
cp -R "$work/unpacked/licenses" "$dest/licenses"
cp "$work/unpacked/VERSION" "$dest/VERSION"
# The build's own record of how it was made, which the V8 check reads.
cp "$work/unpacked/args.gn" "$dest/args.gn"
# The iOS builds are stamped with the SDK they were built with (iOS 26) as
# their minimum, and App Store Connect refuses a framework whose binary asks
# for more than the Info.plist Flutter writes for it (ITMS-90208). PDFium only
# calls CoreGraphics, Foundation and libSystem functions far older than that,
# with its own C++ library inside, so the minimum is restamped to iOS 13, the
# lowest Flutter writes, which every plist Kanso ships is at or above.
case "$platform" in
ios-*)
    for lib in "$dest"/lib/*.dylib; do
        platform_name=ios
        case "$platform" in ios-simulator-*) platform_name=iossim ;; esac
        sdk=$(vtool -show-build "$lib" | awk '/sdk/ {print $2; exit}')
        vtool -set-build-version "$platform_name" 13.0 "${sdk:-26.0}" -replace -output "$lib.tmp" "$lib"
        mv "$lib.tmp" "$lib"
        codesign --force --sign - "$lib" 2>/dev/null || true
    done
    ;;
esac
echo "$PDFIUM_BUILD $expected" >"$dest/BUILD"
echo "PDFium $PDFIUM_BUILD ($platform) is in $dest."
