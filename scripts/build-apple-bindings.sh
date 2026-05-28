#!/usr/bin/env bash
# Build the Kanso engine for Apple platforms and generate Swift bindings.
#
# Produces:
#   target/bindings/swift/kanso_ffi.swift          (Swift glue)
#   target/KansoFFI.xcframework                     (static libs + headers)
#
# The macOS app (apps/KansoMac) consumes both. Re-run whenever the FFI surface
# changes. Requires: Rust targets installed + Xcode command line tools.
set -euo pipefail
cd "$(dirname "$0")/.."

CRATE=kanso-ffi
LIB=libkanso_ffi.a
OUT=target/bindings/swift
mkdir -p "$OUT"

echo "==> Installing Apple Rust targets (no-op if present)"
rustup target add aarch64-apple-darwin x86_64-apple-darwin \
  aarch64-apple-ios aarch64-apple-ios-sim >/dev/null

echo "==> Building staticlib per target (release)"
for t in aarch64-apple-darwin x86_64-apple-darwin aarch64-apple-ios aarch64-apple-ios-sim; do
  cargo build -p "$CRATE" --release --target "$t"
done

echo "==> Universal macOS lib (arm64 + x86_64)"
mkdir -p target/universal-macos
lipo -create \
  "target/aarch64-apple-darwin/release/$LIB" \
  "target/x86_64-apple-darwin/release/$LIB" \
  -output "target/universal-macos/$LIB"

echo "==> Generating Swift bindings from the built library"
# Library mode: read the FFI metadata straight out of the compiled artifact.
cargo run -q -p "$CRATE" --bin uniffi-bindgen -- generate \
  --library "target/aarch64-apple-darwin/release/$LIB" \
  --language swift --out-dir "$OUT"

# UniFFI emits a `<name>FFI.modulemap`; xcframework expects `module.modulemap`.
cp "$OUT/kanso_ffiFFI.modulemap" "$OUT/module.modulemap"

echo "==> Assembling KansoFFI.xcframework"
rm -rf target/KansoFFI.xcframework
xcodebuild -create-xcframework \
  -library "target/universal-macos/$LIB"          -headers "$OUT" \
  -library "target/aarch64-apple-ios/release/$LIB" -headers "$OUT" \
  -library "target/aarch64-apple-ios-sim/release/$LIB" -headers "$OUT" \
  -output target/KansoFFI.xcframework

echo "==> Done."
echo "    Swift glue:    $OUT/kanso_ffi.swift"
echo "    xcframework:   target/KansoFFI.xcframework"
echo "    Add both to apps/KansoMac in Xcode (see apps/KansoMac/README.md)."
