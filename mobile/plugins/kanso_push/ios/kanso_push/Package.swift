// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "kanso_push",
    platforms: [
        .iOS("13.0")
    ],
    products: [
        .library(name: "kanso-push", targets: ["kanso_push"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
    ],
    targets: [
        .target(
            name: "kanso_push",
            dependencies: [
                .product(name: "FlutterFramework", package: "FlutterFramework")
            ]
        )
    ]
)
