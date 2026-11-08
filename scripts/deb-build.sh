#!/usr/bin/env bash
# Makes Kanso's .deb from the binaries `cargo build --release -p kanso-app
# -p kanso-plugin-host -p kanso-pdf --features kanso-pdf/render` already made
# (docs/APT.md), with scripts/fetch-pdfium.sh's library in vendor/pdfium and
# scripts/fetch-onnxruntime.sh's in vendor/onnxruntime.
# Usage: scripts/deb-build.sh <version> <out-dir>
#
# <version> is the release's version, as the release workflow's Version step
# gives it: 0.9.0 for a tag, 0.9.0-dev.abc1234 for a run by hand. The .deb's
# version is that with Debian's revision: 0.9.0-1, and 0.9.0~dev.abc1234-1,
# whose ~ sorts it below 0.9.0-1.
#
# cargo-deb reads everything else from [package.metadata.deb] in
# crates/kanso-app/Cargo.toml. This writes the one-entry Debian changelog
# lintian asks for (the real notes are on the release page), then packs.
set -euo pipefail

if [ $# -ne 2 ]; then
	echo "Usage: $0 <version> <out-dir>" >&2
	exit 2
fi
version=$1
out=$2

if ! [[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$ ]]; then
	echo "$0: '$version' isn't a release version (x.y.z or x.y.z-pre)" >&2
	exit 2
fi
# A literal tilde from a variable: \~ in the replacement keeps the backslash
# before bash 5.2.
tilde='~'
deb_version="${version/-/$tilde}-1"
mkdir -p "$out"
out=$(cd "$out" && pwd)

cd "$(dirname "$0")/.."
for binary in kanso kanso-plugin-host kanso-pdf-worker; do
	if [ ! -x "target/release/$binary" ]; then
		echo "$0: target/release/$binary is missing; build it first:" >&2
		echo "  cargo build --release --locked -p kanso-app -p kanso-plugin-host -p kanso-pdf --features kanso-pdf/render" >&2
		exit 1
	fi
done

# The PDFium build for the machine the binaries were made for: PDFIUM_PLATFORM
# (the release workflow sets it per leg), or this machine's.
platform=${PDFIUM_PLATFORM:-}
if [ -z "$platform" ]; then
	case "$(uname -m)" in
	x86_64) platform=linux-x64 ;;
	aarch64 | arm64) platform=linux-arm64 ;;
	*)
		echo "$0: no PDFium build for $(uname -m); set PDFIUM_PLATFORM" >&2
		exit 1
		;;
	esac
fi
case "$platform" in
linux-x64 | linux-arm64) ;;
*)
	echo "$0: PDFIUM_PLATFORM '$platform' isn't linux-x64 or linux-arm64" >&2
	exit 2
	;;
esac
pdfium=vendor/pdfium/$platform
if [ ! -f "$pdfium/lib/libpdfium.so" ]; then
	echo "$0: the PDFium library is missing; fetch it first:" >&2
	echo "  scripts/fetch-pdfium.sh $platform" >&2
	exit 1
fi
# The library that goes in the package is the build without V8 (issue #43),
# for the same machine as the worker beside it.
scripts/check-pdfium-no-v8.sh "$pdfium/lib/libpdfium.so"
scripts/check-pdfium-arch.sh "$pdfium/lib/libpdfium.so" target/release/kanso-pdf-worker
# Voice input's ONNX Runtime, which the binaries were linked against
# (scripts/fetch-onnxruntime.sh; ORT_LIB_LOCATION when the build ran).
ort=vendor/onnxruntime/$platform
if [ ! -f "$ort/lib/libonnxruntime.so.1" ]; then
	echo "$0: the ONNX Runtime library is missing; fetch it first:" >&2
	echo "  scripts/fetch-onnxruntime.sh $platform" >&2
	exit 1
fi
scripts/check-pdfium-arch.sh "$ort/lib/libonnxruntime.so.1" target/release/kanso-pdf-worker
rm -rf target/deb/onnxruntime
mkdir -p target/deb/onnxruntime/lib
cp "$ort/lib/libonnxruntime.so.1" target/deb/onnxruntime/lib/libonnxruntime.so.1
cp "$ort/LICENSE" "$ort/ThirdPartyNotices.txt" target/deb/onnxruntime/
# cargo-deb's asset paths are fixed ([package.metadata.deb]); they point here.
rm -rf target/deb/pdfium
mkdir -p target/deb/pdfium/lib
cp "$pdfium/lib/libpdfium.so" target/deb/pdfium/lib/libpdfium.so
cp "$pdfium/LICENSE" target/deb/pdfium/LICENSE
cp -R "$pdfium/licenses" target/deb/pdfium/licenses

maintainer=$(sed -n 's/^maintainer = "\(.*\)"$/\1/p' crates/kanso-app/Cargo.toml | head -1)
if [ -z "$maintainer" ]; then
	echo "$0: no maintainer in [package.metadata.deb]" >&2
	exit 1
fi

# The changelog cargo-deb packs (`changelog` in [package.metadata.deb]).
mkdir -p target/deb
{
	echo "kanso ($deb_version) stable; urgency=medium"
	echo
	echo "  * Kanso $version: https://github.com/iamngoni/kanso/releases/tag/v$version"
	echo
	echo " -- $maintainer  $(LC_ALL=C date -u -R)"
} >target/deb/changelog

# dpkg-shlibdeps (cargo-deb's $auto) must find libonnxruntime.so.1, which the
# binary needs and the package carries, to read its other dependencies; the
# package that has it isn't installed here, so it is pointed at the copy.
export LD_LIBRARY_PATH="$PWD/target/deb/onnxruntime/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
deb=$(cargo deb -p kanso-app --no-build --deb-version "$deb_version" -o "$out" | tail -1)

# When dpkg-shlibdeps can't tell which package a library comes from (one
# unpacked outside dpkg, say), cargo-deb only warns and leaves the shared
# libraries out of Depends: a .deb that installs and doesn't run. Refuse it.
depends=$(dpkg-deb --field "$deb" Depends)
for library in libc6 libxkbcommon0 libdbus-1-3; do
	if ! [[ ", $depends" == *", $library "* || ", $depends" == *", $library,"* ]]; then
		echo "$0: $deb doesn't depend on $library, so dpkg-shlibdeps didn't run" >&2
		echo "  Depends: $depends" >&2
		rm -f -- "$deb"
		exit 1
	fi
done
echo "$deb"
