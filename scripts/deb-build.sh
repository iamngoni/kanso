#!/usr/bin/env bash
# Makes Kanso's .deb from the binaries `cargo build --release -p kanso-app
# -p kanso-plugin-host -p kanso-pdf --features kanso-pdf/render` already made
# (docs/APT.md), with scripts/fetch-pdfium.sh's library in vendor/pdfium.
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

if [ ! -f vendor/pdfium/linux-x64/lib/libpdfium.so ]; then
	echo "$0: the PDFium library is missing; fetch it first:" >&2
	echo "  scripts/fetch-pdfium.sh linux-x64" >&2
	exit 1
fi

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
