#!/usr/bin/env bash
# Cuts a Kanso release (docs/RELEASING.md).
#
#   scripts/release.sh <version> [--push]
#       On a clean, up-to-date master: bumps [workspace.package] version and
#       Cargo.lock, adds the release's section to CHANGELOG.md with
#       git-cliff, commits "chore(release): v<version>" and tags v<version>.
#       It prints the push command; --push runs it. The pushed tag starts
#       the release workflow, which builds, checks and publishes the release
#       on its own; everyone's Kanso then picks it up.
#
#   scripts/release.sh publish <version>
#       Only for a release the workflow left as a draft (an older run, or
#       one stopped between upload and publish): waits for the workflow,
#       checks the draft's feed signature, and publishes it.
set -euo pipefail

cd "$(dirname "$0")/.."

die() {
	echo "release: $*" >&2
	exit 1
}

usage() {
	sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'
	exit 1
}

check_version() {
	[[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] ||
		die "$1 isn't a version like 0.2.0 (no leading v)"
}

kanso_release() {
	cargo run --quiet --locked -p kanso-update --bin kanso-release -- "$@"
}

publish() {
	local version=$1 tag="v$1"
	check_version "$version"
	command -v gh >/dev/null || die "the GitHub CLI (gh) is needed"
	git rev-parse -q --verify "refs/tags/$tag" >/dev/null || die "there's no tag $tag here"

	echo "Waiting for the release workflow of ${tag}…"
	local run=""
	for _ in $(seq 1 60); do
		run=$(gh run list --workflow release.yml --branch "$tag" --limit 1 \
			--json databaseId --jq '.[0].databaseId // empty')
		[[ -n "$run" ]] && break
		sleep 10
	done
	[[ -n "$run" ]] || die "no release workflow run for $tag; was the tag pushed?"
	gh run watch "$run" --exit-status || die "the release workflow failed: gh run view $run --log-failed"

	local draft
	draft=$(gh release view "$tag" --json isDraft --jq .isDraft) || die "no release $tag on GitHub"
	[[ "$draft" == "true" ]] || die "$tag is already published"

	local dir
	dir=$(mktemp -d)
	trap 'rm -rf "$dir"' RETURN
	gh release download "$tag" --pattern 'latest.json*' --dir "$dir"
	kanso_release verify --feed "$dir/latest.json" ||
		die "the draft's latest.json doesn't verify against this checkout's trusted keys"
	grep -q "\"version\": \"$version\"" "$dir/latest.json" || die "the draft's feed isn't for $version"

	gh release edit "$tag" --draft=false --latest
	echo "Published $tag. Everyone's Kanso finds it within six hours, or at its next start."
}

cut() {
	local version=$1 push=$2 tag="v$1"
	check_version "$version"

	[[ "$(git rev-parse --abbrev-ref HEAD)" == "master" ]] || die "releases are cut from master"
	[[ -z "$(git status --porcelain)" ]] || die "the working tree isn't clean"
	if git remote get-url origin >/dev/null 2>&1; then
		git fetch -q origin master
		[[ "$(git rev-parse HEAD)" == "$(git rev-parse origin/master)" ]] ||
			die "master isn't the same as origin/master; pull or push first"
	fi
	! git rev-parse -q --verify "refs/tags/$tag" >/dev/null || die "$tag exists already"
	grep -qvE '^\s*(#|$)' crates/kanso-update/keys/trusted-keys.txt ||
		die "crates/kanso-update/keys/trusted-keys.txt has no key, so the release couldn't sign its feed (docs/RELEASING.md › One-time setup)"

	local current
	current=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
	echo "Releasing $version (was $current)."

	# The version in [workspace.package], which every crate inherits.
	awk -v v="$version" '
		/^\[workspace\.package\]/ { in_pkg = 1 }
		/^\[/ && !/^\[workspace\.package\]/ { in_pkg = 0 }
		in_pkg && /^version = / && !done { print "version = \"" v "\""; done = 1; next }
		{ print }
	' Cargo.toml >Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml
	cargo update --workspace --quiet

	# The changelog: a section written ahead (as 0.1.0's was) gets today's
	# date; otherwise git-cliff writes it from the commits since the last tag.
	local today
	today=$(date -u +%Y-%m-%d)
	if grep -qE "^## \[?v?${version//./\\.}[] ]" CHANGELOG.md; then
		sed -i.bak -E "s/^(## \[?v?${version//./\\.}\]?(\([^)]*\))?) - [0-9]{4}-[0-9]{2}-[0-9]{2}/\1 - $today/" CHANGELOG.md
		rm -f CHANGELOG.md.bak
	else
		command -v git-cliff >/dev/null || die "git-cliff is needed: cargo install git-cliff --locked"
		local section
		section=$(mktemp)
		git cliff --unreleased --tag "$tag" --strip header >"$section"
		[[ -s "$section" ]] || die "git-cliff found no conventional commits since the last release"
		# After the header, before the newest release.
		awk -v file="$section" '
			function insert(   n, i, lines, line) {
				while ((getline line < file) > 0) lines[++n] = line
				while (n > 0 && lines[n] == "") n--
				for (i = 1; i <= n; i++) print lines[i]
				inserted = 1
			}
			!inserted && /^## / { insert(); print "" }
			{ print }
			END { if (!inserted) insert() }
		' CHANGELOG.md >CHANGELOG.md.tmp && mv CHANGELOG.md.tmp CHANGELOG.md
		rm -f "$section"
	fi
	kanso_release notes --changelog CHANGELOG.md --version "$version" --out /dev/null >/dev/null ||
		die "CHANGELOG.md has no section for $version"
	# Every breaking change needs an update notice (docs/RELEASING.md). The
	# tree was clean, so a refused release puts back what this run changed.
	if ! kanso_release check-notices --changelog CHANGELOG.md --version "$version"; then
		git checkout -q -- CHANGELOG.md Cargo.toml Cargo.lock
		die "$version isn't released: fix the notices above (docs/RELEASING.md › Breaking changes), then run this again. CHANGELOG.md, Cargo.toml and Cargo.lock are as they were."
	fi

	# The What's new video, if this release has one (docs/RELEASING.md ›
	# What's new video): committed at release-media/<v>/whats-new.webp, an
	# animated WebP up to 8 MB. Anything else there is refused now rather than
	# left out of the release by the workflow.
	local media="release-media/$version"
	if [[ -d "$media" ]]; then
		local video="$media/whats-new.webp" bad
		bad=$(find "$media" -type f ! -path "$video" -print -quit)
		if [[ -n "$bad" || ! -f "$video" ]]; then
			git checkout -q -- CHANGELOG.md Cargo.toml Cargo.lock
			die "$media should hold whats-new.webp and nothing else (found ${bad:-no whats-new.webp})"
		fi
		local size head
		size=$(wc -c <"$video" | tr -d ' ')
		head=$(head -c 12 "$video" | LC_ALL=C tr -c 'A-Z' '.')
		if ((size > 8388608)) || [[ "$head" != RIFF....WEBP ]]; then
			git checkout -q -- CHANGELOG.md Cargo.toml Cargo.lock
			die "$video must be a WebP of at most 8 MB (it is $size bytes)"
		fi
		echo "What's new video: $video ($size bytes)"
	fi

	git add Cargo.toml Cargo.lock CHANGELOG.md
	# The first release may change nothing (0.1.0 is already the version and
	# its section is written): the commit still marks where it was cut.
	git commit -q --allow-empty -m "chore(release): $tag"
	git tag -a "$tag" -m "Kanso $version"
	echo
	echo "Committed and tagged $tag. Read CHANGELOG.md's new section, then:"
	echo "  git push origin master && git push origin $tag"
	echo "The workflow builds, checks and publishes it."
	if [[ "$push" == "--push" ]]; then
		git push origin master
		git push origin "$tag"
		echo "Pushed. The release workflow builds, checks and publishes $tag."
	fi
}

case "${1:-}" in
"" | -h | --help) usage ;;
publish)
	[[ $# -eq 2 ]] || usage
	publish "$2"
	;;
*)
	[[ $# -le 2 ]] || usage
	[[ -z "${2:-}" || "${2:-}" == "--push" ]] || usage
	cut "$1" "${2:-}"
	;;
esac
