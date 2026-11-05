#!/bin/sh
# Refuses a macOS binary that links a libcrypto or libssl outside /usr/lib
# (issue #243). The library is encrypted with SQLCipher on CommonCrypto,
# which is part of macOS; a build that picked up a Homebrew OpenSSL (an
# OPENSSL_DIR left set, say) would only run on a Mac that has it. Used by
# bundle-macos.sh on every binary and library in the bundle, and tested by
# test-check-macos-crypto.sh. POSIX sh.
#
#   scripts/check-macos-crypto.sh <binary>...
#
# A file that isn't Mach-O (a script) has no links and passes.
set -eu

if [ "$#" -eq 0 ]; then
    echo "usage: check-macos-crypto.sh <binary>..." >&2
    exit 2
fi
failed=0
for binary in "$@"; do
    if [ ! -f "$binary" ]; then
        echo "check-macos-crypto: $binary is not a file" >&2
        failed=1
        continue
    fi
    # `otool -L` prints the file (and, for a universal binary, each
    # architecture) on lines ending in ':', then one linked library a line.
    bad=$(otool -L "$binary" 2>/dev/null | grep -v ':$' | awk '{print $1}' \
        | grep -E '(^|/)lib(crypto|ssl)[.-]' | grep -vE '^/usr/lib/' || true)
    if [ -n "$bad" ]; then
        echo "check-macos-crypto: $binary links a libcrypto/libssl outside /usr/lib (Homebrew OpenSSL?):" >&2
        echo "$bad" | sed 's/^/    /' >&2
        failed=1
    fi
done
exit "$failed"
