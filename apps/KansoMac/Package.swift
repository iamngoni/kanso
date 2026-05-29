// swift-tools-version: 6.0
import PackageDescription

// Builds the macOS shell against the engine's prebuilt host static library.
//
// Prereqs (one-time): build the FFI static lib and generate bindings:
//   cargo build -p kanso-ffi
//   cargo run -p kanso-ffi --bin uniffi-bindgen -- generate \
//     --library target/debug/libkanso_ffi.dylib --language swift \
//     --out-dir target/bindings/swift
// then refresh the copied glue:
//   cp target/bindings/swift/kanso_ffi.swift apps/KansoMac/Sources/KansoMac/
//   cp target/bindings/swift/kanso_ffiFFI.h  apps/KansoMac/Sources/kanso_ffiFFI/include/
//
// Then: cd apps/KansoMac && swift build   (or `swift run`)
//
// For a shippable .app / iOS, build the xcframework via
// scripts/build-apple-bindings.sh and use an Xcode app target instead.
let kansoTarget = "/Users/modestnerd/Developer/Projects/kanso/target/debug"

let package = Package(
    name: "KansoMac",
    platforms: [.macOS(.v14)],
    targets: [
        // The UniFFI C header module (`import kanso_ffiFFI`).
        .target(name: "kanso_ffiFFI"),
        .executableTarget(
            name: "KansoMac",
            dependencies: ["kanso_ffiFFI"],
            linkerSettings: [
                .unsafeFlags(["-L\(kansoTarget)", "-lkanso_ffi"]),
                .linkedFramework("CoreFoundation"),
                .linkedFramework("Security"),
                .linkedFramework("SystemConfiguration"),
                .linkedLibrary("c++"),
            ]
        ),
    ]
)
