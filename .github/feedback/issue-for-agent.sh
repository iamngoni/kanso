#!/usr/bin/env bash
# Writes what a feedback agent may read to .feedback/ in the checkout, before the agent
# starts (.github/workflows/feedback-build.yml). The agent has no GitHub access of its own.
#
#   ISSUE=<n> [PR=<n>] GH_TOKEN=... GITHUB_REPOSITORY=owner/repo issue-for-agent.sh
#
# .feedback/issue.md  the issue's title, labels, body, and only the comments by the owner
#                     and by this workflow (its plans). Anyone on GitHub can comment on a
#                     public issue, so every other comment is left out: the agent must never
#                     read instructions from a stranger.
# .feedback/pr.md     with PR set: the pull request's title, branch, body and file list,
#                     never its comments or reviews, for the same reason.
# .feedback/out/      where the agent writes its comment, PR title and body, or review.
#
# .feedback/ is excluded from git, and the publish jobs refuse a branch that touches it.
set -euo pipefail
: "${ISSUE:?}" "${GITHUB_REPOSITORY:?}"
OWNER=iamngoni

mkdir -p .feedback/out
grep -qxF '/.feedback/' .git/info/exclude 2>/dev/null || echo '/.feedback/' >>.git/info/exclude

# A comment counts as trusted when the owner or this workflow (github-actions) wrote it.
# gh reports bots without the "[bot]" suffix; strip it either way.
# shellcheck disable=SC2016 # jq's \( \) and $owner, not the shell's
issue_jq='
def trusted: ((.author.login // "") | sub("\\[bot\\]$"; "")) as $l | ($l == $owner or $l == "github-actions");
"# Issue #\(.number): \(.title)\n\nLabels: \([.labels[].name] | join(", "))\n\n## Body\n\n\(.body)\n\n"
+ "## Comments by the owner and by this workflow\n"
+ ([.comments[] | select(trusted) | "\n### \(.author.login), \(.createdAt)\n\n\(.body)\n"] | join(""))
+ "\n(\([.comments[] | select(trusted | not)] | length) comments by other people are left out.)\n"'

gh issue view "$ISSUE" -R "$GITHUB_REPOSITORY" --json number,title,body,labels,comments \
  | jq -r --arg owner "$OWNER" "$issue_jq" >.feedback/issue.md

if [ -n "${PR:-}" ]; then
  # shellcheck disable=SC2016
  pr_jq='"# Pull request #\(.number): \(.title)\n\nBranch: \(.headRefName) into \(.baseRefName)\n\n## Files\n\n"
  + ([.files[] | "- \(.path) (+\(.additions) -\(.deletions))"] | join("\n"))
  + "\n\n## Body\n\n\(.body)\n"'
  gh pr view "$PR" -R "$GITHUB_REPOSITORY" --json number,title,body,headRefName,baseRefName,files \
    | jq -r "$pr_jq" >.feedback/pr.md
fi

# Public data either way; in the log it can be audited against what the agent did.
# Its lines must not be read as workflow commands (`::add-mask::`, `::set-output` ...).
stop="feedback-$RANDOM$RANDOM$RANDOM"
echo "::group::.feedback/issue.md (what the agent reads)"
echo "::stop-commands::$stop"
cat .feedback/issue.md
echo "::$stop::"
echo "::endgroup::"
