#!/usr/bin/env bash
# Fails when a Linux binary or library needs more than Kanso promises
# (Ubuntu 22.04 and Debian 12: glibc 2.35, libstdc++ GLIBCXX_3.4.30), or when
# the ONNX Runtime library a binary needs isn't found where it will be run.
#
#   scripts/check-linux-portable.sh <file>...
#
# Every file is checked for the newest GLIBC_, GLIBCXX_ and CXXABI_ version
# it uses. A binary that needs libonnxruntime.so.1 (scripts/fetch-onnxruntime.sh,
# found through the rpath crates/kanso-app/build.rs sets) must also resolve it
# with ldd from where it stands, so run this on the staged archive folder.
# GLIBC_MAX, GLIBCXX_MAX and CXXABI_MAX override the limits.
set -euo pipefail

glibc_max=${GLIBC_MAX:-2.35}
glibcxx_max=${GLIBCXX_MAX:-3.4.30}
cxxabi_max=${CXXABI_MAX:-1.3.13}

if [ $# -eq 0 ]; then
	echo "usage: check-linux-portable.sh <file>..." >&2
	exit 2
fi
command -v objdump >/dev/null 2>&1 || {
	echo "check-linux-portable: objdump is required" >&2
	exit 2
}

# newest <prefix> <file>: the highest PREFIX_x.y.z the file uses from other
# libraries (undefined symbols), or nothing.
newest() {
	objdump -T "$2" | grep -F '*UND*' | grep -o "$1_[0-9][0-9.]*" | sed "s/^$1_//" | sort -uV | tail -n 1 || true
}

# too_new <version> <max>: true when version is above max.
too_new() {
	[ -n "$1" ] && [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | tail -n 1)" != "$2" ]
}

status=0
for file in "$@"; do
	[ -f "$file" ] || {
		echo "check-linux-portable: $file does not exist" >&2
		exit 2
	}
	for pair in "GLIBC:$glibc_max" "GLIBCXX:$glibcxx_max" "CXXABI:$cxxabi_max"; do
		prefix=${pair%%:*}
		max=${pair#*:}
		found=$(newest "$prefix" "$file")
		if too_new "$found" "$max"; then
			echo "check-linux-portable: $file needs ${prefix}_$found; the most Kanso supports is ${prefix}_$max." >&2
			status=1
		fi
	done
	if objdump -p "$file" | grep -q 'NEEDED.*libonnxruntime\.so\.1'; then
		resolved=$(ldd "$file" | grep 'libonnxruntime\.so\.1' || true)
		case "$resolved" in
		*"=> /"*) ;;
		*)
			echo "check-linux-portable: $file needs libonnxruntime.so.1 and ldd can't find it beside it (${resolved:-not listed})." >&2
			status=1
			;;
		esac
	fi
done
[ "$status" -eq 0 ] && echo "check-linux-portable: $# file(s) need no more than glibc $glibc_max and libstdc++ $glibcxx_max"
exit "$status"
