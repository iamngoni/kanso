#!/bin/sh
# Fails unless a Linux PDFium library is built for the same machine as the
# worker that loads it (issue #43). A release leg that packaged another
# architecture's library would ship a reader that can never start: the
# worker can't load it, and refuses to run without it.
#
#   scripts/check-pdfium-arch.sh <libpdfium.so> <kanso-pdf-worker>
#
# Compares the ELF header's machine field (x86-64 is 3e 00, AArch64 b7 00).
set -eu

fail() {
    echo "check-pdfium-arch: $1" >&2
    exit 1
}

[ "$#" -eq 2 ] || {
    echo "usage: check-pdfium-arch.sh <libpdfium.so> <kanso-pdf-worker>" >&2
    exit 2
}

machine() {
    [ -f "$1" ] || fail "$1 does not exist."
    magic=$(od -An -tx1 -N4 "$1" | tr -d ' \n')
    [ "$magic" = "7f454c46" ] || fail "$1 is not an ELF file."
    od -An -tx1 -j18 -N2 "$1" | tr -d ' \n'
}

library=$(machine "$1")
worker=$(machine "$2")
[ "$library" = "$worker" ] ||
    fail "$1 is for ELF machine $library and $2 for $worker: this is another architecture's PDFium."
echo "$1 matches $2 (ELF machine $library)."
