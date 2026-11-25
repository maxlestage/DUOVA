// swift-tools-version: 6.4
// The arithmetic, the models and the storage of Duova, shared by the iOS app and its
// tests: amounts in cents, equal shares without losing a cent, balances, the shortest
// list of transfers (the same rules as the website, `site/src/split.rs`), and the
// SQLite database that keeps the groups on the device.

import PackageDescription

let package = Package(
    name: "DuovaKit",
    defaultLocalization: "fr",
    platforms: [.iOS("27.0"), .macOS("27.0")],
    products: [
        .library(name: "DuovaKit", targets: ["DuovaKit"])
    ],
    targets: [
        // SQLite 3.54.1, the amalgamation from sqlite.org (see Sources/CSQLite/README.md):
        // the same, current version on every device, whatever the iOS version ships.
        .target(
            name: "CSQLite",
            exclude: ["README.md", "update.sh"],
            cSettings: [
                .define("SQLITE_THREADSAFE", to: "2"),
                .define("SQLITE_DQS", to: "0"),
                .define("SQLITE_DEFAULT_FOREIGN_KEYS", to: "1"),
                .define("SQLITE_DEFAULT_MEMSTATUS", to: "0"),
                .define("SQLITE_DEFAULT_WAL_SYNCHRONOUS", to: "1"),
                .define("SQLITE_LIKE_DOESNT_MATCH_BLOBS"),
                .define("SQLITE_OMIT_DEPRECATED"),
                .define("SQLITE_OMIT_LOAD_EXTENSION"),
                .define("SQLITE_OMIT_SHARED_CACHE"),
                .define("SQLITE_USE_ALLOCA"),
                .define("SQLITE_ENABLE_API_ARMOR"),
            ],
            linkerSettings: [.linkedLibrary("m", .when(platforms: [.linux]))]
        ),
        .target(name: "DuovaKit", dependencies: ["CSQLite"]),
        .testTarget(name: "DuovaKitTests", dependencies: ["DuovaKit"]),
    ],
    swiftLanguageModes: [.v6]
)
