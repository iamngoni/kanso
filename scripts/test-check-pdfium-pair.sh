#!/bin/sh
# Tests scripts/check-pdfium-pair.sh (issue #43, step 15) against copies of
# the real files with one thing broken at a time. No network.
#
#   scripts/test-check-pdfium-pair.sh
set -eu

cd "$(dirname "$0")/.."
check=scripts/check-pdfium-pair.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failures=0

# fresh <name>: a copy of the three files the check reads.
fresh() {
    rm -rf "$tmp/$1"
    mkdir -p "$tmp/$1/scripts" "$tmp/$1/crates/kanso-pdf"
    cp scripts/fetch-pdfium.sh "$tmp/$1/scripts/"
    cp crates/kanso-pdf/Cargo.toml "$tmp/$1/crates/kanso-pdf/"
    cp Cargo.lock "$tmp/$1/"
}
expect_pass() {
    if "$check" "$tmp/$1" >/dev/null 2>&1; then
        echo "ok   $2"
    else
        echo "FAIL $2: refused, and should have passed" >&2
        failures=$((failures + 1))
    fi
}
# expect_fail <dir> <name> <text the message must contain>
expect_fail() {
    if out=$("$check" "$tmp/$1" 2>&1); then
        echo "FAIL $2: passed, and should have been refused" >&2
        failures=$((failures + 1))
    elif printf '%s' "$out" | grep -q -- "$3"; then
        echo "ok   $2"
    else
        echo "FAIL $2: refused, but not for the expected reason ($3): $out" >&2
        failures=$((failures + 1))
    fi
}
# edit <dir> <file under it> <sed expression>
edit() {
    sed "$3" "$tmp/$1/$2" >"$tmp/$1/$2.new"
    mv "$tmp/$1/$2.new" "$tmp/$1/$2"
}

build=$(sed -n 's/^PDFIUM_BUILD=\([0-9]*\)$/\1/p' scripts/fetch-pdfium.sh)

fresh ok
expect_pass ok "the checked-in pair"

# A new binary against the old wrapper: the case the weekly bump can produce.
fresh newer-binary
edit newer-binary scripts/fetch-pdfium.sh "s/^PDFIUM_BUILD=.*/PDFIUM_BUILD=$((build + 257))/"
expect_fail newer-binary "a newer binary than the wrapper's feature" "built for $build"

# A new wrapper feature against the old binary.
fresh newer-wrapper
edit newer-wrapper crates/kanso-pdf/Cargo.toml "s/\"pdfium_$build\"/\"pdfium_$((build + 257))\"/"
expect_fail newer-wrapper "a newer wrapper feature than the binary" "PDFium build $build"

fresh two-features
edit two-features crates/kanso-pdf/Cargo.toml "s/\"pdfium_$build\"/\"pdfium_$build\", \"pdfium_7763\"/"
expect_fail two-features "two pdfium_<build> features" "more than one"

fresh no-feature
edit no-feature crates/kanso-pdf/Cargo.toml "s/\"pdfium_$build\", //"
expect_fail no-feature "no pdfium_<build> feature" "no pdfium_<build> feature"

fresh no-build
edit no-build scripts/fetch-pdfium.sh "s/^PDFIUM_BUILD=.*/PDFIUM_BUILD=latest/"
expect_fail no-build "a build that is not a number" "no PDFIUM_BUILD"

fresh no-sum
edit no-sum scripts/fetch-pdfium.sh "/^    win-arm64) echo/d"
expect_fail no-sum "a platform without a checksum" "no checksum for win-arm64"

fresh short-sum
edit short-sum scripts/fetch-pdfium.sh "s/^\(    mac-x64) echo [0-9a-f]\{60\}\)....  *;;/\1 ;;/"
expect_fail short-sum "a checksum that is too short" "mac-x64"

fresh loose-pin
edit loose-pin crates/kanso-pdf/Cargo.toml 's/version = "=0.9.4"/version = "0.9.4"/'
expect_fail loose-pin "a wrapper that is not pinned with =" "exact version"

fresh stale-lock
edit stale-lock Cargo.lock '/^name = "pdfium-render"$/{n;s/version = ".*"/version = "0.0.1"/;}'
expect_fail stale-lock "a lock file at another wrapper version" "Cargo.lock has 0.0.1"

if [ "$failures" -gt 0 ]; then
    echo "$failures check(s) failed." >&2
    exit 1
fi
echo "All checks passed."
