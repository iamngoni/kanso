// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "kanso_print",
    platforms: [
        .iOS("17.0")
    ],
    products: [
        .library(name: "kanso-print", targets: ["kanso_print"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
    ],
    targets: [
        .target(
            name: "kanso_print",
            dependencies: [
                .product(name: "FlutterFramework", package: "FlutterFramework")
            ]
        )
    ]
)
