import DuovaKit
import Foundation
import Observation

/// Every group, saved as JSON in Application Support after each change (the app has
/// no account and no server).
@Observable
final class GroupStore {
    private(set) var groups: [ExpenseGroup] = []

    private let url: URL

    init(url: URL? = nil) {
        self.url = url ?? GroupStore.defaultURL
        load()
    }

    static var defaultURL: URL {
        URL.applicationSupportDirectory.appending(path: "groups.json")
    }

    // MARK: Groups

    @discardableResult
    func createGroup(named name: String, emoji: String = "🧾") -> ExpenseGroup {
        let group = ExpenseGroup(name: name, emoji: emoji)
        groups.insert(group, at: 0)
        save()
        return group
    }

    func deleteGroups(at offsets: IndexSet) {
        groups.remove(atOffsets: offsets)
        save()
    }

    func delete(_ id: ExpenseGroup.ID) {
        groups.removeAll { $0.id == id }
        save()
    }

    func group(_ id: ExpenseGroup.ID) -> ExpenseGroup? {
        groups.first { $0.id == id }
    }

    /// Changes a group in place, then saves.
    func update(_ id: ExpenseGroup.ID, _ change: (inout ExpenseGroup) -> Void) {
        guard let index = groups.firstIndex(where: { $0.id == id }) else { return }
        change(&groups[index])
        save()
    }

    // MARK: Members and expenses

    func addMember(named name: String, to id: ExpenseGroup.ID) {
        update(id) { $0.members.append(Member(name: name)) }
    }

    /// Removes a member, unless they paid or share an expense: returns `false` then.
    @discardableResult
    func removeMember(_ member: Member.ID, from id: ExpenseGroup.ID) -> Bool {
        guard let group = group(id), !group.isInvolved(member) else { return false }
        update(id) { $0.members.removeAll { $0.id == member } }
        return true
    }

    func save(_ expense: Expense, in id: ExpenseGroup.ID) {
        update(id) { group in
            if let index = group.expenses.firstIndex(where: { $0.id == expense.id }) {
                group.expenses[index] = expense
            } else {
                group.expenses.insert(expense, at: 0)
            }
        }
    }

    func deleteExpense(_ expense: Expense.ID, in id: ExpenseGroup.ID) {
        update(id) { $0.expenses.removeAll { $0.id == expense } }
    }

    // MARK: Storage

    private func load() {
        guard let data = try? Data(contentsOf: url) else { return }
        groups = (try? JSONDecoder().decode([ExpenseGroup].self, from: data)) ?? []
    }

    private func save() {
        do {
            try FileManager.default.createDirectory(
                at: url.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            let data = try JSONEncoder().encode(groups)
            try data.write(to: url, options: [.atomic, .completeFileProtection])
        } catch {
            assertionFailure("Duova: could not save the groups: \(error)")
        }
    }
}
