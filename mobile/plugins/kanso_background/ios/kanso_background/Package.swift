// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "kanso_background",
    platforms: [
        .iOS("17.0")
    ],
    products: [
        .library(name: "kanso-background", targets: ["kanso_background"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
    ],
    targets: [
        .target(
            name: "kanso_background",
            dependencies: [
                .product(name: "FlutterFramework", package: "FlutterFramework")
            ]
        )
    ]
)
