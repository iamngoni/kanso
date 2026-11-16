# Release media

`release-media/<version>/whats-new.webp` is that release's What's new video:
an animated WebP (640x360, 10 fps, at most 30 s and 8 MB) with a scene for
each user-facing change, committed with the release beside `script.md`, the
video script it was made from.
`scripts/release.sh` makes it with `scripts/whats-new-video.sh` when it
isn't there yet.

`template/` is the HyperFrames project the script starts from. Its
`compositions/example-s*.html` are the three scenes of 0.10.6's video, with
`{{VERSION}}` for the version: the look every release's scenes follow (a
headline card on the left, a mock of Kanso's UI on the right), drawn
larger in each release's scenes so they read at 640x360. The script
writes each release's scenes and `index.html` in `target/whats-new/<v>/`,
never here.

See docs/RELEASING.md › What's new video.
