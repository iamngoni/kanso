#!/usr/bin/env bash
# Makes a release's What's new video (docs/RELEASING.md › What's new video).
#
#   scripts/whats-new-video.sh <version> [--brief "<which items to show>"]
#
# Writes release-media/<version>/whats-new.webp from the release's section of
# CHANGELOG.md. scripts/release.sh runs it when a release has no video yet.
#
# The steps:
#   1. Read the release notes, the same ones the release page and app show.
#   2. Copy release-media/template/ into a work folder.
#   3. Ask Claude Code (headless) to write one scene per headline item,
#      modelled on the template's example scenes. Choosing the items and
#      drawing each feature in Kanso's UI takes judgement; nothing else does.
#   4. Join the scenes, check, render, convert to animated WebP, and check
#      the result against the limits the apps rely on.
#   5. Save a still from the middle of each scene beside the render, so a
#      person can look before the release ships.
#
# --brief steers step 3 (say, "show typed dates, New canvas, shared
# properties and agent reports"); without it Claude picks the top 3 or 4
# user-facing New and Fixed items itself.
#
# Needs: cargo, node (npx), ffmpeg and the claude CLI with the hyperframes
# skills installed. The work folder is kept under target/ for inspection.
set -euo pipefail

cd "$(dirname "$0")/.."

# The HyperFrames CLI version the template was made with. Pinned so a new
# release of it can't change how a release video renders.
HYPERFRAMES="npx --yes hyperframes@0.9.37"

# Every scene runs this long. The apps decode every frame into memory, so
# the whole video stays short: 4 scenes at most, about 28 seconds.
SCENE_SECONDS=7
MAX_SCENES=4

# The WebP the apps take: what docs/RELEASING.md and the feed checks say.
MAX_BYTES=8388608
WIDTH=960
HEIGHT=540
FPS=12
# Still frames are merged in an animated WebP, so a UI video has far fewer
# frames than seconds times fps. This bounds what the apps hold in memory.
MAX_FRAMES=300

die() {
	echo "whats-new-video: $*" >&2
	exit 1
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

# --- 1. The release notes -------------------------------------------------

target=${CARGO_TARGET_DIR:-$PWD/target}
work="$target/whats-new/$version"
rm -rf "$work"
mkdir -p "$work"

# The same tool and section the release workflow uses for the notes.
cargo run --quiet --locked -p kanso-update --bin kanso-release -- \
	notes --changelog CHANGELOG.md --version "$version" --out "$work/notes.md" ||
	die "CHANGELOG.md has no section for $version"

# --- 2. The template ------------------------------------------------------

cp -R release-media/template/. "$work/"
# The examples carry {{VERSION}} in place of a version.
sed -i.bak "s/{{VERSION}}/$version/g" "$work"/compositions/example-*.html
rm -f "$work"/compositions/*.bak

# --- 3. Claude writes the scenes -------------------------------------------

# The brief, when there is one, as one line of the prompt.
brief_line=""
[[ -n "$brief" ]] && brief_line="The release manager's brief for which items to show: $brief"

prompt=$(
	cat <<EOF
You are writing the scenes of Kanso $version's What's new video, a HyperFrames
project in the current folder. Use the hyperframes-core and hyperframes skills
for the composition rules.

Read notes.md (the release notes; ignore "Under the hood") and choose the
3 or 4 user-facing "New" or "Fixed" items most worth showing.
$brief_line

For each item, in order, write compositions/s<N>.html (s1.html, s2.html, …).
Model every scene on compositions/example-s1.html, example-s2.html and
example-s3.html: the same dark card, the left text column with
"0<N> · v$version", a short headline (at most 6 words), a one-line
description (at most 16 words) and the "NEW IN v$version" chip, and on the
right a window that is a faithful mock of the Kanso desktop UI showing that
feature being used. Use the same colours, fonts and sizes as the examples.
The Kanso source is at $PWD if you need to check what a screen looks like
(crates/kanso-app/src for the desktop app).

Rules for each scene file:
- the root id is s<N>-root, data-composition-id="s<N>", every id prefixed s<N>-;
- the card has data-start="0" data-duration="$SCENE_SECONDS";
- its paused GSAP timeline is window.__timelines["s<N>"], fades the card out
  at $((SCENE_SECONDS - 1)).5 s, and every text must fit its box;
- do not touch index.html or the example files; the script writes index.html.

When done, every scene must pass: $HYPERFRAMES lint
Reply with one line per scene: "s<N>: <headline>".
EOF
)

repo=$PWD
(
	cd "$work"
	claude -p "$prompt" \
		--add-dir "$repo" \
		--allowedTools "Read Write Edit Glob Grep Skill Bash(npx --yes hyperframes@0.9.37 *)"
) | tee "$work/scenes.txt" || die "Claude Code couldn't write the scenes (see $work)"

# --- 4. Join, check and render --------------------------------------------

# The scenes Claude wrote, in order.
scenes=()
for n in $(seq 1 "$MAX_SCENES"); do
	[[ -f "$work/compositions/s$n.html" ]] && scenes+=("s$n")
done
((${#scenes[@]} >= 1)) || die "no scenes were written (see $work)"
rm -f "$work"/compositions/example-*.html

# index.html places the scenes back to back on one track.
total=$((${#scenes[@]} * SCENE_SECONDS))
{
	cat <<EOF
<!doctype html>
<html lang="en"><head><meta charset="UTF-8" /><meta name="viewport" content="width=1280, height=720" />
<script src="https://cdn.jsdelivr.net/npm/gsap@3.15.0/dist/gsap.min.js"></script>
<style>*{margin:0;padding:0;box-sizing:border-box}html,body{width:1280px;height:720px;overflow:hidden;background:#121210}#root{width:100%;height:100%;position:relative}</style>
</head><body>
<div id="root" data-composition-id="main" data-start="0" data-duration="$total" data-width="1280" data-height="720">
EOF
	start=0
	for s in "${scenes[@]}"; do
		echo " <div id=\"$s\" data-composition-id=\"$s\" data-composition-src=\"compositions/$s.html\" data-start=\"$start\" data-duration=\"$SCENE_SECONDS\" data-track-index=\"1\" data-track-kind=\"graphics\" data-width=\"1280\" data-height=\"720\"></div>"
		start=$((start + SCENE_SECONDS))
	done
	cat <<EOF
</div>
<script>window.__timelines["main"]=gsap.timeline({paused:true});</script>
</body></html>
EOF
} >"$work/index.html"

(cd "$work" && $HYPERFRAMES lint) || die "the scenes fail HyperFrames lint (see $work)"
(cd "$work" && $HYPERFRAMES render --quality delivery -o whats-new.mp4) ||
	die "HyperFrames couldn't render the video (see $work)"

# The conversion in docs/RELEASING.md: 12 fps, 960 wide, looping, no sound.
# Homebrew's ffmpeg has no libwebp_anim; then ffmpeg writes the frames and
# libwebp's own img2webp encodes them with the same settings.
if ffmpeg -hide_banner -encoders 2>/dev/null | grep -q libwebp_anim; then
	ffmpeg -y -loglevel error -i "$work/whats-new.mp4" \
		-vf "fps=$FPS,scale=$WIDTH:-2:flags=lanczos" -loop 0 \
		-c:v libwebp_anim -quality 70 -compression_level 6 -an "$work/whats-new.webp" ||
		die "ffmpeg couldn't make the WebP"
else
	command -v img2webp >/dev/null ||
		die "ffmpeg has no libwebp_anim encoder and img2webp isn't installed (brew install webp)"
	mkdir -p "$work/frames"
	ffmpeg -y -loglevel error -i "$work/whats-new.mp4" \
		-vf "fps=$FPS,scale=$WIDTH:-2:flags=lanczos" "$work/frames/%04d.png" ||
		die "ffmpeg couldn't write the frames"
	# -d is each frame's length in milliseconds; identical frames merge.
	img2webp -loop 0 -lossy -q 70 -m 6 -d $((1000 / FPS)) \
		-o "$work/whats-new.webp" "$work"/frames/*.png >/dev/null ||
		die "img2webp couldn't make the WebP"
fi

# --- The limits the apps rely on ------------------------------------------

webp="$work/whats-new.webp"
size=$(wc -c <"$webp" | tr -d ' ')
((size <= MAX_BYTES)) || die "the WebP is $size bytes, over $MAX_BYTES"

# A WebP file starts RIFF....WEBP.
head=$(head -c 12 "$webp" | LC_ALL=C tr -c 'A-Z' '.')
[[ "$head" == RIFF....WEBP ]] || die "ffmpeg didn't write a WebP"

# An animated WebP has a VP8X header, then an ANIM chunk.
[[ "$(head -c 34 "$webp" | tail -c 4)" == ANIM ]] || die "the WebP isn't animated"

# VP8X stores canvas width-1 and height-1 as 24-bit little-endian numbers.
read -r w0 w1 w2 h0 h1 h2 < <(od -An -tu1 -j24 -N6 "$webp")
canvas_w=$((w0 + w1 * 256 + w2 * 65536 + 1))
canvas_h=$((h0 + h1 * 256 + h2 * 65536 + 1))
[[ "$canvas_w" == "$WIDTH" && "$canvas_h" == "$HEIGHT" ]] ||
	die "the WebP is ${canvas_w}x$canvas_h, not ${WIDTH}x$HEIGHT"

# Each frame is an ANMF chunk.
frames=$(LC_ALL=C grep -ao ANMF "$webp" | wc -l | tr -d ' ')
((frames <= MAX_FRAMES)) || die "the WebP has $frames frames, over $MAX_FRAMES"

# --- 5. Stills to look at, then the file the release ships ----------------

mids=""
for i in "${!scenes[@]}"; do
	mids+="${mids:+,}$((i * SCENE_SECONDS + SCENE_SECONDS - 2))"
done
(cd "$work" && $HYPERFRAMES snapshot --at "$mids") ||
	echo "whats-new-video: couldn't save stills; look at $work/whats-new.mp4 instead" >&2

mkdir -p "release-media/$version"
cp "$webp" "release-media/$version/whats-new.webp"
echo "What's new video: release-media/$version/whats-new.webp ($size bytes, $frames frames, ${#scenes[@]} scenes, ${total}s)"
echo "Scenes, render and stills: $work"
