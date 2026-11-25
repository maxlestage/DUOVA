import SwiftUI

/// Duova: « Qui doit quoi ? ». Groups, members and expenses stay on the device, in SQLite.
@main
struct DuovaApp: App {
    @State private var store = GroupStore.onDevice()

    var body: some Scene {
        WindowGroup {
            GroupListView()
                .environment(store)
                .tint(Theme.coral)
        }
    }
}
