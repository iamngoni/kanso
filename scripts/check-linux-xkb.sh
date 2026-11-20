#!/bin/sh
# Fails unless the XKB/XCB bridge bundled with a Linux GUI archive can find
# its own dependency beside it.
#
# The GUI has DT_NEEDED libxkbcommon-x11.so.0. That library in turn needs
# libxcb-xkb.so.1, and GNU ld resolves a library's dependencies using its own
# RUNPATH, not the executable's. The archive therefore patches
# libxkbcommon-x11.so.0 to have RUNPATH $ORIGIN and carries libxcb-xkb.so.1
# in the same folder. Without that patch, a host with libxkbcommon-x11
# installed still fails at startup with libxcb-xkb.so.1 missing.
#
#   scripts/check-linux-xkb.sh <kanso> <libxkbcommon-x11.so.0> <libxcb-xkb.so.1>
set -eu

fail() {
    echo "check-linux-xkb: $1" >&2
    exit 1
}

[ "$#" -eq 3 ] || {
    echo "usage: check-linux-xkb.sh <kanso> <libxkbcommon-x11.so.0> <libxcb-xkb.so.1>" >&2
    exit 2
}

binary=$1
xkbcommon_x11=$2
xcb_xkb=$3
for file in "$binary" "$xkbcommon_x11" "$xcb_xkb"; do
    [ -f "$file" ] || fail "$file does not exist."
done

command -v objdump >/dev/null 2>&1 || fail "objdump is required."
command -v patchelf >/dev/null 2>&1 || fail "patchelf is required."
command -v ldd >/dev/null 2>&1 || fail "ldd is required."

machine() {
    magic=$(od -An -tx1 -N4 "$1" | tr -d ' \n')
    [ "$magic" = "7f454c46" ] || fail "$1 is not an ELF file."
    od -An -tx1 -j18 -N2 "$1" | tr -d ' \n'
}

binary_machine=$(machine "$binary")
for library in "$xkbcommon_x11" "$xcb_xkb"; do
    [ "$(machine "$library")" = "$binary_machine" ] ||
        fail "$library is for ELF machine $(machine "$library"), but $binary is $binary_machine."
done

objdump -p "$binary" | grep -q 'NEEDED.*libxkbcommon-x11\.so\.0' ||
    fail "$binary does not link libxkbcommon-x11.so.0; update this check if the GUI dropped X11."
objdump -p "$xkbcommon_x11" | grep -q 'NEEDED.*libxcb-xkb\.so\.1' ||
    fail "$xkbcommon_x11 does not need libxcb-xkb.so.1; update this check if the library changed."

xkb_dir=$(cd "$(dirname "$xkbcommon_x11")" && pwd -P)
xcb_dir=$(cd "$(dirname "$xcb_xkb")" && pwd -P)
[ "$xkb_dir" = "$xcb_dir" ] ||
    fail "$xkbcommon_x11 and $xcb_xkb are in different folders, so \$ORIGIN cannot find the dependency."

rpath=$(patchelf --print-rpath "$xkbcommon_x11")
# shellcheck disable=SC2016 # $ORIGIN is the literal dynamic-loader token.
[ "$rpath" = '$ORIGIN' ] ||
    fail "$xkbcommon_x11 has RUNPATH '$rpath', not exactly \$ORIGIN: it will not find libxcb-xkb.so.1 beside it."

ldd_status=0
ldd_output=$( (unset LD_LIBRARY_PATH LD_PRELOAD; ldd "$xkbcommon_x11") 2>&1 ) || ldd_status=$?
[ "$ldd_status" -eq 0 ] ||
    fail "ldd could not load $xkbcommon_x11 with its bundled dependency: $ldd_output"
resolved=$(printf '%s\n' "$ldd_output" | awk '$1 == "libxcb-xkb.so.1" && $2 == "=>" { print $3 }')
[ -n "$resolved" ] || fail "ldd did not report libxcb-xkb.so.1 for $xkbcommon_x11."
[ "$resolved" != "not" ] || fail "ldd could not find libxcb-xkb.so.1 beside $xkbcommon_x11."
resolved_dir=$(cd "$(dirname "$resolved")" 2>/dev/null && pwd -P) || fail "ldd resolved libxcb-xkb.so.1 to a missing path: $resolved"
[ "$resolved_dir" = "$xkb_dir" ] ||
    fail "ldd resolved libxcb-xkb.so.1 to $resolved, not beside $xkbcommon_x11."
[ "$(basename "$resolved")" = "libxcb-xkb.so.1" ] ||
    fail "ldd resolved libxcb-xkb.so.1 to $resolved, not the bundled file."

echo "check-linux-xkb: $xkbcommon_x11 has exact RUNPATH \$ORIGIN and ldd resolves libxcb-xkb.so.1 beside it"
