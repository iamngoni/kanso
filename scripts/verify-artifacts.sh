#!/usr/bin/env bash
# Checks that the release artifacts downloaded into a directory are all there
# and whole. actions/download-artifact has twice finished without every file
# (a missing Windows zip, a truncated macOS dmg), and a short file would sign
# or publish wrong SHA256SUMS.
#
#   scripts/verify-artifacts.sh DIST VERSION NAME_REGEX
#
# Needs GH_TOKEN, GITHUB_REPOSITORY and GITHUB_RUN_ID (and `actions: read`).
# Checks that each file every platform must ship exists and isn't empty, and
# that the files together aren't smaller than the artifacts the run
# uploaded whose names match NAME_REGEX (stored uncompressed, so an artifact
# is the size of its files plus headers). If not, downloads those artifacts
# again, up to three times, then fails naming what is wrong.
set -euo pipefail

dist=${1:?usage: verify-artifacts.sh DIST VERSION NAME_REGEX}
version=${2:?version}
regex=${3:?artifact name regex}

required=(
  "Kanso-$version-macos-arm64.dmg"
  "Kanso-$version-macos-arm64.app.tar.gz"
  "Kanso-$version-windows-x64.zip"
  "Kanso-$version-linux-x64.tar.gz"
  "Kanso-$version-linux-arm64.tar.gz"
)
# Zip headers per file; a truncated file is off by far more.
slack_per_file=1024

artifacts() {
  gh api --paginate "repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/artifacts" \
    --jq ".artifacts[] | select(.expired | not) | select(.name | test(\"$regex\")) | [.name, .size_in_bytes] | @tsv"
}

# Prints the problems, one per line; no output means the download is whole.
problems() {
  local f expected have=0 count=0
  for f in "${required[@]}"; do
    if [ ! -f "$dist/$f" ]; then
      echo "missing: $f"
    elif [ ! -s "$dist/$f" ]; then
      echo "empty: $f"
    fi
  done
  expected=$(artifacts | awk -F'\t' '{ s += $2 } END { printf "%d", s }')
  while IFS= read -r -d '' f; do
    have=$((have + $(wc -c <"$f")))
    count=$((count + 1))
  done < <(find "$dist" -type f -print0)
  if [ "$((have + slack_per_file * (count + 1)))" -lt "$expected" ]; then
    echo "short: $have bytes in $count files, the run's artifacts are $expected bytes"
  fi
}

# Downloads the matching artifacts again, one directory each, over $dist.
fetch() {
  local tmp name size f
  tmp=$(mktemp -d)
  while IFS=$'\t' read -r name size; do
    gh run download "$GITHUB_RUN_ID" --repo "$GITHUB_REPOSITORY" --name "$name" --dir "$tmp/$name" ||
      echo "::warning::Downloading $name failed"
  done < <(artifacts)
  while IFS= read -r -d '' f; do
    mv -f "$f" "$dist/$(basename "$f")"
  done < <(find "$tmp" -type f -print0)
  rm -rf "$tmp"
}

for attempt in 1 2 3; do
  bad=$(problems)
  if [ -z "$bad" ]; then
    echo "All release artifacts are present and whole."
    exit 0
  fi
  echo "::warning::Artifact download incomplete (attempt $attempt of 3):"
  echo "$bad"
  [ "$attempt" -lt 3 ] || break
  fetch
done
echo "::error::The release artifacts are incomplete after retrying:"
echo "$bad"
exit 1
