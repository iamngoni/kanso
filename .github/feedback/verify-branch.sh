#!/usr/bin/env bash
# Checks a git bundle made by a feedback agent job before anything in it is pushed
# (.github/workflows/feedback-build.yml, the publish jobs). Run it in a clean checkout of
# the repository on a runner the agent never touched, never in the agent's workspace.
#
#   verify-branch.sh <bundle> <issue> <base-sha> [<previous-head>]
#
# <base-sha> is the master commit the run started from. <previous-head> is the branch head
# that is already on GitHub (the review job's case): the bundle must fast-forward it.
#
# Safe: prints `branch=...` and `head=...` lines for GITHUB_OUTPUT and exits 0. A bundle
# with no new commit prints empty values and exits 0. Anything else prints why to stderr
# and exits 1. scripts/test-feedback-build.sh runs it against hostile fixture bundles.
#
# PROTECTED must stay in step with .github/CODEOWNERS and the never-touch list in BUILD.md.
set -euo pipefail
PROTECTED='^(\.github/|\.gitmodules$|\.feedback/|cloud/wrangler|scripts/[^/]*release|scripts/mobile-release/|cliff\.toml$|\.claude/|CLAUDE\.md$|AGENTS\.md$|crates/kanso-update/|crates/kanso-gallery/keys/|(.*/)?trusted-keys\.txt$)'
MAX_COMMITS=50

fail() { echo "refused: $*" >&2; exit 1; }
[ $# -ge 3 ] || fail "usage: verify-branch.sh <bundle> <issue> <base-sha> [<previous-head>]"
bundle=$1 issue=$2 base=$3 previous=${4:-}
[[ $issue =~ ^[0-9]+$ ]] || fail "the issue number is not a number"
[[ $base =~ ^[0-9a-f]{40}$ ]] || fail "the base is not a commit id"
[[ -z $previous || $previous =~ ^[0-9a-f]{40}$ ]] || fail "the previous head is not a commit id"
start=${previous:-$base}

# Exactly one branch, named for this issue. Checked before the name is used anywhere.
heads=$(git bundle list-heads "$bundle" 2>/dev/null) || fail "not a git bundle"
[ "$(printf '%s\n' "$heads" | grep -c .)" -eq 1 ] || fail "the bundle must hold exactly one branch"
sha=${heads%% *}
ref=${heads#* }
name_re="^refs/heads/(feedback/${issue}-[a-z0-9]+(-[a-z0-9]+){0,7})\$"
[[ $ref =~ $name_re ]] && [ "${#ref}" -le 100 ] || fail "the branch is not feedback/${issue}-<slug>"
branch=${BASH_REMATCH[1]}
[[ $sha =~ ^[0-9a-f]{40}$ ]] || fail "the bundle's head is not a commit id"

git bundle verify -q "$bundle" >/dev/null 2>&1 || fail "the bundle does not apply to this repository"
git fetch -q --no-tags "$bundle" "+$ref:refs/feedback-verify/head"
[ "$(git rev-parse refs/feedback-verify/head)" = "$sha" ] || fail "the bundle's head moved"

git merge-base --is-ancestor "$base" "$sha" || fail "the branch does not build on master ($base)"
git merge-base --is-ancestor "$start" "$sha" || fail "the branch does not fast-forward $start"
if [ "$sha" = "$start" ]; then
  echo "branch="
  echo "head="
  exit 0
fi

[ -z "$(git rev-list --min-parents=2 "$start..$sha")" ] || fail "merge commits are not allowed"
[ "$(git rev-list --count "$start..$sha")" -le "$MAX_COMMITS" ] || fail "more than $MAX_COMMITS commits"

# Every path any new commit touched, with renames split into a delete and an add so the old
# name is checked too, as raw bytes (-z: no quoting of non-ASCII names).
paths=$(git log --no-renames --format= --name-only -z "$start..$sha" | tr '\0' '\n')
hits=$(printf '%s\n' "$paths" | grep -E "$PROTECTED" || true)
[ -z "$hits" ] || fail "protected paths changed: $(printf '%s' "$hits" | tr '\n' ' ')"

# No submodule pointer or symlink, added, changed or removed.
modes=$(git log --no-renames --format= --raw "$start..$sha" | awk '$1 ~ /^:(160000|120000)$/ || $2 ~ /^(160000|120000)$/')
[ -z "$modes" ] || fail "submodule or symlink changes are not allowed"

# Best effort, the agent could encode it: no secret-shaped string in what was added.
SECRET_SHAPES='sk-ant-[a-z]{2,4}[0-9]{2}-|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}'
if git log -p --no-renames --format= "$start..$sha" | grep -E '^\+' | grep -qE "$SECRET_SHAPES"; then
  fail "the change holds a secret-shaped string"
fi

echo "branch=$branch"
echo "head=$sha"
