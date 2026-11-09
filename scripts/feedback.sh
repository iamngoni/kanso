#!/usr/bin/env bash
# In-app feedback, from the owner's terminal (docs/FEEDBACK.md). Reads the
# production D1/R2 of the sync Worker with your wrangler login; changes nothing
# unless a command says so.
#
#   scripts/feedback.sh list [N]          newest N reports (default 20): id, age, kind, status, issue
#   scripts/feedback.sh pending           reports waiting for triage, with attempts
#   scripts/feedback.sh show <fb_id>      the full report, Haiku's triage and the status history
#   scripts/feedback.sh screenshot <id>   downloads the screenshot and opens it
#   scripts/feedback.sh stats             counts by status and kind
#   scripts/feedback.sh issues            the GitHub issues filed from the app
#   scripts/feedback.sh logs              live Worker logs (cron runs, triage, filing)
#   scripts/feedback.sh gateway           opens the AI Gateway's log page (each Haiku call)
#   scripts/feedback.sh crashes [N]       newest N crash reports
set -euo pipefail

cd "$(dirname "$0")/../cloud"

DB=kanso-sync
BUCKET=kanso-files
ACCOUNT=5425ee3677afb301418677cafaf53e59
GATEWAY=kanso-feedback
REPO=iamngoni/kanso

die() {
	echo "feedback: $*" >&2
	exit 1
}

sql() {
	npx --yes wrangler d1 execute "$DB" --remote --json --command "$1" 2>/dev/null |
		jq -r '.[0].results'
}

table() {
	sql "$1" | jq -r '(.[0] // {} | keys_unsorted) as $k | ($k | @tsv), (.[] | [.[$k[]]] | map(. // "-" | tostring) | @tsv)' |
		column -t -s $'\t'
}

check_id() {
	[[ "${1:-}" =~ ^fb_[0-9a-z]{26}$ ]] || die "not a feedback id: ${1:-<none>}"
}

AGE="CASE WHEN (strftime('%s','now')*1000 - created_at) < 3600000
            THEN ((strftime('%s','now')*1000 - created_at) / 60000) || 'm'
          WHEN (strftime('%s','now')*1000 - created_at) < 86400000
            THEN ((strftime('%s','now')*1000 - created_at) / 3600000) || 'h'
          ELSE ((strftime('%s','now')*1000 - created_at) / 86400000) || 'd' END"

cmd=${1:-list}
shift || true
case "$cmd" in
list)
	n=${1:-20}
	[[ "$n" =~ ^[0-9]+$ ]] || die "N must be a number"
	table "SELECT id, $AGE AS age, kind, status, issue, has_screenshot AS shot,
	              substr(json_extract(report, '\$.text'), 1, 60) AS text
	       FROM feedback ORDER BY created_at DESC LIMIT $n"
	;;
pending)
	table "SELECT id, $AGE AS age, kind, triage_attempts AS attempts
	       FROM feedback WHERE status = 'received' ORDER BY created_at"
	;;
show)
	check_id "${1:-}"
	sql "SELECT id, datetime(created_at/1000,'unixepoch') AS created, kind, status, issue,
	            duplicate_of, pr, release, credit_name, screenshot_state, triage_attempts,
	            json(report) AS report, json(triage) AS triage
	     FROM feedback WHERE id = '$1'" |
		jq '.[0] // error("no such report") | .report |= (fromjson? // .) | .triage |= (fromjson? // .)'
	echo
	echo "History:"
	table "SELECT datetime(at/1000,'unixepoch') AS at, status, detail
	       FROM feedback_events WHERE feedback_id = '$1' ORDER BY at"
	;;
screenshot)
	check_id "${1:-}"
	out="${TMPDIR:-/tmp}/kanso-feedback-$1"
	npx --yes wrangler r2 object get "$BUCKET/feedback/$1/screenshot" --remote --file "$out" >/dev/null 2>&1 ||
		die "no screenshot stored for $1"
	echo "$out"
	command -v open >/dev/null && open "$out"
	;;
stats)
	table "SELECT status, kind, count(*) AS n FROM feedback GROUP BY status, kind ORDER BY status, kind"
	;;
issues)
	gh issue list --repo "$REPO" --label from-app --state all --limit 50
	;;
logs)
	exec npx --yes wrangler tail --format pretty
	;;
gateway)
	url="https://dash.cloudflare.com/$ACCOUNT/ai/ai-gateway/gateways/$GATEWAY/logs"
	echo "$url"
	command -v open >/dev/null && open "$url"
	;;
crashes)
	n=${1:-20}
	[[ "$n" =~ ^[0-9]+$ ]] || die "N must be a number"
	table "SELECT id, received_day AS day, kind, type, app_version AS version FROM crash_reports
	       ORDER BY id DESC LIMIT $n"
	;;
-h | --help | help)
	sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
	;;
*)
	die "unknown command $cmd (try --help)"
	;;
esac
