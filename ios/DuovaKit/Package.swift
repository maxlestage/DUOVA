// swift-tools-version: 6.4
// The arithmetic and the models of Duova, shared by the iOS app and its tests: amounts
// in cents, equal shares without losing a cent, balances and the shortest list of
// transfers. The same rules as the website (`site/src/split.rs`).

import PackageDescription

let package = Package(
    name: "DuovaKit",
    defaultLocalization: "fr",
    platforms: [.iOS("27.0"), .macOS("27.0")],
    products: [
        .library(name: "DuovaKit", targets: ["DuovaKit"])
    ],
    targets: [
        .target(name: "DuovaKit"),
        .testTarget(name: "DuovaKitTests", dependencies: ["DuovaKit"]),
    ],
    swiftLanguageModes: [.v6]
)
