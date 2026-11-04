#!/usr/bin/env bash
# Makes sure CMake is on PATH: kanso-audio's `opus` crate builds the libopus
# source it bundles (opusic-sys) with CMake, on every platform. Linux gets
# cmake from scripts/linux-deps.sh.
# Usage: scripts/cmake-deps.sh
#
# Nothing to do when cmake already runs. Otherwise Homebrew on macOS, and on
# Windows the Kitware installer's usual folders (put on PATH for later steps)
# before Chocolatey. A clear error when none of that works.
set -euo pipefail

if command -v cmake >/dev/null 2>&1; then
	echo "cmake: $(cmake --version | head -1)"
	exit 0
fi

case "$(uname -s)" in
Darwin)
	if command -v brew >/dev/null 2>&1; then
		brew install cmake
		exit 0
	fi
	;;
MINGW* | MSYS* | CYGWIN*)
	for dir in "${ProgramFiles:-/c/Program Files}/CMake/bin" "${LOCALAPPDATA:-}/Programs/CMake/bin"; do
		if [ -x "$dir/cmake.exe" ]; then
			echo "Using cmake from $dir."
			if [ -n "${GITHUB_PATH:-}" ]; then
				(cd "$dir" && pwd -W) >>"$GITHUB_PATH"
			fi
			exit 0
		fi
	done
	if command -v choco >/dev/null 2>&1; then
		choco install cmake --installargs 'ADD_CMAKE_TO_PATH=System' -y
		exit 0
	fi
	;;
esac

echo "::error::cmake isn't installed and can't be installed here. libopus (kanso-audio) builds with it; add CMake to the runner image."
exit 1
