// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Iridescence",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "IridescenceCore",
            targets: ["IridescenceCore"]
        ),
        .executable(
            name: "Iridescence",
            targets: ["IridescenceApp"]
        )
    ],
    targets: [
        .target(
            name: "IridescenceCore",
            path: "Sources/IridescenceCore",
            linkerSettings: [
                .linkedFramework("AppKit"),
                .linkedFramework("SwiftUI"),
                .linkedFramework("PDFKit"),
                .linkedFramework("Vision"),
                .linkedFramework("UniformTypeIdentifiers"),
                .linkedFramework("QuartzCore")
            ]
        ),
        .executableTarget(
            name: "IridescenceApp",
            dependencies: ["IridescenceCore"],
            path: "Sources/IridescenceApp"
        ),
        .testTarget(
            name: "IridescenceTests",
            dependencies: ["IridescenceCore"],
            path: "Tests/IridescenceTests"
        )
    ]
)
