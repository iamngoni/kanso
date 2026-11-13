#!/usr/bin/env bash
# Shared notebooks in the desktop app, end to end against a local Worker
# (issue #390 slice 38).
#
# Everything here is Miniflare: a throwaway state directory, `wrangler dev`
# on 127.0.0.1 with sign-ups open and a fixed test pepper, and the app's
# tests in crates/kanso-app/src/book_share_e2e.rs pointed at it. They sign
# real accounts up, then drive the Share notebook sheet and the Shared
# section over HTTP and the notebook's live socket. No Cloudflare
# credentials are read or needed; the script unsets them to be sure.
#
#   cloud/scripts/book-share-e2e.sh        from anywhere in the repository
#
# KANSO_E2E_KEEP=1 keeps the state directory for inspection. The Worker
# listens on a port the OS picks, so runs side by side never collide.
set -euo pipefail

unset CLOUDFLARE_API_TOKEN CLOUDFLARE_ACCOUNT_ID IMNGONII_CLOUDFLARE_API_TOKEN

CLOUD="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="$(cd "$CLOUD/.." && pwd)"
ENDPOINT=""
# A fixed test pepper (bytes 0..31, as in vitest.config.ts), never a deployed one.
TEST_PEPPER="AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"

STATE="$(mktemp -d "${TMPDIR:-/tmp}/kanso-book-e2e.XXXXXX")"
DEV_PID=""
cleanup() {
	if [[ -n "$DEV_PID" ]] && kill -0 "$DEV_PID" 2>/dev/null; then
		kill "$DEV_PID" 2>/dev/null || true
		wait "$DEV_PID" 2>/dev/null || true
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

# 1. The migrations, and sign-ups open.
"${WRANGLER[@]}" d1 migrations apply kanso-sync --local --persist-to "$STATE"
"${WRANGLER[@]}" d1 execute kanso-sync --local --persist-to "$STATE" --command "UPDATE settings SET value = 'open' WHERE name = 'signup'"

# 2. The Worker, on a port the OS picks; its "Ready on" line names it.
"${WRANGLER[@]}" dev --local --persist-to "$STATE" --ip 127.0.0.1 --port 0 --inspector-port 0 \
	--show-interactive-dev-session=false --var "AUTH_PEPPER:$TEST_PEPPER" >"$STATE/wrangler-dev.log" 2>&1 &
DEV_PID=$!
for _ in $(seq 1 120); do
	if [[ -z "$ENDPOINT" ]]; then
		READY="$(grep -Eo 'Ready on http://127\.0\.0\.1:[0-9]+' "$STATE/wrangler-dev.log" | head -n 1 || true)"
		ENDPOINT="${READY#Ready on }"
	fi
	if [[ -n "$ENDPOINT" ]] && curl -fsS "$ENDPOINT/v2/health" >/dev/null 2>&1; then break; fi
	if ! kill -0 "$DEV_PID" 2>/dev/null; then
		echo "wrangler dev exited:" >&2
		cat "$STATE/wrangler-dev.log" >&2
		exit 1
	fi
	sleep 0.5
done
if [[ -z "$ENDPOINT" ]] || ! curl -fsS "$ENDPOINT/v2/health" >/dev/null; then
	echo "The Worker never answered /v2/health:" >&2
	cat "$STATE/wrangler-dev.log" >&2
	exit 1
fi
echo "The Worker listens on $ENDPOINT."

# 3. The app's shared notebook tests against it.
cd "$REPO"
KANSO_BOOK_E2E_ENDPOINT="$ENDPOINT" \
	cargo test -p kanso-app book_share_e2e -- --ignored --test-threads=1 --nocapture
