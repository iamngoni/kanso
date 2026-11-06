#!/usr/bin/env bash
# The server CLI must not load a desktop, graphics or security-key runtime.
set -euo pipefail

binary=${1:?usage: check-headless-ldd.sh BINARY}
if ! command -v ldd >/dev/null 2>&1; then
  echo "check-headless-ldd: ldd is required" >&2
  exit 2
fi
deps=$(ldd "$binary")
if grep -Eiq 'lib(vulkan|wayland|X11|fido2)' <<<"$deps"; then
  echo "check-headless-ldd: forbidden display, graphics, or libfido2 dependency in $binary:" >&2
  grep -Ei 'lib(vulkan|wayland|X11|fido2)' <<<"$deps" >&2
  exit 1
fi
echo "check-headless-ldd: no Wayland, X11, Vulkan, or libfido2 dependency in $binary"
