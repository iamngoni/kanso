#!/usr/bin/env bash
# Makes a release's What's new video (docs/RELEASING.md › What's new video).
#
#   scripts/whats-new-video.sh <version> [--brief "<guidance for the script>"]
#
# Writes release-media/<version>/script.md and release-media/<version>/
# whats-new.webp from the release's section of CHANGELOG.md.
# scripts/release.sh runs it when a release has no video yet.
#
# The steps:
#   1. Read the release notes and list every user-facing entry in them.
#   2. Get the video script: one scene per change, or per group of closely
#      related changes, each with a headline, a one-line description and
#      what the UI mock shows. Claude Code (headless) writes it, unless a
#      reviewed release-media/<v>/script.md is already there.
#   3. Check the script covers every user-facing entry, so none is dropped
#      silently, and has no more scenes than fit.
#   4. Claude Code writes one HyperFrames scene per script scene, modelled on
#      release-media/template/'s examples. Drawing each feature in Kanso's UI
#      takes judgement; nothing else here does.
#   5. Join the scenes, lint, render, convert to animated WebP and check it
#      against the limits the apps rely on.
#   6. Save a still from each scene beside the render, for a person to read
#      before the release ships.
#
# --brief steers step 2 (say, "lead with the 0.10.8 changes").
#
# Needs: cargo, node (npx), ffmpeg (or img2webp) and the claude CLI with the
# hyperframes skills installed. The work folder is kept under target/.
set -euo pipefail

cd "$(dirname "$0")/.."
repo=$PWD

# The HyperFrames CLI version the template was made with. Pinned so a new
# release of it can't change how a release video renders.
hyperframes() {
	npx --yes hyperframes@0.9.37 "$@"
}

# Memory: both apps decode every frame of the WebP into RGBA, which is
# frames x width x height x 4 bytes. At 640x360 a frame is 0.92 MB; 10 fps
# for 30 s is at most 300 frames, so at most about 276 MB, and less in
# practice because WebP merges frames that don't change. (960x540 at 12 fps
# for 30 s would be up to 746 MB.) So: 640x360, 10 fps, 30 s, 300 frames.
WIDTH=640
HEIGHT=360
FPS=10
MAX_SECONDS=30
MAX_FRAMES=300
# The apps refuse a larger file (kanso_update::whats_new::MAX_MEDIA_BYTES).
MAX_BYTES=8388608

# Each scene is on screen this long, in tenths of a second. 12 scenes of
# 2.5 s fill the 30 s; a release with more changes groups them further.
SCENE_TENTHS=25
MAX_SCENES=$((MAX_SECONDS * 10 / SCENE_TENTHS))

# Changelog scopes that never reach a user's screen.
INTERNAL_SCOPES="ci|refactor|test|tests|docs|site|release|build|chore|deps|bench"

die() {
	echo "whats-new-video: $*" >&2
	exit 1
}

# Tenths of a second as seconds: 25 -> 2.5.
seconds() {
	printf '%d.%d' $(($1 / 10)) $(($1 % 10))
}

version=${1:-}
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] ||
	die "usage: scripts/whats-new-video.sh <version> [--brief \"…\"] (version like 0.10.9, no leading v)"
shift
brief=""
if [[ "${1:-}" == "--brief" ]]; then
	brief=${2:?--brief needs text}
	shift 2
fi
[[ $# -eq 0 ]] || die "unexpected argument: $1"

for tool in cargo npx ffmpeg claude; do
	command -v "$tool" >/dev/null || die "$tool is needed and isn't on PATH"
done

media="release-media/$version"
target=${CARGO_TARGET_DIR:-$repo/target}
work="$target/whats-new/$version"
rm -rf "$work"
mkdir -p "$work"

# --- 1. The release notes and their user-facing entries --------------------

# The same tool and section the release workflow uses for the notes.
cargo run --quiet --locked -p kanso-update --bin kanso-release -- \
	notes --changelog CHANGELOG.md --version "$version" --out "$work/notes.md" ||
	die "CHANGELOG.md has no section for $version"

# Every entry under a user-facing heading whose scope isn't internal, one
# per line. "Under the hood" is never user-facing.
awk -v internal="^\\*\\*($INTERNAL_SCOPES):\\*\\*" '
	/^### / { heading = tolower(substr($0, 5)); next }
	/^## / { heading = ""; next }
	/^- / {
		if (heading !~ /^(new|changed|improved|fixed|breaking)$/) next
		if (substr($0, 3) ~ internal) next
		print
	}
' "$work/notes.md" >"$work/entries.md"
[[ -s "$work/entries.md" ]] || die "the $version notes have no user-facing entries"

# The short commit hashes the entries link, which the script must cover.
grep -oE '\[[0-9a-f]{7}\]' "$work/entries.md" | tr -d '[]' | sort -u >"$work/hashes.txt"

# --- 2. The video script ----------------------------------------------------

script="$work/script.md"
if [[ -f "$media/script.md" ]]; then
	# A script someone already reviewed (or edited) is used as it is.
	cp "$media/script.md" "$script"
else
	brief_line=""
	[[ -n "$brief" ]] && brief_line="The release manager's guidance: $brief"

	prompt=$(
		cat <<EOF
Write the script of Kanso $version's What's new video to script.md in the
current folder. Write nothing else.

entries.md lists every user-facing change in the release notes (notes.md).
The video must show every one of them: give each change its own scene, or
group closely related changes into one scene (for example two commits about
shared properties become "Collab: shared properties"). There can be at most
$MAX_SCENES scenes; if the changes don't fit, group them further. Never leave
a change out. Order the scenes from the most to the least useful to someone
using Kanso.
$brief_line

Use exactly this format, numbering scenes from 1:

# Kanso $version · What's new

## 1 · <headline, at most 5 words>

- Description: <one line, at most 12 words, plain words for a user>
- Shows: <what the Kanso UI mock shows happening, one or two sentences>
- Covers: <the 7-character commit hashes of the entries this scene shows, comma-separated>

Every hash in entries.md must appear under exactly one "Covers:". An entry
without a hash still needs a scene or a place in one.
EOF
	)
	(cd "$work" && claude -p "$prompt" --allowedTools "Read Write Edit") >"$work/script.log" ||
		die "Claude Code couldn't write the video script (see $work/script.log)"
	[[ -f "$script" ]] || die "Claude Code didn't write script.md (see $work/script.log)"
fi

# --- 3. Check the script ----------------------------------------------------

scene_count=$(grep -cE '^## [0-9]+ · ' "$script" || true)
((scene_count >= 1)) || die "$script has no scenes (\"## 1 · Headline\")"
((scene_count <= MAX_SCENES)) ||
	die "$script has $scene_count scenes; at most $MAX_SCENES fit in ${MAX_SECONDS}s. Group related changes further."

# Every user-facing change is in the video: none dropped silently.
missing=""
while read -r hash; do
	grep -E '^- Covers:' "$script" | grep -q "$hash" || missing+=" $hash"
done <"$work/hashes.txt"
[[ -z "$missing" ]] ||
	die "$script leaves out user-facing changes:$missing (see $work/entries.md). Add them to a scene."

# --- 4. Claude writes the scenes --------------------------------------------

cp -R release-media/template/. "$work/"
# The examples carry {{VERSION}} in place of a version.
sed -i.bak "s/{{VERSION}}/$version/g" "$work"/compositions/example-*.html
rm -f "$work"/compositions/*.bak

scene_seconds=$(seconds "$SCENE_TENTHS")
prompt=$(
	cat <<EOF
You are building Kanso $version's What's new video, a HyperFrames project in
the current folder. Use the hyperframes-core and hyperframes skills for the
composition rules.

script.md is the video script. For each of its scenes N, write
compositions/s<N>.html showing that scene's headline, description and what
"Shows:" says. Model every scene on compositions/example-s1.html,
example-s2.html and example-s3.html: the same dark card, colours and fonts,
the text column on the left ("0<N> · v$version", the headline, the
description, the "NEW IN v$version" chip) and on the right a window that is
a faithful mock of the Kanso desktop UI using the feature. The Kanso source
is at $repo (crates/kanso-app/src) if you need to check what a screen
looks like.

The video plays at ${WIDTH}x$HEIGHT, half the 1280x720 you draw, and each
scene lasts only $scene_seconds seconds. So, unlike the examples:
- headline 56px, description 26px, chip 16px; text in the mock at least
  18px, with only the few elements the feature needs;
- the card is fully visible from 0 s (no card fade in or out), so scenes cut
  cleanly; the text appears by 0.4 s, the mock's one action by 1.4 s;
- every text must fit its box.

Rules for each scene file:
- the root id is s<N>-root, data-composition-id="s<N>", every id prefixed s<N>-;
- the card has data-start="0" data-duration="$scene_seconds";
- its paused GSAP timeline is window.__timelines["s<N>"];
- do not touch index.html, script.md or the example files.

When done, check your scenes with: npx --yes hyperframes@0.9.37 lint
Reply with one line per scene: "s<N>: <headline>".
EOF
)
(
	cd "$work"
	claude -p "$prompt" \
		--add-dir "$repo" \
		--allowedTools "Read Write Edit Glob Grep Skill Bash(npx --yes hyperframes@0.9.37 *)"
) >"$work/scenes.log" || die "Claude Code couldn't write the scenes (see $work/scenes.log)"

for n in $(seq 1 "$scene_count"); do
	[[ -f "$work/compositions/s$n.html" ]] || die "scene s$n wasn't written (see $work/scenes.log)"
done
rm -f "$work"/compositions/example-*.html

# --- 5. Join, render, convert and check ---------------------------------------

# index.html places the scenes back to back on one track.
total_tenths=$((scene_count * SCENE_TENTHS))
{
	cat <<EOF
<!doctype html>
<html lang="en"><head><meta charset="UTF-8" /><meta name="viewport" content="width=1280, height=720" />
<script src="https://cdn.jsdelivr.net/npm/gsap@3.15.0/dist/gsap.min.js"></script>
<style>*{margin:0;padding:0;box-sizing:border-box}html,body{width:1280px;height:720px;overflow:hidden;background:#121210}#root{width:100%;height:100%;position:relative}</style>
</head><body>
<div id="root" data-composition-id="main" data-start="0" data-duration="$(seconds "$total_tenths")" data-width="1280" data-height="720">
EOF
	for n in $(seq 1 "$scene_count"); do
		start=$(seconds $(((n - 1) * SCENE_TENTHS)))
		echo " <div id=\"s$n\" data-composition-id=\"s$n\" data-composition-src=\"compositions/s$n.html\" data-start=\"$start\" data-duration=\"$scene_seconds\" data-track-index=\"1\" data-track-kind=\"graphics\" data-width=\"1280\" data-height=\"720\"></div>"
	done
	cat <<EOF
</div>
<script>window.__timelines["main"]=gsap.timeline({paused:true});</script>
</body></html>
EOF
} >"$work/index.html"

(cd "$work" && hyperframes lint) || die "the scenes fail HyperFrames lint (see $work)"
(cd "$work" && hyperframes render --quality delivery -o whats-new.mp4) ||
	die "HyperFrames couldn't render the video (see $work)"

# Scale down, drop to 10 fps, loop, no sound. Homebrew's ffmpeg has no
# libwebp_anim; then ffmpeg writes the frames and libwebp's own img2webp
# encodes them with the same settings.
webp="$work/whats-new.webp"
filter="fps=$FPS,scale=$WIDTH:-2:flags=lanczos"
if ffmpeg -hide_banner -encoders 2>/dev/null | grep -q libwebp_anim; then
	ffmpeg -y -loglevel error -i "$work/whats-new.mp4" -vf "$filter" -loop 0 \
		-c:v libwebp_anim -quality 70 -compression_level 6 -an "$webp" ||
		die "ffmpeg couldn't make the WebP"
else
	command -v img2webp >/dev/null ||
		die "ffmpeg has no libwebp_anim encoder and img2webp isn't installed (brew install webp)"
	mkdir -p "$work/frames"
	ffmpeg -y -loglevel error -i "$work/whats-new.mp4" -vf "$filter" "$work/frames/%04d.png" ||
		die "ffmpeg couldn't write the frames"
	# -d is each frame's length in milliseconds; identical frames merge.
	img2webp -loop 0 -lossy -q 70 -m 6 -d $((1000 / FPS)) \
		-o "$webp" "$work"/frames/*.png >/dev/null ||
		die "img2webp couldn't make the WebP"
fi

size=$(wc -c <"$webp" | tr -d ' ')
((size <= MAX_BYTES)) || die "the WebP is $size bytes, over $MAX_BYTES"

# A WebP file starts "RIFF", a 4-byte size, then "WEBP".
[[ "$(head -c 4 "$webp")" == RIFF && "$(head -c 12 "$webp" | tail -c 4)" == WEBP ]] ||
	die "the encoder didn't write a WebP"

# An animated WebP has a VP8X header, then an ANIM chunk.
[[ "$(head -c 34 "$webp" | tail -c 4)" == ANIM ]] || die "the WebP isn't animated"

# VP8X stores canvas width-1 and height-1 as 24-bit little-endian numbers.
read -r w0 w1 w2 h0 h1 h2 < <(od -An -tu1 -j24 -N6 "$webp")
canvas_w=$((w0 + w1 * 256 + w2 * 65536 + 1))
canvas_h=$((h0 + h1 * 256 + h2 * 65536 + 1))
[[ "$canvas_w" == "$WIDTH" && "$canvas_h" == "$HEIGHT" ]] ||
	die "the WebP is ${canvas_w}x$canvas_h, not ${WIDTH}x$HEIGHT"

# Each frame is an ANMF chunk; their count bounds the decoded memory.
frames=$(LC_ALL=C grep -ao ANMF "$webp" | wc -l | tr -d ' ')
((frames <= MAX_FRAMES)) || die "the WebP has $frames frames, over $MAX_FRAMES"

# --- 6. Stills to read, then the files the release ships ---------------------

# 1.8 s into each scene, when its text and its mock's action are in place.
stills=""
for n in $(seq 1 "$scene_count"); do
	stills+="${stills:+,}$(seconds $(((n - 1) * SCENE_TENTHS + 18)))"
done
(cd "$work" && hyperframes snapshot --at "$stills") ||
	echo "whats-new-video: couldn't save stills; look at $work/whats-new.mp4 instead" >&2

mkdir -p "$media"
cp "$script" "$media/script.md"
cp "$webp" "$media/whats-new.webp"
echo "What's new video: $media/whats-new.webp ($size bytes, $frames frames, $scene_count scenes, $(seconds "$total_tenths")s)"
echo "Script: $media/script.md. Scenes, render and stills: $work"
