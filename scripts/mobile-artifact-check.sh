#!/usr/bin/env bash
# Looks inside a built mobile app for what the native side finds by name (#48).
# Usage: scripts/mobile-artifact-check.sh ios <Runner.app>
#        scripts/mobile-artifact-check.sh android <app-release.apk>...
#
# iOS: Runner.app/Frameworks/kanso_mobile_bridge.framework holds the library
#   (arm64 only), and it must export the generated bridge's dispatcher and the
#   C entry the background task calls.
# Android: every ABI's lib/<abi>/libkanso_mobile_bridge.so must export the same
#   two and the JNI function Kotlin's KansoNative.init resolves to. The classes
#   the library calls by name (KansoNative, KansoSecrets and the certificate
#   verifier's Kotlin part) must survive R8's shrinking, which proguard-rules.pro
#   is meant to ensure; the release build is the only thing that runs it.
#
# Android needs the SDK's `apkanalyzer` (cmdline-tools), Java, and the NDK's
# llvm-nm (scripts/mobile/env.sh finds it).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=mobile/env.sh
. "$here/mobile/env.sh"

platform="${1:-}"
[ "$#" -gt 0 ] && shift
[ "$#" -gt 0 ] || {
	echo "Usage: $0 ios <Runner.app> | android <apk>..." >&2
	exit 2
}

lib=kanso_mobile_bridge
bridge_symbols=(frb_pde_ffi_dispatcher_primary kanso_background_sync kanso_passkey_answer)
failed=0

fail() {
	echo "::error::$*"
	failed=1
}

# Reports each wanted symbol that the list on stdin lacks.
require_symbols() {
	local where="$1" listed
	shift
	listed="$(cat)"
	local symbol
	for symbol in "$@"; do
		# Mach-O names carry a leading underscore.
		if ! grep -Eqx "_?$symbol" <<<"$listed"; then
			fail "$where doesn't export $symbol."
		fi
	done
	grep -Eq '^_?frbgen_kanso_rust_arc_' <<<"$listed" ||
		fail "$where exports none of the generated bridge's frbgen_kanso_rust_arc_ symbols."
}

case "$platform" in
ios)
	app="$1"
	binary="$app/Frameworks/$lib.framework/$lib"
	[ -f "$binary" ] || {
		echo "::error::$binary is missing: the library isn't embedded in the app."
		exit 1
	}
	archs="$(lipo -archs "$binary")"
	[ "$archs" = arm64 ] || fail "$binary has architectures '$archs', expected only arm64."
	nm -gU "$binary" | awk '{print $NF}' | require_symbols "$binary" "${bridge_symbols[@]}"
	[ "$failed" = 0 ] && echo "ios: $binary exports ${bridge_symbols[*]} (arm64)."
	;;
android)
	sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}"
	apkanalyzer="$(command -v apkanalyzer || true)"
	[ -n "$apkanalyzer" ] || apkanalyzer="$sdk/cmdline-tools/latest/bin/apkanalyzer"
	[ -x "$apkanalyzer" ] || {
		echo "::error::apkanalyzer isn't on PATH or in $sdk/cmdline-tools/latest/bin."
		exit 1
	}
	nm="$(echo "${ANDROID_NDK_HOME:?no Android NDK found}"/toolchains/llvm/prebuilt/*/bin/llvm-nm)"
	work="$(mktemp -d)"
	trap 'rm -rf "$work"' EXIT
	jni=(Java_app_kanso_kanso_KansoNative_init Java_app_kanso_kanso_KansoNative_backgroundSync Java_app_kanso_kanso_KansoNative_passkeyAnswer)
	for apk in "$@"; do
		[ -f "$apk" ] || {
			fail "$apk doesn't exist."
			continue
		}
		# 1. The native library of each ABI in the APK.
		entries="$(unzip -Z1 "$apk" | grep -E "^lib/[^/]+/lib$lib\.so$" || true)"
		[ -n "$entries" ] || fail "$apk has no lib/<abi>/lib$lib.so."
		for entry in $entries; do
			unzip -oq "$apk" "$entry" -d "$work"
			"$nm" -D --defined-only "$work/$entry" | awk '{print $NF}' |
				require_symbols "$apk:$entry" "${bridge_symbols[@]}" "${jni[@]}"
			[ "$failed" = 0 ] && echo "android: $(basename "$apk"):$entry exports ${bridge_symbols[*]} ${jni[*]}."
		done
		# 2. The classes proguard-rules.pro keeps.
		classes="$("$apkanalyzer" dex packages --defined-only "$apk" | awk '$1 == "C" {print $NF}')"
		for class in app.kanso.kanso.KansoNative app.kanso.kanso.KansoSecrets; do
			grep -Fqx "$class" <<<"$classes" || fail "$apk lost $class to shrinking."
		done
		grep -q '^org\.rustls\.platformverifier\.' <<<"$classes" ||
			fail "$apk has no org.rustls.platformverifier classes left."
		[ "$failed" = 0 ] && echo "android: $(basename "$apk") keeps KansoNative, KansoSecrets and org.rustls.platformverifier.*"
		# 3. Shrinking ran: R8 writes its mapping next to the outputs.
		mapping="$(dirname "$apk")/../mapping/release/mapping.txt"
		[ -s "$mapping" ] || fail "R8's mapping.txt is missing at $mapping: the release build wasn't shrunk."
	done
	;;
*)
	echo "Usage: $0 ios <Runner.app> | android <apk>..." >&2
	exit 2
	;;
esac

exit "$failed"
