#!/usr/bin/env bash
# Looks inside the signed IPA that is about to go to TestFlight (#48 step 13).
# Usage: scripts/mobile-release/ipa-check.sh <Kanso.ipa> <version> <build number>
#
# The IPA is the bundle app.kanso.Kanso, at this version and build number, for
# iOS 17 and later, signed for the App Store by team 4973Y692JU, without the
# debugging entitlement, with the library in it, and with its export
# compliance answer in the Info.plist. macOS only (plutil, codesign).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[ "$#" -eq 3 ] || {
	echo "Usage: $0 <Kanso.ipa> <version> <build number>" >&2
	exit 2
}
ipa="$1" version="$2" number="$3"
failed=0
fail() {
	echo "::error::$*"
	failed=1
}

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
unzip -q "$ipa" -d "$work"
app="$(echo "$work"/Payload/*.app)"
[ -d "$app" ] || {
	echo "::error::$ipa has no Payload/*.app."
	exit 1
}

plist() { plutil -extract "$1" raw -o - "$app/Info.plist" 2>/dev/null || true; }
expect() { # key, wanted
	local got
	got="$(plist "$1")"
	[ "$got" = "$2" ] || fail "Info.plist $1 is '${got:-missing}', expected '$2'."
}
expect CFBundleIdentifier app.kanso.Kanso
expect CFBundleShortVersionString "$version"
expect CFBundleVersion "$number"
expect MinimumOSVersion 17.0
# The export compliance answer (docs/RELEASING-MOBILE.md): present, so App
# Store Connect doesn't ask for it with every build.
[ -n "$(plist ITSAppUsesNonExemptEncryption)" ] ||
	fail "Info.plist has no ITSAppUsesNonExemptEncryption."

# Signed for distribution by the team, not for development.
signature="$(codesign -dvv "$app" 2>&1 || true)"
grep -q '^TeamIdentifier=4973Y692JU$' <<<"$signature" ||
	fail "the app isn't signed by team 4973Y692JU."
# Older distribution certificates are named iPhone Distribution.
grep -Eq '^Authority=(Apple|iPhone) Distribution' <<<"$signature" ||
	fail "the app isn't signed with an Apple Distribution certificate."
entitlements="$(codesign -d --entitlements :- "$app" 2>/dev/null || true)"
# codesign prints the plist on one line, so the value is read by key.
if [ "$(plutil -extract get-task-allow raw -o - - <<<"$entitlements" 2>/dev/null || true)" = true ]; then
	fail "the app carries get-task-allow: it is a development build."
fi
# App Store Connect refuses a framework whose binary asks for a newer iOS
# than its own Info.plist says (ITMS-90208).
for framework in "$app"/Frameworks/*.framework; do
	[ -d "$framework" ] || continue
	name="$(basename "$framework" .framework)"
	wanted="$(plutil -extract MinimumOSVersion raw -o - "$framework/Info.plist" 2>/dev/null || true)"
	minos="$(vtool -show-build "$framework/$name" 2>/dev/null | awk '/minos/ {print $2; exit}')"
	[ -z "$wanted" ] || [ -z "$minos" ] && continue
	[ "$(printf '%s\n%s\n' "$minos" "$wanted" | sort -V | tail -n 1)" = "$wanted" ] ||
		fail "$name.framework's binary needs iOS $minos but its Info.plist says $wanted (ITMS-90208)."
done
[ -f "$app/embedded.mobileprovision" ] || fail "the app has no embedded.mobileprovision."
codesign --verify --deep --strict "$app" 2>&1 || fail "codesign --verify failed."

# The library is in and exports what the bridge and the background task need.
"$here/../mobile-artifact-check.sh" ios "$app" || failed=1

[ "$failed" = 0 ] && echo "ipa: $(basename "$ipa") is app.kanso.Kanso $version ($number), signed by 4973Y692JU for distribution."
exit "$failed"
