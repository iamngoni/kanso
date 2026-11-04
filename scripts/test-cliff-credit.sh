#!/usr/bin/env bash
# Checks cliff.toml's credit rules against fixture commits in a throwaway repository
# (docs/RELEASING.md, docs/FEEDBACK.md "Credit"):
#   - a `Reported-by: Name (in app)` trailer credits the plain name, never an @-mention;
#   - a name the Worker would refuse (links, HTML, entities, @, backticks, too long) gets no
#     credit at all, since anyone can write a trailer;
#   - the feedback agent's own commits (claude[bot]) are not credited as a contributor;
#   - a contributor's no-reply address still ends "by @login";
#   - a commit with no trailer gets nothing extra.
# Needs git and git-cliff.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"
git init -q -b main
git config commit.gpgsign false

commit() { # commit <author name> <author email> <message>
  GIT_AUTHOR_NAME="$1" GIT_AUTHOR_EMAIL="$2" GIT_COMMITTER_NAME="$1" GIT_COMMITTER_EMAIL="$2" \
    git commit -q --allow-empty -m "$3"
}
bot="209825114+claude[bot]@users.noreply.github.com"
commit "claude[bot]" "$bot" $'fix: Keep wide tables inside the PDF page\n\nCloses #300\n\nReported-by: tendai (in app)'
commit "claude[bot]" "$bot" $'fix: Name with spaces\n\nReported-by: Mary Jane-Doe (in app)'
commit "claude[bot]" "$bot" $'fix: Hostile name\n\nReported-by: @evil (in app)'
commit "claude[bot]" "$bot" $'fix: Unicode name\n\nReported-by: Zoë Ndlovu-Smith (in app)'
commit "claude[bot]" "$bot" $'fix: Link name\n\nReported-by: [win](https://evil.example) (in app)'
commit "claude[bot]" "$bot" $'fix: Html name\n\nReported-by: <img src=https://evil.example/p.gif> (in app)'
commit "claude[bot]" "$bot" $'fix: Entity name\n\nReported-by: &#64;evil (in app)'
commit "claude[bot]" "$bot" $'fix: Backtick name\n\nReported-by: `rm -rf` (in app)'
commit "claude[bot]" "$bot" $'fix: Long name\n\nReported-by: Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa (in app)'
commit "Sam" "1234+samdev@users.noreply.github.com" $'fix: Contributor trailer\n\nReported-by: **bold** [x](https://evil.example)'
commit "claude[bot]" "$bot" "fix: Fix without a reporter"
commit "Sam" "1234+samdev@users.noreply.github.com" "feat: A contributor's feature"
commit "Ngonidzashe Mangudya" "owner@example.invalid" $'feat: The owner\'s feature'

out="$(git cliff --config "$root/cliff.toml" --unreleased --strip all)"
echo "$out"

fail=0
expect() { # expect <description> <fixed string that must appear>
  if grep -qF -- "$2" <<<"$out"; then echo "ok   $1"; else echo "FAIL $1 (missing: $2)"; fail=1; fi
}
refuse() { # refuse <description> <fixed string that must not appear>
  if grep -qF -- "$2" <<<"$out"; then echo "FAIL $1 (found: $2)"; fail=1; else echo "ok   $1"; fi
}
line() { grep -F -- "$1" <<<"$out" || true; }

expect "a reporter is credited by plain name" ", reported by tendai"
expect "a name with spaces is kept whole" ", reported by Mary Jane-Doe"
expect "a non-ASCII name is kept" ", reported by Zoë Ndlovu-Smith"
refuse "no @-mention for a reporter" "@tendai"
refuse "no @-mention for a hostile name" "@evil"
refuse "the trailer's '(in app)' is not printed" "(in app)"
refuse "the agent's bot account is not credited" "claude[bot]"
refuse "no link from a trailer" "evil.example"
refuse "no HTML from a trailer" "<img"
refuse "no entity from a trailer" "&#64;"
refuse "no backticks from a trailer" '`'
refuse "no Markdown emphasis from a trailer" "**bold**"
for subject in "Hostile name" "Link name" "Html name" "Entity name" "Backtick name" "Long name" "Contributor trailer"; do
  if [[ "$(line "$subject")" == *"reported by"* ]]; then echo "FAIL no credit for an unsafe name ($subject)"; fail=1; else echo "ok   no credit for an unsafe name ($subject)"; fi
done
expect "a contributor is still credited by login" "by @samdev"
if [[ "$(line "Fix without a reporter")" == *"reported by"* ]]; then echo "FAIL no credit without a trailer"; fail=1; else echo "ok   no credit without a trailer"; fi
if [[ "$(line "The owner's feature")" == *" by "* ]]; then echo "FAIL owner is credited as a contributor"; fail=1; else echo "ok   owner is not credited as a contributor"; fi

exit "$fail"
