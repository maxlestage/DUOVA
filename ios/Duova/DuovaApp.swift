import SwiftUI

/// Duova: « Qui doit quoi ? ». Groups, members and expenses stay on the device.
@main
struct DuovaApp: App {
    @State private var store = GroupStore()

    var body: some Scene {
        WindowGroup {
            GroupListView()
                .environment(store)
                .tint(Theme.coral)
        }
    }
}
