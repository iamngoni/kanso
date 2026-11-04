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
	# ProgramFiles and LOCALAPPDATA are Windows paths (C:\...); cygpath
	# turns them into ones Git Bash's tests understand.
	unix() { cygpath -u "$1" 2>/dev/null || printf '%s' "$1"; }
	program_files=$(unix "${ProgramFiles:-C:\Program Files}")
	local_app_data=$(unix "${LOCALAPPDATA:-}")
	use_dir() {
		for dir in "$program_files/CMake/bin" "$local_app_data/Programs/CMake/bin"; do
			if [ -x "$dir/cmake.exe" ]; then
				echo "Using cmake from $dir."
				# Later steps start new shells: put it on their PATH too.
				if [ -n "${GITHUB_PATH:-}" ]; then
					(cd "$dir" && pwd -W) >>"$GITHUB_PATH"
				fi
				return 0
			fi
		done
		return 1
	}
	if use_dir; then
		exit 0
	fi
	if command -v choco >/dev/null 2>&1; then
		choco install cmake --installargs 'ADD_CMAKE_TO_PATH=System' -y
		# The installer's System PATH only reaches new logins, not this
		# runner's later steps.
		if use_dir; then
			exit 0
		fi
	fi
	;;
esac

echo "::error::cmake isn't installed and can't be installed here. libopus (kanso-audio) builds with it; add CMake to the runner image."
exit 1
