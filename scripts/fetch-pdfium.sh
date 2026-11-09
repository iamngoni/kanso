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

PDFIUM_BUILD=7881
REPO=bblanchon/pdfium-binaries

sha256_for() {
    case "$1" in
    mac-arm64) echo 52e94ca5aa8847934330daf3f8150c190682c5ca93831468794f8b90d4392e40 ;;
    mac-x64) echo 6dedf83990e0e3d6b7c93c9e7589c5a126b0ae14b7464d76120cff7a26afb18b ;;
    linux-x64) echo 1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d ;;
    linux-arm64) echo ee7f7b7d5468958336a818c1cd580bdd20972846b7377b13f9a923d92d1d4674 ;;
    win-x64) echo 73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08 ;;
    win-arm64) echo d3035d4d2cacac6ecd1a2ece197a3d702a1b2a58466276b9f870b8cb278a9d84 ;;
    ios-device-arm64) echo 0f73152a94bbfb467dadedce745691e55b09792ed38cc9a6b0d11f2a9dc95ea7 ;;
    ios-simulator-arm64) echo fcccdae4c81b39234398c102645490705da150d77bec43f75ed60738c2f50cd9 ;;
    ios-simulator-x64) echo 7186c9ac94d4b25421b9cffd7716ac02e4153c8f27f955c831a98aab2ec107cd ;;
    android-arm64) echo 16d23bb86c4188d59326dc509938f59df3c417ecfdd3d2ca2f160dc5bd49a839 ;;
    android-arm) echo 57f7f5c349a3268f39b1737f830e50e271aa3116417abed2c30d0c0840b1b58b ;;
    android-x64) echo 58bd708d6083173008c63860dbffa93396a2fee00dd76747cbd782bcf4ff9bd4 ;;
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
