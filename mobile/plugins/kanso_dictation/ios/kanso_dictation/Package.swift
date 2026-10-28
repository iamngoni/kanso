// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "kanso_dictation",
    platforms: [
        .iOS("17.0")
    ],
    products: [
        .library(name: "kanso-dictation", targets: ["kanso_dictation"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
    ],
    targets: [
        .target(
            name: "kanso_dictation",
            dependencies: [
                .product(name: "FlutterFramework", package: "FlutterFramework")
            ]
        )
    ]
)
