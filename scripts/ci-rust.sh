#!/usr/bin/env bash
# Gets the toolchain pinned in rust-toolchain.toml onto a CI runner.
# Usage: scripts/ci-rust.sh
#
# The self-hosted runners don't load a login shell, so ~/.cargo/bin is often
# off PATH even where Rust is installed, and fresh runners have no rustup at
# all. This puts ~/.cargo/bin on PATH for later steps, installs rustup for
# this user (no sudo) when it's missing, then installs the pinned toolchain.
#
# Runners on one machine share this user's rustup, and two jobs installing a
# toolchain at once can leave it broken (on the Windows runner, without
# cargo). So one job at a time installs, under a lock in the rustup home, and
# a toolchain whose tools don't run is reinstalled.
set -euo pipefail

cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin"
export PATH="$cargo_bin:$PATH"
echo "$cargo_bin" >>"$GITHUB_PATH"

# Whoever creates the lock directory goes first. A lock older than 30 minutes
# belongs to a job that died holding it.
rustup_home="${RUSTUP_HOME:-$HOME/.rustup}"
lock="$rustup_home/ci-install.lock"
mkdir -p "$rustup_home"
waited=0
until mkdir "$lock" 2>/dev/null; do
	if [ -n "$(find "$lock" -maxdepth 0 -mmin +30 2>/dev/null)" ]; then
		echo "Removing a toolchain install lock left by a job that died."
		rm -rf "$lock"
		continue
	fi
	if [ $((waited % 60)) -eq 0 ]; then
		echo "Another job is installing Rust on this machine; waiting for it."
	fi
	sleep 5
	waited=$((waited + 5))
done
trap 'rm -rf "$lock"' EXIT

if ! command -v rustup >/dev/null 2>&1; then
	echo "No rustup on this runner; installing it for $(id -un)."
	case "$(uname -s)" in
	MINGW* | MSYS* | CYGWIN*)
		curl --retry 5 --retry-all-errors --proto '=https' --tlsv1.2 -sSfL -o "$RUNNER_TEMP/rustup-init.exe" \
			https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
		"$RUNNER_TEMP/rustup-init.exe" -y --no-modify-path --default-toolchain none --profile minimal
		;;
	*)
		curl --retry 5 --retry-all-errors --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
			sh -s -- -y --no-modify-path --default-toolchain none --profile minimal
		;;
	esac
fi

toolchain_works() {
	cargo --version && rustc --version && cargo clippy --version && rustfmt --version
}

# With no arguments, rustup installs what rust-toolchain.toml pins, with its
# components, and leaves an installed toolchain as it is.
rustup toolchain install
if ! toolchain_works >/dev/null 2>&1; then
	toolchain="$(rustup show active-toolchain | cut -d' ' -f1)"
	echo "The $toolchain toolchain on this runner is broken; reinstalling it."
	rustup toolchain uninstall "$toolchain"
	rustup toolchain install
fi
toolchain_works
