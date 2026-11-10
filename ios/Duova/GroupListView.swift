import DuovaKit
import SwiftUI

/// The first screen: every group, and a button to create one.
struct GroupListView: View {
    @Environment(GroupStore.self) private var store
    @State private var path: [ExpenseGroup.ID] = []
    @State private var isCreating = false
    @State private var newName = ""
    @State private var pendingDeletion: ExpenseGroup?

    var body: some View {
        NavigationStack(path: $path) {
            Group {
                if store.groups.isEmpty {
                    ContentUnavailableView {
                        Label("Aucun groupe", systemImage: "person.3.sequence.fill")
                            .symbolEffect(.pulse)
                    } description: {
                        Text("Créez un groupe pour un week-end, une coloc ou des vacances.")
                    } actions: {
                        Button("Créer un groupe") { isCreating = true }
                            .buttonStyle(.borderedProminent)
                    }
                } else {
                    List {
                        ForEach(store.groups) { group in
                            NavigationLink(value: group.id) {
                                GroupRow(group: group)
                            }
                            .swipeActions {
                                Button("Supprimer", systemImage: "trash", role: .destructive) {
                                    pendingDeletion = group
                                }
                            }
                        }
                    }
                    .animation(.default, value: store.groups.map(\.id))
                }
            }
            .navigationTitle("Qui doit quoi ?")
            .navigationDestination(for: ExpenseGroup.ID.self) { id in
                GroupDetailView(groupID: id)
            }
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button("Nouveau groupe", systemImage: "plus") { isCreating = true }
                }
            }
            .alert("Nouveau groupe", isPresented: $isCreating) {
                TextField("Ex. Week-end à Lyon", text: $newName)
                Button("Créer") {
                    let name = newName.trimmingCharacters(in: .whitespacesAndNewlines)
                    newName = ""
                    guard !name.isEmpty else { return }
                    let group = store.createGroup(named: name)
                    path.append(group.id)
                }
                Button("Annuler", role: .cancel) { newName = "" }
            }
            .confirmationDialog(
                "Supprimer « \(pendingDeletion?.name ?? "") » et toutes ses dépenses ?",
                isPresented: Binding(
                    get: { pendingDeletion != nil },
                    set: { if !$0 { pendingDeletion = nil } }
                ),
                titleVisibility: .visible
            ) {
                Button("Supprimer", role: .destructive) {
                    if let group = pendingDeletion {
                        withAnimation { store.delete(group.id) }
                    }
                    pendingDeletion = nil
                }
            }
        }
    }
}

private struct GroupRow: View {
    let group: ExpenseGroup

    var body: some View {
        HStack(spacing: 14) {
            Text(group.emoji)
                .font(.title2)
                .frame(width: 46, height: 46)
                .background(Theme.violet.opacity(0.18), in: .rect(cornerRadius: 14))
            VStack(alignment: .leading, spacing: 2) {
                Text(group.name)
                    .font(.headline)
                Text("\(group.members.count) membres · \(group.expenses.count) dépenses")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 8)
            AmountText(cents: group.total)
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(Theme.gold)
        }
        .padding(.vertical, 4)
        .accessibilityElement(children: .combine)
    }
}

#Preview {
    GroupListView()
        .environment(GroupStore(url: URL.temporaryDirectory.appending(path: "preview.json")))
}
