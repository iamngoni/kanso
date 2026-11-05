#!/bin/sh
# Tests scripts/check-pdfium-no-v8.sh (issue #43, step 9) against stand-in
# folders and libraries, and, with --real-v8, against the real V8 archive
# the pdfium-binaries project publishes beside the one Kanso ships:
#
#   scripts/test-check-pdfium.sh              stand-ins only, no network
#   scripts/test-check-pdfium.sh --real-v8    also download the V8 archive
#                                             for this platform (about 13 MB)
#                                             and expect it to be refused
#
# The V8 archive is only read, never run, and needs no checksum: the test
# passes only if it is refused. It is taken from the build in
# scripts/fetch-pdfium.sh, so a bump of that build is tested without edits
# here. If the project ever changes what a V8 build contains so that the
# marks in check-pdfium-no-v8.sh miss it, this is where it shows.
set -eu

cd "$(dirname "$0")/.."
check=scripts/check-pdfium-no-v8.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failures=0

# expect_pass|expect_fail <name> <args...>
expect_pass() {
    name=$1
    shift
    if "$check" "$@" >/dev/null 2>&1; then
        echo "ok   $name"
    else
        echo "FAIL $name: refused, and should have passed" >&2
        failures=$((failures + 1))
    fi
}
expect_fail() {
    name=$1
    shift
    if "$check" "$@" >/dev/null 2>&1; then
        echo "FAIL $name: passed, and should have been refused" >&2
        failures=$((failures + 1))
    else
        echo "ok   $name"
    fi
}

# A library of ordinary-looking bytes.
plain_library() {
    mkdir -p "$(dirname "$1")"
    printf 'FPDF_InitLibrary\0FPDF_LoadMemDocument64\0FPDF_RenderPageBitmap\0' >"$1"
}
args_gn() {
    printf 'is_debug = false\npdf_enable_v8 = %s\npdf_enable_xfa = %s\ntarget_cpu = "arm64"\n' "$2" "$3" >"$1/args.gn"
}

# A folder that is what Kanso ships.
plain_library "$tmp/good/lib/libpdfium.dylib"
args_gn "$tmp/good" false false
expect_pass "the non-V8 folder" "$tmp/good"
expect_pass "its library alone" "$tmp/good/lib/libpdfium.dylib"

# More than one at once: every one is checked.
expect_fail "a good folder and a missing one" "$tmp/good" "$tmp/nope"

# args.gn says otherwise.
plain_library "$tmp/v8-args/lib/libpdfium.so"
args_gn "$tmp/v8-args" true true
expect_fail "args.gn with V8 on" "$tmp/v8-args"
plain_library "$tmp/xfa-args/lib/libpdfium.so"
args_gn "$tmp/xfa-args" false true
expect_fail "args.gn with XFA on" "$tmp/xfa-args"
plain_library "$tmp/silent-args/bin/pdfium.dll"
printf 'is_debug = false\n' >"$tmp/silent-args/args.gn"
expect_fail "args.gn that does not say" "$tmp/silent-args"
# ...even though the library inside looks fine: args.gn is the build's record.
expect_pass "the same library alone" "$tmp/silent-args/bin/pdfium.dll"

# The library says otherwise, whatever args.gn claims.
plain_library "$tmp/liar/lib/libpdfium.dylib"
args_gn "$tmp/liar" false false
printf 'x\0_FPDF_GetRecommendedV8Flags\0y' >>"$tmp/liar/lib/libpdfium.dylib"
expect_fail "a V8 export under an honest-looking args.gn" "$tmp/liar"
plain_library "$tmp/symbols/lib/libpdfium.so"
printf '_ZN2v88internal7IsolateE' >>"$tmp/symbols/lib/libpdfium.so"
expect_fail "V8's own symbols" "$tmp/symbols/lib/libpdfium.so"
plain_library "$tmp/strings/bin/pdfium.dll"
printf 'v8::Isolate::New' >>"$tmp/strings/bin/pdfium.dll"
expect_fail "V8's own strings" "$tmp/strings/bin/pdfium.dll"
plain_library "$tmp/alloc/lib/libpdfium.so"
printf 'FPDF_GetArrayBufferAllocatorSharedInstance' >>"$tmp/alloc/lib/libpdfium.so"
expect_fail "V8's allocator export" "$tmp/alloc/lib/libpdfium.so"

# Too big to be the non-V8 build.
# (Exported and unset by hand: a variable set in front of a shell function
# call can outlive it.)
export PDFIUM_MAX_BYTES=10
expect_fail "a library over the size ceiling" "$tmp/good/lib/libpdfium.dylib"
unset PDFIUM_MAX_BYTES

# Nothing to check.
mkdir -p "$tmp/empty"
expect_fail "a folder with no library" "$tmp/empty"
expect_fail "a path that is not there" "$tmp/nope"
expect_fail "no arguments"

# A path with a space in it ("Kanso Dev.app").
plain_library "$tmp/Kanso Dev.app/Contents/Frameworks/libpdfium.dylib"
expect_pass "a folder with a space in its name" "$tmp/Kanso Dev.app"
printf '_FPDF_GetRecommendedV8Flags' >>"$tmp/Kanso Dev.app/Contents/Frameworks/libpdfium.dylib"
expect_fail "a V8 library under a folder with a space" "$tmp/Kanso Dev.app"

if [ "${1:-}" = "--real-v8" ]; then
    build=$(sed -n 's/^PDFIUM_BUILD=//p' scripts/fetch-pdfium.sh)
    case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) platform=mac-arm64 ;;
    Darwin-x86_64) platform=mac-x64 ;;
    Linux-x86_64) platform=linux-x64 ;;
    Linux-aarch64) platform=linux-arm64 ;;
    *) platform=win-x64 ;;
    esac
    url="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/$build/pdfium-v8-$platform.tgz"
    mkdir "$tmp/real"
    curl --fail --silent --show-error --location --output "$tmp/real.tgz" "$url"
    tar -xzf "$tmp/real.tgz" -C "$tmp/real"
    expect_fail "the real V8 archive ($platform, build $build)" "$tmp/real"
    # And the library on its own, with the size ceiling lifted and args.gn
    # gone: the marks in the binary alone must catch it.
    rm "$tmp/real/args.gn"
    export PDFIUM_MAX_BYTES=999999999
    expect_fail "the real V8 library by its marks alone" "$tmp/real"
    unset PDFIUM_MAX_BYTES
fi

if [ "$failures" -gt 0 ]; then
    echo "$failures check(s) failed." >&2
    exit 1
fi
echo "All checks passed."
