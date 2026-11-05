#!/bin/sh
# Fails if a PDFium library, or the folder it came in, is a V8 build
# (issue #43, step 9). PDF JavaScript runs in V8; Kanso ships the build
# without it, so no JavaScript in a PDF can ever run, whatever else goes
# wrong.
#
#   scripts/check-pdfium-no-v8.sh vendor/pdfium/mac-arm64
#   scripts/check-pdfium-no-v8.sh "Kanso.app/Contents/Frameworks/libpdfium.dylib"
#
# Each argument is a library file (libpdfium.dylib, libpdfium.so or
# pdfium.dll) or a folder holding one: an unpacked pdfium-binaries archive,
# or what scripts/fetch-pdfium.sh installed. A folder is checked three ways,
# a lone file by the last two:
#
#   1. args.gn, the build's own record, says `pdf_enable_v8 = false` and
#      does not enable XFA (which needs V8). A folder with an args.gn that
#      does not say so fails; one without an args.gn is judged by the rest.
#   2. The library exports none of V8's marks: PDFium's V8-only exports
#      (FPDF_GetRecommendedV8Flags and friends) and V8's own symbols.
#      Checked on the binary, so it holds even if args.gn is missing or
#      wrong. (A V8 build has both; the non-V8 build has neither.)
#   3. The library is under a size ceiling. V8 is statically linked, which
#      makes the library six times larger (46 MB against 7.7 MB on macOS).
#      PDFIUM_MAX_BYTES overrides the ceiling, for the tests.
set -eu

max_bytes=${PDFIUM_MAX_BYTES:-25000000}
found=$(mktemp)
trap 'rm -f "$found"' EXIT

fail() {
    echo "check-pdfium-no-v8: $1" >&2
    exit 1
}

check_library() {
    lib=$1
    size=$(wc -c <"$lib" | tr -d ' ')
    if [ "$size" -gt "$max_bytes" ]; then
        fail "$lib is $size bytes, over the $max_bytes a non-V8 PDFium takes: this looks like a V8 build."
    fi
    # Plain bytes, whatever the platform's locale: an export table, a symbol
    # table or a string table all hold these names as text.
    if LC_ALL=C grep -aq -e 'FPDF_GetRecommendedV8Flags' -e 'FPDF_GetArrayBufferAllocatorSharedInstance' "$lib"; then
        fail "$lib exports PDFium's V8 entry points: it is a V8 build."
    fi
    if LC_ALL=C grep -aq -e 'N2v88internal' -e 'v8::internal' -e 'v8::Isolate' "$lib"; then
        fail "$lib contains V8's own code: it is a V8 build."
    fi
}

check_args() {
    args=$1
    grep -q '^pdf_enable_v8 = false[[:space:]]*$' "$args" ||
        fail "$args does not say pdf_enable_v8 = false: this is not the non-V8 build."
    if grep -q '^pdf_enable_xfa = true' "$args"; then
        fail "$args enables XFA, which needs V8."
    fi
}

check_folder() {
    dir=$1
    # Every args.gn under the folder (one platform's, or all of
    # vendor/pdfium), one path per line whatever spaces they hold
    # ("Kanso Dev.app").
    find "$dir" -type f -name args.gn >"$found"
    while IFS= read -r file; do
        check_args "$file"
    done <"$found"
    find "$dir" -type f \( -name libpdfium.dylib -o -name libpdfium.so -o -name pdfium.dll \) >"$found"
    [ -s "$found" ] || fail "no PDFium library in $dir."
    while IFS= read -r file; do
        check_library "$file"
    done <"$found"
}

[ "$#" -gt 0 ] || {
    echo "usage: check-pdfium-no-v8.sh <folder-or-library>..." >&2
    exit 2
}
for target in "$@"; do
    if [ -d "$target" ]; then
        check_folder "$target"
    elif [ -f "$target" ]; then
        check_library "$target"
    else
        fail "$target does not exist."
    fi
    echo "$target is not a V8 build."
done
