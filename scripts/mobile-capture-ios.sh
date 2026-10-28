#!/usr/bin/env bash
# Runs the capture acceptance tests on an iOS Simulator (#48 step 10): a link
# from Safari, an image from Photos and a PDF from Files (and from Safari)
# are shared to Kanso's share extension with Kanso not running, and each
# becomes a note in the Inbox.
# Usage: scripts/mobile-capture-ios.sh "<simulator name>" [screenshot folder]
#
# It boots the simulator, builds the app (flutter build ios --simulator) and
# the UI tests (mobile/ios/CaptureUITests), installs Kanso, launches it once
# (that makes the library in the App Group; the extension only writes a drop
# until library.db exists), puts the test photo in Photos and the test PDF in
# Files (On My iPhone/iPad), serves the test page on localhost:8765, runs the
# UI tests, then reads the library with sqlite3 to check the notes and the
# attachments. Screenshots of the share confirmations and of Kanso's list go
# to the folder (default: a new temporary one, printed at the end).
# KANSO_SKIP_BUILD=1 reuses the last build.
# Needs Xcode, Flutter, sqlite3 and python3.
set -euo pipefail

name="${1:?Usage: $0 \"<simulator name>\" [screenshot folder]}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
mobile="$root/mobile"
media="$mobile/integration_test/media"
shots="${2:-$(mktemp -d)}"
mkdir -p "$shots"
derived="$(mktemp -d)"
bundle="app.kanso.Kanso"
server_pid=""

cleanup() {
	[ -n "$server_pid" ] && kill "$server_pid" 2>/dev/null || true
	rm -rf "$derived"
}
trap cleanup EXIT

udid="$(xcrun simctl list devices available -j | python3 -c '
import json, sys
name = sys.argv[1]
for runtime in json.load(sys.stdin)["devices"].values():
    for device in runtime:
        if device["name"] == name:
            print(device["udid"])
            raise SystemExit
' "$name")"
[ -n "$udid" ] || {
	echo "No simulator called \"$name\"." >&2
	exit 1
}
echo "Simulator: $name ($udid)"
xcrun simctl boot "$udid" 2>/dev/null || true
xcrun simctl bootstatus "$udid" -b >/dev/null

if [ -z "${KANSO_SKIP_BUILD:-}" ]; then
	(cd "$mobile" && flutter build ios --simulator --debug)
fi
(cd "$mobile/ios" && xcodebuild build-for-testing -workspace Runner.xcworkspace -scheme Runner \
	-destination "platform=iOS Simulator,id=$udid" -derivedDataPath "$derived" >"$derived/build.log" 2>&1) || {
	tail -30 "$derived/build.log" >&2
	exit 1
}

# Kanso, launched once so the library exists, then closed.
xcrun simctl terminate "$udid" "$bundle" 2>/dev/null || true
xcrun simctl install "$udid" "$derived/Build/Products/Debug-iphonesimulator/Runner.app"
xcrun simctl launch "$udid" "$bundle" >/dev/null
group="$(xcrun simctl get_app_container "$udid" "$bundle" groups | awk -F'\t' '$1 == "group.app.kanso.Kanso" { print $2 }')"
library="$group/Library/Application Support/kanso/library.db"
for _ in $(seq 60); do
	[ -f "$library" ] && break
	sleep 1
done
[ -f "$library" ] || {
	echo "Kanso didn't make its library." >&2
	exit 1
}
sleep 3
xcrun simctl terminate "$udid" "$bundle"

# The photo in Photos, the PDF in Files (the local File Provider's folder).
xcrun simctl addmedia "$udid" "$media/Kanso-test-photo.png"
files_local="$(xcrun simctl get_app_container "$udid" com.apple.DocumentsApp groups |
	awk -F'\t' '$1 == "group.com.apple.FileProvider.LocalStorage" { print $2 }')"
mkdir -p "$files_local/File Provider Storage"
cp "$media/Quarterly-report.pdf" "$files_local/File Provider Storage/"

# The page the link test shares.
(cd "$media" && exec python3 -m http.server 8765 --bind 127.0.0.1 >/dev/null 2>&1) &
server_pid=$!
sleep 1

started_ms=$(($(date +%s) * 1000))
status=0
(cd "$mobile/ios" && TEST_RUNNER_CAPTURE_SHOTS="$shots" xcodebuild test-without-building \
	-workspace Runner.xcworkspace -scheme Runner -only-testing:CaptureUITests \
	-destination "platform=iOS Simulator,id=$udid" -derivedDataPath "$derived") \
	>"$derived/test.log" 2>&1 || status=$?
grep -E "Test Case .* (passed|failed)|skipped|error:" "$derived/test.log" | cut -c1-300 || true

# The library itself: the notes and files the shares made.
sql() { sqlite3 "$library" "$1"; }
check() {
	local what="$1" count="$2"
	if [ "$count" -ge 1 ]; then
		echo "ok   $what"
	else
		echo "FAIL $what"
		status=1
	fi
}
echo "Library checks (notes made since the run started):"
inbox="type = 'note' AND deleted = 0 AND id LIKE 'note:cap-%' AND book_id = 'book:inbox' AND created_at >= $started_ms"
check "a link note" "$(sql "SELECT COUNT(*) FROM docs WHERE $inbox AND title = 'Kanso capture link test'")"
check "an image note (![](inkdrop://file:)" "$(sql "SELECT COUNT(*) FROM docs WHERE $inbox AND json_extract(json, '\$.body') LIKE '%![](inkdrop://file:%'")"
check "a PDF note ([Quarterly-report.pdf](inkdrop://file:)" "$(sql "SELECT COUNT(*) FROM docs WHERE $inbox AND json_extract(json, '\$.body') LIKE '%[Quarterly-report.pdf](inkdrop://file:%'")"
check "a file doc image/png" "$(sql "SELECT COUNT(*) FROM docs WHERE type = 'file' AND id LIKE 'file:cap-%' AND json_extract(json, '\$.contentType') = 'image/png' AND created_at >= $started_ms")"
check "a file doc application/pdf" "$(sql "SELECT COUNT(*) FROM docs WHERE type = 'file' AND id LIKE 'file:cap-%' AND json_extract(json, '\$.contentType') = 'application/pdf' AND created_at >= $started_ms")"
left="$(find "$group/Library/Application Support/kanso/inbox" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l | tr -d ' ')"
check "inbox folder empty" "$([ "$left" = 0 ] && echo 1 || echo 0)"

echo "Screenshots: $shots"
exit "$status"
