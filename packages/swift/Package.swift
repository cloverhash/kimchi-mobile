// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "KimchiMobile",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "KimchiMobile",
            targets: ["KimchiMobile"]
        ),
    ],
    targets: [
        .target(
            name: "KimchiMobile",
            dependencies: ["KimchiFfi"],
            path: "Sources/KimchiMobile"
        ),
        .binaryTarget(
            name: "KimchiFfi",
            path: "../../ios-output/KimchiFfi.xcframework"
        ),
    ]
)
