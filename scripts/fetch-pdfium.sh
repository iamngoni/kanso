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

PDFIUM_BUILD=8152
REPO=bblanchon/pdfium-binaries

sha256_for() {
    case "$1" in
    mac-arm64) echo 30b0c3f55448840f0947b54bedbe35185bbaccbadd35b8cd960270d6fa6caf80 ;;
    mac-x64) echo acb975e637ffa1f41430205c62baacac70e34b60811e22fc0e0274dfcd601fcc ;;
    linux-x64) echo 384ab5c51882408a143a21f9c14dcd2caf1f75280ae41c554d0489b46c3dba9f ;;
    linux-arm64) echo c6ed1a91234aede0cfafdc09c401b83bd44ac69deafc4599fda75d511c36403e ;;
    win-x64) echo cee2b0e9990bc67c0721774932961b9dbf2766046c10eec925acd4cfba6fdfe3 ;;
    win-arm64) echo 8614b10c6a780acc3ae9fdd425d6b61f9d22ae84301b19e1ac82f573930c217c ;;
    ios-device-arm64) echo 720108deb507122b46ffc9478436790318a07fbeeb835275f140d99069ea2845 ;;
    ios-simulator-arm64) echo 4d5d0dfdf7465a249831c7c64ad40ea6c8d1fdad748b6db5ee581e18ebbf4884 ;;
    ios-simulator-x64) echo d21aa22f5be106b72fc6da8dffc5ef30496a4ebf24e6f57813c23d8cf26200bd ;;
    android-arm64) echo 97085db0665639f2c7d5453d3b982b16c6bbb450f4c5170a3cf7ec0df7575ab9 ;;
    android-arm) echo 026644bb11653491e89ea2f9af490b5d765127bdb1c94aadc27bb3652ed34898 ;;
    android-x64) echo 25436b6e5a761b92faae592edcf6c355a802860e842bd34c73cf9f519c9c3e43 ;;
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
