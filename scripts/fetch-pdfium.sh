#!/bin/sh
# Fetches the PDFium library Kanso's PDF worker loads (issue #43), from the
# prebuilt, non-V8 archives of bblanchon/pdfium-binaries, and checks each
# against a pinned SHA-256 before it is unpacked.
#
#   scripts/fetch-pdfium.sh                 this machine's platform
#   scripts/fetch-pdfium.sh win-x64         another one (for packaging)
#   PDFIUM_DIR=/some/dir scripts/fetch-pdfium.sh
#
# The result is $PDFIUM_DIR/<platform>/ (default vendor/pdfium/<platform>/):
# the library (lib/libpdfium.dylib, bin/pdfium.dll or lib/libpdfium.so), the
# archive's LICENSE and its licenses/ folder, which ship in the app's notices.
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
    echo "fetch-pdfium: unknown platform $platform (mac-arm64, mac-x64, linux-x64, linux-arm64, win-x64, win-arm64)" >&2
    exit 1
}

dest=${PDFIUM_DIR:-vendor/pdfium}/$platform
if [ -f "$dest/BUILD" ] && [ "$(cat "$dest/BUILD")" = "$PDFIUM_BUILD $expected" ]; then
    echo "PDFium $PDFIUM_BUILD ($platform) is already in $dest."
    exit 0
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
archive="pdfium-$platform.tgz"
url="https://github.com/$REPO/releases/download/chromium/$PDFIUM_BUILD/$archive"
curl --fail --silent --show-error --location --output "$work/$archive" "$url"

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

# Kanso ships the build without V8: PDF JavaScript can't run in it.
if ! grep -q '^pdf_enable_v8 = false' "$work/unpacked/args.gn"; then
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
echo "$PDFIUM_BUILD $expected" >"$dest/BUILD"
echo "PDFium $PDFIUM_BUILD ($platform) is in $dest."
