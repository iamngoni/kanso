#!/usr/bin/env bash
# Builds the Kanso iOS spike for the simulator and launches it.
#   ./build.sh [--release] [--no-run]
# SIM=<udid> picks a simulator when several are booted.
# Needs: rustup target add aarch64-apple-ios-sim, brew install xcodegen.
set -euo pipefail
cd "$(dirname "$0")"
export IPHONEOS_DEPLOYMENT_TARGET=16.0

profile=debug; flags=(); run=1
for a in "$@"; do
  case "$a" in
    --release) profile=release; flags=(--release) ;;
    --no-run) run=0 ;;
    *) echo "unknown option $a" >&2; exit 1 ;;
  esac
done

target_dir="${CARGO_TARGET_DIR:-$PWD/target}"
cargo build --lib --target aarch64-apple-ios-sim "${flags[@]}"

cd ios
xcodegen generate --spec project.yml
configuration=Debug; [ "$profile" = release ] && configuration=Release
xcodebuild -project KansoSpike.xcodeproj -scheme KansoSpike -sdk iphonesimulator \
  -configuration "$configuration" -derivedDataPath build \
  KANSO_STATIC_LIB="$target_dir/aarch64-apple-ios-sim/$profile/libkanso_mobile.a" build

app="$PWD/build/Build/Products/$configuration-iphonesimulator/KansoSpike.app"
echo "$app"
if [ "$run" = 1 ]; then
  xcrun simctl install "${SIM:-booted}" "$app"
  xcrun simctl launch --console-pty "${SIM:-booted}" com.iamngoni.kanso.spike
fi
