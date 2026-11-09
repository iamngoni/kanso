#!/bin/sh
# Fetches the ONNX Runtime library Kanso's Linux release builds link
# dynamically (voice input, kanso-voice), from Microsoft's own release
# archives, and checks each against a pinned SHA-256 before it is unpacked.
#
#   scripts/fetch-onnxruntime.sh                 this machine's platform
#   scripts/fetch-onnxruntime.sh linux-arm64     the other one (for packaging)
#   ORT_DIR=/some/dir scripts/fetch-onnxruntime.sh
#
# Why: the `ort` crate's download-binaries feature links pyke's static
# archives, which are built against glibc 2.38 and a newer libstdc++ than
# Ubuntu 22.04 and Debian 12 have, so a release built on ubuntu-22.04 (the
# glibc 2.35 floor Kanso promises) fails to link. Microsoft's Linux archives
# are built for old glibc. macOS and Windows keep the static archives.
#
# The result is $ORT_DIR/<platform>/ (default vendor/onnxruntime/<platform>/):
#   lib/libonnxruntime.so.1   the library, under its SONAME
#   lib/libonnxruntime.so     a symlink to it, which the linker finds
#   LICENSE, ThirdPartyNotices.txt, VERSION
#
# Build against it with
#   ORT_LIB_LOCATION=$PWD/vendor/onnxruntime/<platform>/lib ORT_PREFER_DYNAMIC_LINK=1
# (the release workflow sets both). The binaries find the library at run time
# through the rpath crates/kanso-app/build.rs adds.
#
# ORT_VERSION must be one the `ort` crate's `api-<n>` feature in
# crates/kanso-voice/Cargo.toml can talk to (the library may be newer than
# the API, never older); the pyke archives ort 2.0.0-rc.15 pins are 1.31.0.
set -eu

cd "$(dirname "$0")/.."

ORT_VERSION=1.31.0
REPO=microsoft/onnxruntime

sha256_for() {
    case "$1" in
    linux-x64) echo cc5c72baf5ae5c8238a6841f0897227be2d02826b9cf98eaf02fdefaeeb03a57 ;;
    linux-arm64) echo c5b8b3cca31f3d643a3b313b8f42e2d0f16a4fcefd02cda343d97bd5306afaab ;;
    *) return 1 ;;
    esac
}

platform=${1:-}
if [ -z "$platform" ]; then
    arch=$(uname -m)
    case "$(uname -s)-$arch" in
    Linux-x86_64) platform=linux-x64 ;;
    Linux-aarch64) platform=linux-arm64 ;;
    *)
        echo "fetch-onnxruntime: no Microsoft Linux build for $(uname -s) $arch" >&2
        exit 1
        ;;
    esac
fi
expected=$(sha256_for "$platform") || {
    echo "fetch-onnxruntime: unknown platform $platform (linux-x64, linux-arm64)" >&2
    exit 1
}
case "$platform" in
linux-x64) name=linux-x64 ;;
linux-arm64) name=linux-aarch64 ;;
esac

dest=${ORT_DIR:-vendor/onnxruntime}/$platform
if [ -f "$dest/BUILD" ] && [ -f "$dest/lib/libonnxruntime.so.1" ] && [ "$(cat "$dest/BUILD")" = "$ORT_VERSION $expected" ]; then
    echo "ONNX Runtime $ORT_VERSION ($platform) is already in $dest."
    exit 0
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
archive="onnxruntime-$name-$ORT_VERSION.tgz"
url="https://github.com/$REPO/releases/download/v$ORT_VERSION/$archive"
curl --retry 5 --retry-all-errors --retry-delay 5 --fail --silent --show-error --location --output "$work/$archive" "$url"

if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$work/$archive" | cut -d' ' -f1)
else
    actual=$(shasum -a 256 "$work/$archive" | cut -d' ' -f1)
fi
if [ "$actual" != "$expected" ]; then
    echo "fetch-onnxruntime: $archive does not match its pinned checksum." >&2
    echo "  expected $expected" >&2
    echo "  got      $actual" >&2
    exit 1
fi

mkdir "$work/unpacked"
tar -xzf "$work/$archive" -C "$work/unpacked" --strip-components=1

rm -rf "$dest"
mkdir -p "$dest/lib"
# The real file goes in under the SONAME, so a copy of it anywhere (a
# tarball, a .deb, ~/.local/lib/kanso) is found by the loader as it is.
cp -L "$work/unpacked/lib/libonnxruntime.so.$ORT_VERSION" "$dest/lib/libonnxruntime.so.1"
ln -s libonnxruntime.so.1 "$dest/lib/libonnxruntime.so"
cp "$work/unpacked/LICENSE" "$dest/LICENSE"
cp "$work/unpacked/ThirdPartyNotices.txt" "$dest/ThirdPartyNotices.txt"
echo "$ORT_VERSION" >"$dest/VERSION"
echo "$ORT_VERSION $expected" >"$dest/BUILD"
echo "ONNX Runtime $ORT_VERSION ($platform) is in $dest."
