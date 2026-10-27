#!/usr/bin/env bash
# End to end for the phone's sync (#48 step 8) against a local Worker, with a
# second device played by the command-line peer (crates/kanso-mobile/examples/e2e_peer.rs).
#
# Everything here is Miniflare: a throwaway state directory, `wrangler dev` on
# 127.0.0.1 with sign-ups open and a fixed test pepper. No Cloudflare
# credentials are read or needed; the script unsets them to be sure, and
# nothing here talks to the production server.
#
#   scripts/mobile-sync-e2e.sh                       the facade's tests against the Worker
#   scripts/mobile-sync-e2e.sh --device <flutter id> and the Flutter flows on that device
#
# 1. `cargo test -p kanso-mobile --features test-support --test e2e_worker
#    -- --ignored --test-threads=1` with KANSO_E2E_ENDPOINT set. The test file
#    is written by another builder; while it doesn't exist the step says so
#    and is skipped.
# 2. With --device, `flutter test mobile/integration_test/sync_flow_test.dart`
#    (skipped, with a note, while the file doesn't exist). First the peer sets
#    the world up, then the flows run, then the peer's side is checked:
#      - three accounts are made with `sign-up`: the phone's (U, with notes),
#        a friend's (F), and a second one for joining by recovery key (R, with
#        notes). Each prints its recovery key.
#      - `request-approval` for U asks, in the background, to join U's account
#        as a new device; the phone, signed in as U, approves it from its
#        sheet, and the peer joins.
#      - `share` makes a note owned by F and shares it with U; the phone finds
#        the invitation, accepts, edits the note, and a background `wait-text`
#        for F ends when the phone's edit arrives.
#    The flow test and the peer meet only on the server: the phone waits for
#    the peer's request and its note, and the peer waits for the phone's
#    approval and edit. What the test
#    needs comes in as --dart-define values (below); the peers' JSON lines are
#    kept in $STATE/peers/*.log, and $STATE/flow.json collects what the setup
#    made, for people reading a failure.
#
# The values the flow test can read with String.fromEnvironment:
#   KANSO_E2E_SERVER           the Worker as the device reaches it
#   KANSO_E2E_USER             the phone's account, U
#   KANSO_E2E_PASSWORD         its password (and everyone's here)
#   KANSO_E2E_RECOVERY_KEY     U's recovery key
#   KANSO_E2E_NOTES            how many notes U holds
#   KANSO_E2E_CANARY_PREFIX    U's note text is "<prefix>-<n>", n from 0
#   KANSO_E2E_RECOVERY_USER    R, the account to join with its recovery key
#   KANSO_E2E_RECOVERY_ACCOUNT_KEY   R's recovery key
#   KANSO_E2E_RECOVERY_CANARY_PREFIX the text prefix of R's notes
#   KANSO_E2E_FRIEND           F, who shared a note with U
#   KANSO_E2E_SHARE_TITLE      the shared note's title
#   KANSO_E2E_SHARE_BODY       and its text
#   KANSO_E2E_EDIT_TEXT        what the phone should append to it
#   KANSO_E2E_NEW_PASSWORD     a password for R's new device (recovery sets one)
#
# KANSO_E2E_PORT fixes the Worker's port (a port in use, and 19840, are
# refused); otherwise the OS picks one. KANSO_E2E_KEEP=1 keeps the state
# directory. The Worker's log is printed when a step fails.
set -euo pipefail

unset CLOUDFLARE_API_TOKEN CLOUDFLARE_ACCOUNT_ID IMNGONII_CLOUDFLARE_API_TOKEN

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLOUD="$REPO/cloud"
# 0 has wrangler dev bind a port the OS picks, in the same step that takes it,
# so no other process can grab it first; its "Ready on" line names the port.
PORT="${KANSO_E2E_PORT:-0}"
# A fixed test pepper (bytes 0..31, as in vitest.config.ts), never a deployed one.
TEST_PEPPER="AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"
# The test password of every account here: throwaway data on a throwaway Worker.
TEST_PASSWORD="correct horse battery e2e"

DEVICE=""
while [[ $# -gt 0 ]]; do
	case "$1" in
	--device)
		[[ $# -ge 2 ]] || {
			echo "--device needs a Flutter device id." >&2
			exit 2
		}
		DEVICE="$2"
		shift 2
		;;
	-h | --help)
		sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'
		exit 0
		;;
	*)
		echo "Unknown argument $1 (see --help)." >&2
		exit 2
		;;
	esac
done

if [[ "$PORT" == "19840" ]]; then
	echo "Port 19840 is the desktop app's; set KANSO_E2E_PORT to another port." >&2
	exit 1
fi
if [[ "$PORT" != "0" ]] && (exec 3<>"/dev/tcp/127.0.0.1/$PORT") 2>/dev/null; then
	echo "Something already listens on 127.0.0.1:$PORT; set KANSO_E2E_PORT to a free port, or unset it to take one the OS picks." >&2
	exit 1
fi

# The state directory is a throwaway one, with the peers' libraries and logs
# beside the Worker's data.
STATE="$(mktemp -d "${TMPDIR:-/tmp}/kanso-mobile-e2e.XXXXXX")"
mkdir -p "$STATE/worker" "$STATE/peers"
DEV_PID=""
BACKGROUND_PIDS=()
ENDPOINT=""
FAILED=1

stop_worker() {
	if [[ -n "$DEV_PID" ]] && kill -0 "$DEV_PID" 2>/dev/null; then
		kill "$DEV_PID" 2>/dev/null || true
		wait "$DEV_PID" 2>/dev/null || true
	fi
	DEV_PID=""
}
stop_peers() {
	local pid
	for pid in ${BACKGROUND_PIDS[@]+"${BACKGROUND_PIDS[@]}"}; do
		if kill -0 "$pid" 2>/dev/null; then
			kill "$pid" 2>/dev/null || true
			wait "$pid" 2>/dev/null || true
		fi
	done
	BACKGROUND_PIDS=()
}
cleanup() {
	stop_peers
	stop_worker
	if [[ "$FAILED" == "1" && -f "$STATE/worker/wrangler-dev.log" ]]; then
		echo "--- The Worker's log ---" >&2
		tail -n 80 "$STATE/worker/wrangler-dev.log" >&2 || true
	fi
	if [[ "${KANSO_E2E_KEEP:-}" == "1" ]]; then
		echo "State kept in $STATE"
	else
		rm -rf "$STATE"
	fi
}
trap cleanup EXIT

cd "$CLOUD"
if [[ ! -x node_modules/.bin/wrangler ]]; then
	echo "Run npm ci in cloud/ first." >&2
	exit 1
fi
WRANGLER=(node_modules/.bin/wrangler)
WORKER_STATE="$STATE/worker/data"

# 1. The migrations, and sign-ups open.
"${WRANGLER[@]}" d1 migrations apply kanso-sync --local --persist-to "$WORKER_STATE"
"${WRANGLER[@]}" d1 execute kanso-sync --local --persist-to "$WORKER_STATE" --command "UPDATE settings SET value = 'open' WHERE name = 'signup'"
# The default takes five new accounts a day; the facade's tests make more.
"${WRANGLER[@]}" d1 execute kanso-sync --local --persist-to "$WORKER_STATE" --command "UPDATE settings SET value = '500' WHERE name = 'signups_per_day'"

# 2. The Worker, with /__scheduled for the cron.
"${WRANGLER[@]}" dev --local --persist-to "$WORKER_STATE" --ip 127.0.0.1 --port "$PORT" --inspector-port 0 \
	--test-scheduled --show-interactive-dev-session=false --var "AUTH_PEPPER:$TEST_PEPPER" >"$STATE/worker/wrangler-dev.log" 2>&1 &
DEV_PID=$!

# Only this run's wrangler dev writes this log, so the port it names is ours.
for _ in $(seq 1 120); do
	if [[ -z "$ENDPOINT" ]]; then
		READY="$(grep -Eo 'Ready on http://127\.0\.0\.1:[0-9]+' "$STATE/worker/wrangler-dev.log" | head -n 1 || true)"
		ENDPOINT="${READY#Ready on }"
	fi
	if [[ -n "$ENDPOINT" ]] && curl -fsS "$ENDPOINT/v2/health" >/dev/null 2>&1; then break; fi
	if ! kill -0 "$DEV_PID" 2>/dev/null; then
		echo "wrangler dev exited." >&2
		exit 1
	fi
	sleep 0.5
done
if [[ -z "$ENDPOINT" ]]; then
	echo "wrangler dev never said which port it listens on." >&2
	exit 1
fi
# wrangler dev may move to another port when a fixed one is taken; then the
# tests would talk to whatever holds the port asked for.
if [[ "$PORT" != "0" && "$ENDPOINT" != "http://127.0.0.1:$PORT" ]]; then
	echo "wrangler dev listens on $ENDPOINT, not port $PORT." >&2
	exit 1
fi
if [[ "$ENDPOINT" == *":19840" ]]; then
	echo "wrangler dev took the desktop app's port." >&2
	exit 1
fi
echo "The Worker listens on $ENDPOINT."
curl -fsS "$ENDPOINT/v2/health" || {
	echo "The Worker didn't answer /v2/health." >&2
	exit 1
}
echo

cd "$REPO"

# 3. The facade against the Worker.
if [[ -f crates/kanso-mobile/tests/e2e_worker.rs ]]; then
	KANSO_E2E_ENDPOINT="$ENDPOINT" \
		cargo test -p kanso-mobile --features test-support --test e2e_worker -- --ignored --test-threads=1 --nocapture
else
	echo "Skipping the facade's end-to-end tests: crates/kanso-mobile/tests/e2e_worker.rs doesn't exist yet."
fi

# 4. The Flutter flows, when a device was named.
if [[ -z "$DEVICE" ]]; then
	FAILED=0
	echo "No --device given: the Flutter flows are not run."
	exit 0
fi
FLOW=mobile/integration_test/sync_flow_test.dart
if [[ ! -f "$FLOW" ]]; then
	FAILED=0
	echo "Skipping the Flutter flows: $FLOW doesn't exist yet."
	exit 0
fi

# Where the device reaches this computer: an iOS simulator shares the host's
# loopback, and an Android emulator is given it with `adb reverse` (the app
# takes plain http only to 127.0.0.1, so 10.0.2.2 won't do). A real phone
# can't reach a Worker bound to 127.0.0.1, so it is refused rather than
# guessed at.
command -v jq >/dev/null || {
	echo "jq is needed to read 'flutter devices --machine'." >&2
	exit 1
}
DEVICE_INFO="$(flutter devices --machine | jq -c --arg id "$DEVICE" '[.[] | select(.id == $id)][0] // empty')"
if [[ -z "$DEVICE_INFO" ]]; then
	echo "Flutter doesn't list a device with id $DEVICE (flutter devices)." >&2
	exit 1
fi
PLATFORM="$(jq -r '.targetPlatform' <<<"$DEVICE_INFO")"
EMULATOR="$(jq -r '.emulator' <<<"$DEVICE_INFO")"
case "$PLATFORM:$EMULATOR" in
android*:true)
	PORT_OF_WORKER="${ENDPOINT##*:}"
	adb -s "$DEVICE" reverse "tcp:$PORT_OF_WORKER" "tcp:$PORT_OF_WORKER"
	DEVICE_HOST=127.0.0.1
	;;
ios:true) DEVICE_HOST=127.0.0.1 ;;
*)
	echo "$DEVICE is a $PLATFORM device that isn't a simulator or emulator; it can't reach a Worker on 127.0.0.1." >&2
	exit 1
	;;
esac
PORT_OF_WORKER="${ENDPOINT##*:}"
DEVICE_SERVER="http://$DEVICE_HOST:$PORT_OF_WORKER"
echo "The device reaches the Worker as $DEVICE_SERVER."

cargo build -p kanso-mobile --example e2e_peer
PEER_BIN="${CARGO_TARGET_DIR:-$REPO/target}/debug/examples/e2e_peer"
[[ -x "$PEER_BIN" ]] || {
	echo "$PEER_BIN wasn't built." >&2
	exit 1
}

RUN="$(date +%s)$$"
USER_U="phone${RUN}"
USER_F="friend${RUN}"
USER_R="recover${RUN}"
NOTES=3
EDIT_TEXT=" phone-edit-$RUN"
SHARE_TITLE="Shared by the peer $RUN"
SHARE_BODY="peer-body-$RUN"

peer() { "$PEER_BIN" "$@"; }

# One JSON line's field, from the last line with that event.
field() { # file event key
	jq -r --arg e "$2" --arg k "$3" 'select(.event == $e) | .[$k]' "$1" | tail -n 1
}

sign_up() { # user dir log
	peer sign-up --server "$ENDPOINT" --username "$1" --password "$TEST_PASSWORD" --notes "$NOTES" --dir "$STATE/peers/$1" >"$STATE/peers/$1.log"
}

echo "Setting the world up with the peer."
sign_up "$USER_U"
sign_up "$USER_F"
sign_up "$USER_R"
RECOVERY_U="$(field "$STATE/peers/$USER_U.log" ready recovery_key)"
RECOVERY_R="$(field "$STATE/peers/$USER_R.log" ready recovery_key)"

peer share --server "$ENDPOINT" --username "$USER_F" --password "$TEST_PASSWORD" --to "$USER_U" \
	--title "$SHARE_TITLE" --body "$SHARE_BODY" --dir "$STATE/peers/$USER_F" >"$STATE/peers/share.log"
SHARE_ID="$(field "$STATE/peers/share.log" shared share_id)"
[[ -n "$SHARE_ID" ]] || {
	echo "The peer didn't share a note." >&2
	exit 1
}

# What the setup made, for people reading a failure. The phone never reads it:
# a device can't, which is why the values travel as dart-defines.
jq -n --arg server "$DEVICE_SERVER" --arg u "$USER_U" --arg f "$USER_F" --arg r "$USER_R" --arg share "$SHARE_ID" \
	'{server: $server, phone_account: $u, friend: $f, recovery_account: $r, share_id: $share}' >"$STATE/flow.json"

# The peers that wait for the phone: one asking it for approval, one to see
# its edit arrive. Their lines are in the logs, and a failure of either fails the run.
peer request-approval --server "$ENDPOINT" --username "$USER_U" --password "$TEST_PASSWORD" --timeout-secs 900 \
	--dir "$STATE/peers/${USER_U}-new-device" >"$STATE/peers/request-approval.log" 2>"$STATE/peers/request-approval.err" &
APPROVER_PID=$!
BACKGROUND_PIDS+=("$APPROVER_PID")
peer wait-text --server "$ENDPOINT" --username "$USER_F" --password "$TEST_PASSWORD" --share "$SHARE_ID" \
	--contains "$EDIT_TEXT" --timeout-secs 900 --dir "$STATE/peers/$USER_F" >"$STATE/peers/wait-text.log" 2>"$STATE/peers/wait-text.err" &
WATCHER_PID=$!
BACKGROUND_PIDS+=("$WATCHER_PID")

echo "Running the Flutter flows on $DEVICE."
FLOW_STATUS=0
(
	cd mobile
	flutter test "integration_test/sync_flow_test.dart" -d "$DEVICE" \
		--dart-define="KANSO_E2E_SERVER=$DEVICE_SERVER" \
		--dart-define="KANSO_E2E_USER=$USER_U" \
		--dart-define="KANSO_E2E_PASSWORD=$TEST_PASSWORD" \
		--dart-define="KANSO_E2E_RECOVERY_KEY=$RECOVERY_U" \
		--dart-define="KANSO_E2E_NOTES=$NOTES" \
		--dart-define="KANSO_E2E_CANARY_PREFIX=canary-$USER_U" \
		--dart-define="KANSO_E2E_RECOVERY_USER=$USER_R" \
		--dart-define="KANSO_E2E_RECOVERY_ACCOUNT_KEY=$RECOVERY_R" \
		--dart-define="KANSO_E2E_RECOVERY_CANARY_PREFIX=canary-$USER_R" \
		--dart-define="KANSO_E2E_FRIEND=$USER_F" \
		--dart-define="KANSO_E2E_SHARE_TITLE=$SHARE_TITLE" \
		--dart-define="KANSO_E2E_SHARE_BODY=$SHARE_BODY" \
		--dart-define="KANSO_E2E_EDIT_TEXT=$EDIT_TEXT" \
		--dart-define="KANSO_E2E_NEW_PASSWORD=$TEST_PASSWORD new"
) || FLOW_STATUS=$?

# The peer's side of what the flows should have done: the phone approved the
# peer's request, and the phone's edit reached the note's owner.
PEER_STATUS=0
for name in approver watcher; do
	case "$name" in
	approver) pid="$APPROVER_PID" log=request-approval ;;
	*) pid="$WATCHER_PID" log=wait-text ;;
	esac
	if [[ "$FLOW_STATUS" == "0" ]]; then
		if ! wait "$pid"; then
			echo "The peer's $log failed:" >&2
			cat "$STATE/peers/$log.log" "$STATE/peers/$log.err" >&2
			PEER_STATUS=1
		fi
	fi
done
if [[ "$FLOW_STATUS" != "0" || "$PEER_STATUS" != "0" ]]; then
	echo "--- The peers' lines ---" >&2
	tail -n 20 "$STATE"/peers/*.log "$STATE"/peers/*.err >&2 2>/dev/null || true
	exit 1
fi
FAILED=0
echo "The Flutter flows passed: the phone approved the peer's request, and its edit arrived."
