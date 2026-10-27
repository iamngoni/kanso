// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "kanso_lock",
    platforms: [
        .iOS("17.0")
    ],
    products: [
        .library(name: "kanso-lock", targets: ["kanso_lock"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
    ],
    targets: [
        .target(
            name: "kanso_lock",
            dependencies: [
                .product(name: "FlutterFramework", package: "FlutterFramework")
            ]
        )
    ]
)
