#!/bin/sh
# Bumps the PDFium build Kanso ships (issue #43, step 15). The weekly
# workflow (.github/workflows/pdfium-bump.yml) runs it.
#
#   scripts/bump-pdfium.sh                 latest bblanchon/pdfium-binaries release
#   scripts/bump-pdfium.sh --build 8125    a given build
#   scripts/bump-pdfium.sh --dry-run       change nothing; show the diff it would make
#   scripts/bump-pdfium.sh --report FILE   also write the PR description to FILE
#
# What it does:
#   1. finds the build (the release tag chromium/<build>);
#   2. downloads that release's non-V8 archive for every platform
#      scripts/fetch-pdfium.sh fetches, and computes each SHA-256 (the V8
#      archives, pdfium-v8-*.tgz, are never touched);
#   3. rewrites PDFIUM_BUILD and the checksums in scripts/fetch-pdfium.sh;
#   4. looks for a `pdfium-render` release with a `pdfium_<build>` feature
#      (the newest 0.x.* of the pinned minor version). If there is one it
#      rewrites the feature and the `=` pin in crates/kanso-pdf/Cargo.toml and
#      refreshes Cargo.lock. If there isn't, it leaves the wrapper alone: the
#      binary and wrapper then disagree, and scripts/check-pdfium-pair.sh
#      (a CI step) fails until a wrapper catches up.
#
# It never runs the corpus or opens the PR; the workflow does both.
# Needs curl, jq, and (outside --dry-run) cargo. GH_TOKEN, when set, is sent to
# the GitHub API to avoid its anonymous rate limit.
#
# With $GITHUB_OUTPUT set it writes: status (unchanged, bumped or no-wrapper),
# build, previous, wrapper, and changed (true or false).
set -eu

cd "$(dirname "$0")/.."

REPO=bblanchon/pdfium-binaries
CRATE=pdfium-render
PLATFORMS="mac-arm64 mac-x64 linux-x64 linux-arm64 win-x64 win-arm64"
FETCH=scripts/fetch-pdfium.sh
MANIFEST=crates/kanso-pdf/Cargo.toml

dry_run=0
build=
report=
while [ "$#" -gt 0 ]; do
    case "$1" in
    --dry-run) dry_run=1 ;;
    --build)
        build=${2:-}
        shift
        ;;
    --report)
        report=${2:-}
        shift
        ;;
    *)
        echo "usage: bump-pdfium.sh [--dry-run] [--build N] [--report FILE]" >&2
        exit 2
        ;;
    esac
    shift
done

fail() {
    echo "bump-pdfium: $1" >&2
    exit 1
}
command -v jq >/dev/null 2>&1 || fail "jq is needed."

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

api() {
    if [ -n "${GH_TOKEN:-}" ]; then
        curl --fail --silent --show-error --location -H "Authorization: Bearer $GH_TOKEN" -H "Accept: application/vnd.github+json" "$1"
    else
        curl --fail --silent --show-error --location -H "Accept: application/vnd.github+json" "$1"
    fi
}
# crates.io refuses a request with no User-Agent; the GitHub token is never sent there.
crates() {
    curl --fail --silent --show-error --location -H "User-Agent: kanso-pdfium-bump (github.com/iamngoni/kanso)" "https://crates.io/api/v1/crates/$1"
}
sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}
output() {
    [ -z "${GITHUB_OUTPUT:-}" ] || echo "$1=$2" >>"$GITHUB_OUTPUT"
}

previous=$(sed -n 's/^PDFIUM_BUILD=\([0-9][0-9]*\)$/\1/p' "$FETCH")
[ -n "$previous" ] || fail "$FETCH has no PDFIUM_BUILD line."
wrapper_now=$(sed -n 's/^pdfium-render *=.*version *= *"=\([0-9][0-9.]*\)".*/\1/p' "$MANIFEST")
[ -n "$wrapper_now" ] || fail "$MANIFEST does not pin pdfium-render with an exact version."

# 1. The build. A release tag is chromium/<build>; anything else is refused,
# since the value goes into a URL and into the files.
if [ -z "$build" ]; then
    tag=$(api "https://api.github.com/repos/$REPO/releases/latest" | jq -r .tag_name)
    build=${tag#chromium/}
fi
case "$build" in
'' | *[!0-9]*) fail "'$build' is not a PDFium build number." ;;
esac
[ "$build" -ge "$previous" ] || fail "build $build is older than the pinned $previous."

if [ "$build" = "$previous" ]; then
    echo "PDFium is already at build $build; nothing to bump."
    output status unchanged
    output build "$build"
    output previous "$previous"
    output wrapper "$wrapper_now"
    output changed false
    exit 0
fi

# 2. The checksums of the non-V8 archives, computed here from the bytes.
echo "PDFium $previous -> $build"
for platform in $PLATFORMS; do
    archive="pdfium-$platform.tgz"
    curl --fail --silent --show-error --location --output "$work/$archive" \
        "https://github.com/$REPO/releases/download/chromium/$build/$archive" ||
        fail "could not download $archive for build $build."
    sum=$(sha256 "$work/$archive")
    echo "$sum" >"$work/sum-$platform"
    echo "  $platform $sum"
done

# 3 and 4 are done on a copy under $work when dry; otherwise in place.
if [ "$dry_run" -eq 1 ]; then
    mkdir -p "$work/new/scripts" "$work/new/crates/kanso-pdf"
    cp "$FETCH" "$work/new/$FETCH"
    cp "$MANIFEST" "$work/new/$MANIFEST"
    target=$work/new
else
    target=.
fi
edit() {
    sed "$2" "$target/$1" >"$work/edit.tmp"
    cat "$work/edit.tmp" >"$target/$1" # keeps the file's mode
}

edit "$FETCH" "s/^PDFIUM_BUILD=.*/PDFIUM_BUILD=$build/"
for platform in $PLATFORMS; do
    edit "$FETCH" "s/^\\(    $platform) echo \\)[0-9a-f]*\\( ;;\\)\$/\\1$(cat "$work/sum-$platform")\\2/"
done

# 4. A wrapper that has a feature for this build: the newest release of the
# same 0.x line (a new minor version of a 0.x crate may break its API, and is
# a human's call).
line=${wrapper_now%.*}
wrapper=
all_versions=$(crates "$CRATE" | jq -r '.versions[] | select(.yanked | not) | .num') ||
    fail "could not read the $CRATE versions from crates.io."
candidates=$(printf '%s\n' "$all_versions" | grep "^$line\.[0-9][0-9]*\$" | sort -t. -k3 -n -r || true)
for v in $candidates; do
    if crates "$CRATE/$v" | jq -e --arg f "pdfium_$build" '.version.features | has($f)' >/dev/null; then
        wrapper=$v
        break
    fi
done
# What the newest wrapper does cover, for the report when there is no match.
newest_feature=
if [ -z "$wrapper" ]; then
    newest=$(printf '%s\n' "$candidates" | head -n 1)
    [ -z "$newest" ] || newest_feature=$(crates "$CRATE/$newest" |
        jq -r '[.version.features | keys[] | select(test("^pdfium_[0-9]+$")) | ltrimstr("pdfium_") | tonumber] | max // empty')
fi

if [ -n "$wrapper" ]; then
    status=bumped
    # The feature name, the "# <build>." line of the comment above the pin, and the version.
    edit "$MANIFEST" "s/\"pdfium_$previous\"/\"pdfium_$build\"/; s/\`pdfium_$previous\`/\`pdfium_$build\`/; s/^# $previous\\./# $build./; /^$CRATE *=/s/version = \"=[0-9.]*\"/version = \"=$wrapper\"/"
    echo "$CRATE $wrapper_now -> $wrapper (has pdfium_$build)"
else
    status=no-wrapper
    wrapper=$wrapper_now
    echo "No $CRATE $line.x release has a pdfium_$build feature (newest covers ${newest_feature:-unknown}); the wrapper is left at $wrapper_now."
fi

if [ "$dry_run" -eq 1 ]; then
    echo
    echo "Dry run: the diff this would make (Cargo.lock would be refreshed by 'cargo update -p $CRATE --precise $wrapper' when the wrapper changes):"
    diff -u --label "a/$FETCH" --label "b/$FETCH" "$FETCH" "$work/new/$FETCH" || true
    diff -u --label "a/$MANIFEST" --label "b/$MANIFEST" "$MANIFEST" "$work/new/$MANIFEST" || true
else
    if [ "$wrapper" != "$wrapper_now" ]; then
        cargo update -p "$CRATE" --precise "$wrapper"
    fi
fi

if [ -n "$report" ]; then
    {
        echo "Bumps the PDFium binary from build $previous to **$build** (\`bblanchon/pdfium-binaries\` \`chromium/$build\`)."
        echo
        echo "- Non-V8 archives only; SHA-256s were computed from the downloaded archives for: $PLATFORMS."
        if [ "$status" = bumped ]; then
            echo "- \`pdfium-render\` $wrapper_now -> $wrapper, feature \`pdfium_$previous\` -> \`pdfium_$build\`."
        else
            echo
            echo "> [!WARNING]"
            echo "> **No \`pdfium-render\` $line.x release has a \`pdfium_$build\` feature** (the newest covers build ${newest_feature:-unknown}), so the wrapper is still built for build $previous. \`scripts/check-pdfium-pair.sh\` fails on this PR on purpose: a binary and wrapper from different PDFium APIs are not shipped. Wait for a wrapper release that has \`pdfium_$build\`, or bump to the newest build the wrapper covers (\`scripts/bump-pdfium.sh --build ${newest_feature:-<build>}\`)."
        fi
        echo
        echo "If this build fixes a PDFium security issue, Kanso releases with it within a week (issue #43, decision 6)."
    } >"$report"
fi

output status "$status"
output build "$build"
output previous "$previous"
output wrapper "$wrapper"
output changed true
