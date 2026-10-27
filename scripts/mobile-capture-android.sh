#!/usr/bin/env bash
# Runs the capture acceptance tests on an Android emulator (#48 step 10): a
# link, an image, a PDF and two files at once are shared to Kanso's share
# activity and each becomes a note in the Inbox.
# Usage: scripts/mobile-capture-android.sh "<AVD name>" [screenshot folder]
#
# It boots the AVD (an emulator already running for it is used), builds the
# app (flutter build apk --debug), runs the instrumentation tests in
# mobile/android/app/src/androidTest (they read the library with
# android.database.sqlite in Kanso's process) and then shares a link with
# Kanso's process stopped (`am force-stop`, `am start`), reading the library
# with sqlite3 through run-as. The share's toast and Kanso's list are
# screenshotted into the folder (default: a new temporary one, printed at the
# end). The emulator is shut down at the end unless it was already running.
# Needs the Android SDK (ANDROID_HOME, default ~/Library/Android/sdk),
# Flutter and sqlite3.
set -euo pipefail

avd="${1:?Usage: $0 \"<AVD name>\" [screenshot folder]}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
mobile="$root/mobile"
shots="${2:-$(mktemp -d)}"
mkdir -p "$shots"
sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}"
adb="$sdk/platform-tools/adb"
package=app.kanso.kanso
work="$(mktemp -d)"
started_emulator=""

cleanup() {
	rm -rf "$work"
	if [ -n "$started_emulator" ]; then
		"$adb" -s "$serial" emu kill >/dev/null 2>&1 || true
	fi
}
trap cleanup EXIT

# The emulator for this AVD: one already running, else a new one.
serial=""
for candidate in $("$adb" devices | awk '/^emulator-/ { print $1 }'); do
	if [ "$("$adb" -s "$candidate" emu avd name 2>/dev/null | head -n 1 | tr -d '\r')" = "$avd" ]; then
		serial="$candidate"
	fi
done
if [ -z "$serial" ]; then
	nohup "$sdk/emulator/emulator" -avd "$avd" -no-snapshot-save -no-audio -no-boot-anim \
		>"$work/emulator.log" 2>&1 &
	started_emulator=1
	for _ in $(seq 120); do
		serial="$("$adb" devices | awk '/^emulator-/ && $2 == "device" { print $1; exit }')"
		[ -n "$serial" ] && break
		sleep 2
	done
	[ -n "$serial" ] || {
		echo "The emulator didn't start." >&2
		exit 1
	}
fi
export ANDROID_SERIAL="$serial"
for _ in $(seq 120); do
	[ "$("$adb" shell getprop sys.boot_completed | tr -d '\r')" = 1 ] && break
	sleep 2
done
echo "Emulator: $avd ($serial)"

(cd "$mobile" && flutter build apk --debug)
status=0
(cd "$mobile/android" && ./gradlew :app:connectedDebugAndroidTest \
	-Pandroid.testInstrumentationRunnerArguments.class=app.kanso.kanso.CaptureShareTest) || status=$?

# The cold share: Kanso's process stopped, a link shared with `am start`.
# (Gradle removes the app after its tests, so it is installed again.)
"$adb" install -r "$mobile/build/app/outputs/flutter-apk/app-debug.apk" >/dev/null
"$adb" shell am start -n "$package/.MainActivity" >/dev/null
for _ in $(seq 60); do
	"$adb" shell run-as "$package" test -f files/kanso/library.db && break
	sleep 1
done
sleep 3
"$adb" shell am force-stop "$package"
title="Kanso cold share $(date +%s)"
"$adb" shell am start -n "$package/.ShareActivity" -a android.intent.action.SEND -t text/plain \
	--es android.intent.extra.SUBJECT "'$title'" \
	--es android.intent.extra.TEXT "'https://example.com/cold-share'" >/dev/null
sleep 1
"$adb" exec-out screencap -p >"$shots/share-confirmation.png"
sleep 4
for file in library.db library.db-wal library.db-shm; do
	"$adb" exec-out run-as "$package" cat "files/kanso/$file" >"$work/$file" 2>/dev/null || true
done
count="$(sqlite3 "$work/library.db" "SELECT COUNT(*) FROM docs WHERE id LIKE 'note:cap-%' AND title = '$title' AND book_id = 'book:inbox'")"
if [ "$count" = 1 ]; then
	echo "ok   a link shared with the app's process stopped became an Inbox note"
else
	echo "FAIL the cold share left $count notes"
	status=1
fi

# Kanso's list with the shared notes.
"$adb" shell am start -n "$package/.MainActivity" >/dev/null
sleep 8
"$adb" exec-out screencap -p >"$shots/kanso-list.png"
echo "Screenshots: $shots"
exit "$status"
