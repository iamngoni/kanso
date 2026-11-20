#!/usr/bin/env bash
# Checks the store listing text in mobile/store/ against the stores' limits,
# and that it names no other notes app (#48 step 13).
# Usage: scripts/mobile-store-check.sh
#
# App Store: name and subtitle 30 characters, promotional text 170,
# description 4000, keywords 100, what's new 4000.
# Play: title 30, short description 80, full description 4000, release notes 500.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python3 -I - "$root/mobile/store" <<'PY'
import re
import sys
from pathlib import Path

store = Path(sys.argv[1])
limits = {
    "app-store/name.txt": 30,
    "app-store/subtitle.txt": 30,
    "app-store/promotional-text.txt": 170,
    "app-store/description.txt": 4000,
    "app-store/keywords.txt": 100,
    "app-store/whats-new.txt": 4000,
    "play/title.txt": 30,
    "play/short-description.txt": 80,
    "play/full-description.txt": 4000,
    "play/whatsnew/whatsnew-en-US": 500,
}
# Other notes apps are not named in listings (store rules and Kanso's own).
others = re.compile(
    r"obsidian|evernote|notion|bear\b|apple notes|google keep|onenote|joplin|logseq|roam|simplenote|standard notes",
    re.I,
)
failures = []
for name, limit in limits.items():
    path = store / name
    if not path.is_file():
        failures.append(f"{name} is missing")
        continue
    text = path.read_text().rstrip("\n")
    if len(text) > limit:
        failures.append(f"{name} is {len(text)} characters; the limit is {limit}")
    if not text.strip():
        failures.append(f"{name} is empty")
for path in sorted(store.rglob("*")):
    if path.is_file() and path.suffix in {".txt", ""} and "screenshots" not in path.parts:
        hit = others.search(path.read_text())
        if hit:
            failures.append(f"{path.relative_to(store)} names another notes app ('{hit.group(0)}')")
keywords = (store / "app-store/keywords.txt").read_text().strip()
if " " in keywords:
    failures.append("app-store/keywords.txt has spaces; they cost characters (comma only)")
if failures:
    for failure in failures:
        print(f"::error::{failure}")
    sys.exit(1)
print(f"store listing: {len(limits)} texts are within the stores' limits.")
PY
