// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "kanso_auth_session",
    platforms: [
        .iOS("17.0")
    ],
    products: [
        .library(name: "kanso-auth-session", targets: ["kanso_auth_session"])
    ],
    dependencies: [
        .package(name: "FlutterFramework", path: "../FlutterFramework")
    ],
    targets: [
        .target(
            name: "kanso_auth_session",
            dependencies: [
                .product(name: "FlutterFramework", package: "FlutterFramework")
            ]
        )
    ]
)
