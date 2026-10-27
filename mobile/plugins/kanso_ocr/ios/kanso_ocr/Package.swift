// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "kanso_ocr",
    platforms: [
        .iOS("17.0")
    ],
    products: [
        .library(name: "kanso-ocr", targets: ["kanso_ocr"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
    ],
    targets: [
        // Vision and ImageIO only: no Flutter, so it is testable alone.
        .target(name: "KansoOcrCore"),
        .target(
            name: "kanso_ocr",
            dependencies: [
                "KansoOcrCore",
                .product(name: "FlutterFramework", package: "FlutterFramework"),
            ]
        ),
        .testTarget(name: "KansoOcrCoreTests", dependencies: ["KansoOcrCore"]),
    ]
)
