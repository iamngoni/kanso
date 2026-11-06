#!/usr/bin/env bash
# Builds the core (#48) for iOS and Android without running it: `cargo check`
# for the five targets, clippy, and a look at what each phone's dependency
# tree must not hold.
# Usage: scripts/mobile-check.sh [target...]   (default: all five)
#
# Needs the Rust targets (`rustup target add ...`) and, for Android, the NDK
# (see scripts/mobile/env.sh). CI runs it as the `engine-mobile` job.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=mobile/env.sh
. "$here/mobile/env.sh"
cd "$KANSO_ROOT"

if [ "$#" -gt 0 ]; then
	targets=("$@")
else
	targets=(aarch64-apple-ios aarch64-apple-ios-sim aarch64-linux-android armv7-linux-androideabi x86_64-linux-android)
fi

# The crates the facade is built on, the facade (kanso-mobile) and the
# library the app loads (kanso-mobile-bridge). kanso-sync and kanso-ai leave
# their desktop features off: the system Keychain and agents on this computer.
with_defaults=(-p kanso-core -p kanso-store -p kanso-engine -p kanso-render -p kanso-collab -p kanso-import -p kanso-tls -p kanso-mobile -p kanso-mobile-bridge)
without_defaults=(-p kanso-sync -p kanso-ai)

for t in "${targets[@]}"; do
	echo "== cargo check ($t)"
	# kanso-collab depends on kanso-sync with its default features, so the
	# two lists are checked apart; the second, with the features off, is the
	# build the phones get.
	cargo check --locked --target "$t" "${with_defaults[@]}"
	cargo check --locked --target "$t" --no-default-features "${without_defaults[@]}"
done

# No window system, desktop Objective-C bindings (the old `objc` crate,
# AppKit, Cocoa), desktop Keychain or agent connection in any crate's tree
# for a phone. `cargo tree` lists what the target builds. The `objc2-*`
# framework crates cpal uses for CoreAudio and AVFAudio are allowed: the
# phones record and play audio through kanso-audio.
banned_pattern='gpui|[─ ]objc v|objc2-app-kit|cocoa|keyring|kanso-acp'
for t in "${targets[@]}"; do
	case "$t" in
	aarch64-apple-ios | aarch64-linux-android) ;;
	*) continue ;;
	esac
	for crate in kanso-core kanso-store kanso-engine kanso-render kanso-collab kanso-import kanso-tls kanso-mobile kanso-mobile-bridge; do
		banned="$(cargo tree -p "$crate" --locked --target "$t" -e normal | grep -Ei "$banned_pattern" || true)"
		if [ -n "$banned" ]; then
			echo "$banned"
			echo "::error::$crate's $t dependencies include a window system, Objective-C bindings, a Keychain or an agent connection."
			exit 1
		fi
	done
	for crate in kanso-sync kanso-ai; do
		banned="$(cargo tree -p "$crate" --locked --target "$t" -e normal --no-default-features | grep -Ei "$banned_pattern" || true)"
		if [ -n "$banned" ]; then
			echo "$banned"
			echo "::error::$crate's $t dependencies include a window system, Objective-C bindings, a Keychain or an agent connection."
			exit 1
		fi
	done
done

# Catches what only a phone's build compiles (cfg-gated code, unused imports).
echo "== cargo clippy (aarch64-apple-ios-sim, aarch64-linux-android)"
for t in aarch64-apple-ios-sim aarch64-linux-android; do
	cargo clippy --locked --target "$t" "${with_defaults[@]}" -- -D warnings
	cargo clippy --locked --target "$t" --no-default-features "${without_defaults[@]}" -- -D warnings
done
