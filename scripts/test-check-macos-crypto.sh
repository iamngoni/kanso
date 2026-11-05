#!/bin/sh
# Tests scripts/check-macos-crypto.sh (issue #243) against binaries built
# here: stand-in libcrypto/libssl dylibs with the install names a Homebrew
# OpenSSL, an @rpath OpenSSL and the system's would have, and programs
# linked to each. Needs a Mac with the command-line tools; no network, no
# Homebrew.
#
#   scripts/test-check-macos-crypto.sh [<real binary>...]
#
# Real binaries given (a built `kanso`, say) are expected to pass too.
set -eu

cd "$(dirname "$0")/.."
check=scripts/check-macos-crypto.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failures=0

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

# A stand-in library named <install name>, and a program linked to it.
printf 'int stand_in(void) { return 1; }\n' >"$tmp/lib.c"
printf 'int stand_in(void);\nint main(void) { return stand_in(); }\n' >"$tmp/main.c"
linked_to() { # <out> <install name>
    lib="$tmp/$(basename "$2")"
    cc -dynamiclib -o "$lib" -install_name "$2" "$tmp/lib.c"
    cc -o "$1" "$tmp/main.c" "$lib"
}

cc -o "$tmp/plain" "$tmp/main.c" "$tmp/lib.c"
expect_pass "a program that links no OpenSSL" "$tmp/plain"

linked_to "$tmp/system" /usr/lib/libcrypto.46.dylib
expect_pass "a program that links the system's libcrypto" "$tmp/system"

linked_to "$tmp/homebrew" /opt/homebrew/opt/openssl@3/lib/libcrypto.3.dylib
expect_fail "a program that links Homebrew's libcrypto" "$tmp/homebrew"

linked_to "$tmp/rpath" @rpath/libssl.3.dylib
expect_fail "a program that links an @rpath libssl" "$tmp/rpath"

expect_fail "a good program and a bad one" "$tmp/plain" "$tmp/homebrew"
expect_fail "a file that isn't there" "$tmp/nope"
printf '#!/bin/sh\n' >"$tmp/script"
expect_pass "a script (not Mach-O)" "$tmp/script"

for real in "$@"; do
    expect_pass "the real binary $real" "$real"
done

if [ "$failures" -gt 0 ]; then
    echo "test-check-macos-crypto: $failures failed" >&2
    exit 1
fi
echo "test-check-macos-crypto: ok"
