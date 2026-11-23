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

PDFIUM_BUILD=8165
REPO=bblanchon/pdfium-binaries

sha256_for() {
    case "$1" in
    mac-arm64) echo 112e74d717f3e0310273b70f3c8780e8a61ff11b34834124465c8305c841d350 ;;
    mac-x64) echo 07285915b564e46a432a9909dbc1f15892885ca6489e3faeefc70dee77ed371a ;;
    linux-x64) echo ba7865e16c0271daab75256790be06c26272190afe6e2a70eae0bb39bb99089c ;;
    linux-arm64) echo 338f5dcc907a20331812ae8ab01b16ff18966f4a247d790c8ad8fb49029d768b ;;
    win-x64) echo ca7338ae7779b8b1b82f4580305c4d762d214f8df1d54b47647fe3cc75989ded ;;
    win-arm64) echo d65e405c59cffe6fe36c0935b90d40ddf3ce75ef284f3168a28dcf00d0388131 ;;
    ios-device-arm64) echo 77329e61107846d7f3e68d458a28a65a92cf95fc449c86c9006f215e343800c8 ;;
    ios-simulator-arm64) echo 8bda624984d9e9b2e6a3aa83307b25807151aab1a2e801f11b35bef868986168 ;;
    ios-simulator-x64) echo d82be3b3233d73445a560851a6db01d4763a847dba580d0c7f7204a1ce5ffeb5 ;;
    android-arm64) echo ce4f17fea8881cdf163950748ab90854cf33f296309fab68de9441a1d41ff6cc ;;
    android-arm) echo 3698a4fcf0349d78be2bc04b90d828f25b3ea2154d1ada2ed6c528edc2797a10 ;;
    android-x64) echo e5d8646165eff2f837e5d8fc356eda6b7e624cbfad5915e2500428e244a89b44 ;;
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
