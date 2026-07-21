#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-run}"
APP_NAME="KansoMac"
BUNDLE_ID="za.co.codecraftsolutions.KansoMac"
PROJECT_PATH="apps/KansoMac/KansoMac.xcodeproj"
SCHEME="KansoMac"
CONFIGURATION="Release"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DERIVED_DATA_PATH="/tmp/kanso-derived-run"
BUILD_PRODUCTS_DIR="$DERIVED_DATA_PATH/Build/Products/$CONFIGURATION"
BUILT_APP="$BUILD_PRODUCTS_DIR/$APP_NAME.app"
DIST_DIR="$ROOT_DIR/dist"
APP_BUNDLE="$DIST_DIR/$APP_NAME.app"
APP_BINARY="$APP_BUNDLE/Contents/MacOS/$APP_NAME"

usage() {
  echo "usage: $0 [run|--debug|--logs|--telemetry|--verify]" >&2
}

stop_app() {
  pkill -x "$APP_NAME" >/dev/null 2>&1 || true
}

build_app() {
  xcodebuild \
    -project "$ROOT_DIR/$PROJECT_PATH" \
    -scheme "$SCHEME" \
    -configuration "$CONFIGURATION" \
    -derivedDataPath "$DERIVED_DATA_PATH" \
    build \
    CODE_SIGNING_ALLOWED=NO
}

stage_app() {
  rm -rf "$APP_BUNDLE"
  mkdir -p "$DIST_DIR"
  /usr/bin/ditto "$BUILT_APP" "$APP_BUNDLE"
  /usr/bin/codesign --force --deep --sign - "$APP_BUNDLE" >/dev/null
}

open_app() {
  /usr/bin/open -n "$APP_BUNDLE"
}

case "$MODE" in
  run|--debug|debug|--logs|logs|--telemetry|telemetry|--verify|verify)
    stop_app
    build_app
    stage_app
    ;;
  *)
    usage
    exit 2
    ;;
esac

case "$MODE" in
  run)
    open_app
    ;;
  --debug|debug)
    lldb -- "$APP_BINARY"
    ;;
  --logs|logs)
    open_app
    /usr/bin/log stream --info --style compact --predicate "process == \"$APP_NAME\""
    ;;
  --telemetry|telemetry)
    open_app
    /usr/bin/log stream --info --style compact --predicate "subsystem == \"$BUNDLE_ID\""
    ;;
  --verify|verify)
    open_app
    sleep 1
    pgrep -x "$APP_NAME" >/dev/null
    ;;
esac
