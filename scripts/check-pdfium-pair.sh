#!/bin/sh
# Fails unless the PDFium binary Kanso fetches and the `pdfium-render`
# wrapper it is built with are a pair (issue #43, step 15).
#
#   scripts/check-pdfium-pair.sh [root]       # root defaults to the repository
#
# `pdfium-render` declares one `pdfium_<build>` feature per PDFium API, and a
# wrapper built for one API against a binary of another can fail at runtime
# (a missing symbol, a changed struct). So:
#
#   - scripts/fetch-pdfium.sh pins PDFIUM_BUILD, and every platform it
#     fetches has a SHA-256;
#   - crates/kanso-pdf/Cargo.toml names exactly one `pdfium_<build>` feature
#     on pdfium-render, and it is that build;
#   - the wrapper is pinned with `=`, and Cargo.lock holds that version.
#
# The weekly bump (scripts/bump-pdfium.sh) opens a PR that fails here when no
# wrapper has a feature for the new build yet: a binary newer than any
# wrapper is not shipped.
set -eu

root=${1:-$(dirname "$0")/..}
fetch=$root/scripts/fetch-pdfium.sh
manifest=$root/crates/kanso-pdf/Cargo.toml
lock=$root/Cargo.lock

fail() {
    echo "check-pdfium-pair: $1" >&2
    exit 1
}

for f in "$fetch" "$manifest" "$lock"; do
    [ -f "$f" ] || fail "$f does not exist."
done

build=$(sed -n 's/^PDFIUM_BUILD=\([0-9][0-9]*\)$/\1/p' "$fetch")
[ -n "$build" ] || fail "scripts/fetch-pdfium.sh has no PDFIUM_BUILD=<number> line."
[ "$(printf '%s\n' "$build" | wc -l)" -eq 1 ] || fail "scripts/fetch-pdfium.sh has more than one PDFIUM_BUILD line."

for platform in mac-arm64 mac-x64 linux-x64 linux-arm64 win-x64 win-arm64; do
    sum=$(sed -n "s/^    $platform) echo \([0-9a-f]*\) ;;\$/\1/p" "$fetch")
    case "$sum" in
    '') fail "scripts/fetch-pdfium.sh has no checksum for $platform." ;;
    esac
    [ "${#sum}" -eq 64 ] || fail "the checksum for $platform is not a SHA-256 (64 hex digits)."
done

# The pdfium-render dependency line (its features are on one line).
dep=$(grep '^pdfium-render *=' "$manifest" || true)
[ -n "$dep" ] || fail "crates/kanso-pdf/Cargo.toml has no pdfium-render dependency."
features=$(printf '%s\n' "$dep" | grep -o '"pdfium_[0-9][0-9]*"' | tr -d '"' || true)
[ -n "$features" ] || fail "pdfium-render has no pdfium_<build> feature in crates/kanso-pdf/Cargo.toml."
[ "$(printf '%s\n' "$features" | wc -l)" -eq 1 ] || fail "pdfium-render names more than one pdfium_<build> feature: $(echo $features)."
if [ "$features" != "pdfium_$build" ]; then
    fail "the binary is PDFium build $build (scripts/fetch-pdfium.sh) but pdfium-render is built for ${features#pdfium_} ($features, crates/kanso-pdf/Cargo.toml). Use a pdfium-render release that has a pdfium_$build feature, or keep the binary at the wrapper's build."
fi

# Exact pin, and the lock agrees.
version=$(printf '%s\n' "$dep" | sed -n 's/.*version *= *"=\([0-9][0-9.]*\)".*/\1/p')
[ -n "$version" ] || fail "pdfium-render must be pinned with an exact version (version = \"=x.y.z\")."
locked=$(awk '/^name = "pdfium-render"$/ { getline; sub(/^version = "/, ""); sub(/"$/, ""); print; exit }' "$lock")
[ "$locked" = "$version" ] || fail "Cargo.toml pins pdfium-render =$version but Cargo.lock has ${locked:-none}."

echo "PDFium build $build and pdfium-render $version ($features) are a pair."
