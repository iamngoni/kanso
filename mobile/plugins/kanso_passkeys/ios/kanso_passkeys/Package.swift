// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "kanso_passkeys",
    platforms: [
        .iOS("17.0")
    ],
    products: [
        .library(name: "kanso-passkeys", targets: ["kanso_passkeys"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
    ],
    targets: [
        .target(
            name: "kanso_passkeys",
            dependencies: [
                .product(name: "FlutterFramework", package: "FlutterFramework")
            ]
        )
    ]
)
