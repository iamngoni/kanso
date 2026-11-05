#!/usr/bin/env bash
# `ok` never fails, so "check && ok || bad" works as an if-then-else here.
# shellcheck disable=SC2015

# Checks cliff.toml's Breaking group and `kanso-release check-notices` against
# fixture commits in a throwaway repository (docs/RELEASING.md "Breaking
# changes"):
#   - a breaking commit (`type!:`, or a BREAKING CHANGE / BREAKING-CHANGE footer)
#     with a `Notice: <id>` footer goes under Breaking, first, and ends with
#     "(notice: <id>)";
#   - a contributor's breaking commit keeps "by @login", with the notice after it;
#   - a Notice footer on a commit that isn't breaking prints nothing;
#   - a malformed notice ID isn't printed;
#   - the gate passes the section with only noticed breaking changes, fails one
#     with a breaking change and no notice (naming the entry and the fix), and
#     passes a release with no breaking change.
# Needs git, git-cliff and cargo (or KANSO_RELEASE set to a built kanso-release).
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

kanso_release() {
  if [[ -n "${KANSO_RELEASE:-}" ]]; then
    "$KANSO_RELEASE" "$@"
  else
    (cd "$root" && cargo run --quiet --locked -p kanso-update --bin kanso-release -- "$@")
  fi
}

# The notice the fixtures name: the first one the app is built with.
id="$(sed -n 's/^pub const API_KEYS_ID: &str = "\(.*\)";/\1/p' "$root/crates/kanso-core/src/notices.rs")"
[[ -n "$id" ]] || { echo "FAIL no notice ID found in crates/kanso-core/src/notices.rs"; exit 1; }

repo="$tmp/repo"
mkdir "$repo"
cd "$repo"
git init -q -b main
git config commit.gpgsign false

commit() { # commit <author name> <author email> <message>
  GIT_AUTHOR_NAME="$1" GIT_AUTHOR_EMAIL="$2" GIT_COMMITTER_NAME="$1" GIT_COMMITTER_EMAIL="$2" \
    git commit -q --allow-empty -m "$3"
}
owner=("Ngonidzashe Mangudya" "owner@example.invalid")
sam=("Sam" "1234+samdev@users.noreply.github.com")

commit "${owner[@]}" "feat: The first release"
git tag v0.9.0
# 0.9.1: nothing breaking.
commit "${owner[@]}" "fix: A crash"
commit "${sam[@]}" "feat: A contributor's feature"
git tag v0.9.1
# 0.10.0: every breaking change has its notice.
commit "${owner[@]}" $'feat(server)!: Drop the login\n\nNotice: '"$id"
commit "${owner[@]}" $'fix: Footer breaking\n\nThe body.\n\nBREAKING CHANGE: the old way is gone\nNotice: '"$id"
commit "${owner[@]}" $'fix: Hyphen footer breaking\n\nBREAKING-CHANGE: gone\nNotice: '"$id"
commit "${sam[@]}" $'feat!: A contributor\'s breaking change\n\nNotice: '"$id"
commit "${owner[@]}" $'feat: Not breaking\n\nNotice: '"$id"
# A contributor branch merged with a merge commit (merge, don't squash).
git checkout -q -b side
commit "${sam[@]}" "fix: A merged contributor fix"
git checkout -q main
GIT_AUTHOR_NAME="${owner[0]}" GIT_AUTHOR_EMAIL="${owner[1]}" GIT_COMMITTER_NAME="${owner[0]}" \
  GIT_COMMITTER_EMAIL="${owner[1]}" git merge -q --no-ff side -m "Merge pull request #78 from samdev/side"
git tag v0.10.0
# 0.11.0: breaking changes without a usable notice.
commit "${owner[@]}" "refactor!: Breaks with no notice"
commit "${owner[@]}" $'feat!: Breaks with a bad ID\n\nNotice: Not An ID'

cliff() { git cliff --config "$root/cliff.toml" "$@" --strip all 2>/dev/null; }
cliff v0.9.0..v0.9.1 --tag v0.9.1 >"$tmp/0.9.1.md"
cliff v0.9.1..v0.10.0 --tag v0.10.0 >"$tmp/0.10.0.md"
cliff --unreleased --tag v0.11.0 >"$tmp/0.11.0.md"
cat "$tmp/0.10.0.md" "$tmp/0.11.0.md"

fail=0
ok() { echo "ok   $1"; }
bad() { echo "FAIL $1"; fail=1; }
line() { grep -F -- "$2" "$tmp/$1.md" || true; }
ends() { # ends <description> <release> <subject> <suffix>
  if [[ "$(line "$2" "$3")" == *"$4" ]]; then ok "$1"; else bad "$1 (wanted the line to end: $4)"; fi
}

[[ "$(grep -m1 '^### ' "$tmp/0.10.0.md")" == "### Breaking" ]] && ok "Breaking is the first group" ||
  bad "Breaking is the first group"
breaking="$(sed -n '/^### Breaking/,/^### /p' "$tmp/0.10.0.md")"
for subject in "Drop the login" "Footer breaking" "Hyphen footer breaking" "A contributor's breaking change"; do
  [[ "$breaking" == *"$subject"* ]] && ok "under Breaking: $subject" || bad "under Breaking: $subject"
done
[[ "$breaking" != *"Not breaking"* ]] && ok "a commit that isn't breaking stays out of Breaking" ||
  bad "a commit that isn't breaking stays out of Breaking"
ends "a notice footer ends the entry" 0.10.0 "Drop the login" "(notice: $id)"
ends "a BREAKING CHANGE footer's entry gets its notice" 0.10.0 "Footer breaking" "(notice: $id)"
ends "a contributor's breaking change is credited, then its notice" 0.10.0 \
  "A contributor's breaking change" "**(breaking)** by @samdev (notice: $id)"
ends "a contributor's feature is credited" 0.9.1 "A contributor's feature" "by @samdev"
ends "a merged contributor commit is credited" 0.10.0 "A merged contributor fix" "by @samdev"
[[ "$(line 0.10.0 "Not breaking")" != *"(notice:"* ]] && ok "no notice printed for a commit that isn't breaking" ||
  bad "no notice printed for a commit that isn't breaking"
[[ "$(line 0.11.0 "bad ID")" != *"(notice:"* ]] && ok "a malformed notice ID isn't printed" ||
  bad "a malformed notice ID isn't printed"

# The gate, on each release's section as release.sh would write it.
gate() { # gate <release>
  { echo "# Changelog"; echo; cat "$tmp/$1.md"; } >"$tmp/CHANGELOG-$1.md"
  kanso_release check-notices --changelog "$tmp/CHANGELOG-$1.md" --version "$1" >"$tmp/gate-$1.txt" 2>&1
}
gate 0.9.1 && ok "the gate passes a release with nothing breaking" ||
  { bad "the gate passes a release with nothing breaking"; cat "$tmp/gate-0.9.1.txt"; }
gate 0.10.0 && ok "the gate passes breaking changes with notices" ||
  { bad "the gate passes breaking changes with notices"; cat "$tmp/gate-0.10.0.txt"; }
if gate 0.11.0; then
  bad "the gate fails breaking changes without a notice"
else
  cat "$tmp/gate-0.11.0.txt"
  ok "the gate fails breaking changes without a notice"
  for want in "Breaks with no notice" "Breaks with a bad ID" "Notice: <id>" "section in CHANGELOG.md ahead"; do
    grep -qF -- "$want" "$tmp/gate-0.11.0.txt" && ok "the failure says: $want" || bad "the failure says: $want"
  done
fi

exit "$fail"
