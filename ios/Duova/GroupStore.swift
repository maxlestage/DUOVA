import DuovaKit
import Foundation
import Observation

/// Every group, kept in an SQLite database on the device (the app has no account and no
/// server). Each change is written to the database first, then `groups` is read back
/// from it, so the screens always show what is saved.
@Observable
final class GroupStore {
    private(set) var groups: [ExpenseGroup] = []

    /// The last error of the database, shown to the person.
    var lastError: String?

    private let database: GroupDatabase?

    init(database: GroupDatabase?) {
        self.database = database
        reload()
    }

    /// The database of the app, in Application Support. The groups of the JSON file of the
    /// first version move into it once.
    static func onDevice() -> GroupStore {
        let folder = URL.applicationSupportDirectory.appending(path: "Duova", directoryHint: .isDirectory)
        do {
            try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
            // The database, its journal and the files SQLite creates next to it can only
            // be read once the iPhone has been unlocked.
            try FileManager.default.setAttributes(
                [.protectionKey: FileProtectionType.completeUnlessOpen],
                ofItemAtPath: folder.path(percentEncoded: false)
            )
            let database = try GroupDatabase(url: folder.appending(path: "duova.sqlite"))
            try database.importJSON(at: URL.applicationSupportDirectory.appending(path: "groups.json"))
            return GroupStore(database: database)
        } catch {
            let store = GroupStore(database: nil)
            store.lastError = "La base de données n'a pas pu s'ouvrir : \(error)"
            return store
        }
    }

    /// A store in memory, for the previews.
    static func preview(_ groups: [ExpenseGroup] = []) -> GroupStore {
        let database = try? GroupDatabase.inMemory()
        for group in groups { try? database?.insert(group) }
        return GroupStore(database: database)
    }

    // MARK: Groups

    @discardableResult
    func createGroup(named name: String, emoji: String = "🧾") -> ExpenseGroup {
        let group = ExpenseGroup(name: name, emoji: emoji)
        write { try $0.insert(group) }
        return group
    }

    func delete(_ id: ExpenseGroup.ID) {
        write { try $0.deleteGroup(id) }
    }

    func group(_ id: ExpenseGroup.ID) -> ExpenseGroup? {
        groups.first { $0.id == id }
    }

    // MARK: Members and expenses

    func addMember(named name: String, to id: ExpenseGroup.ID) {
        write { try $0.insert(Member(name: name), in: id) }
    }

    /// Removes a member, unless they paid or share an expense: returns `false` then.
    @discardableResult
    func removeMember(_ member: Member.ID, from id: ExpenseGroup.ID) -> Bool {
        guard let group = group(id), !group.isInvolved(member) else { return false }
        var removed = false
        write { removed = try $0.deleteMember(member) }
        return removed
    }

    func save(_ expense: Expense, in id: ExpenseGroup.ID) {
        write { try $0.save(expense, in: id) }
    }

    func deleteExpense(_ expense: Expense.ID, in id: ExpenseGroup.ID) {
        write { try $0.deleteExpense(expense) }
    }

    // MARK: Storage

    private func write(_ change: (GroupDatabase) throws -> Void) {
        guard let database else { return }
        do {
            try change(database)
        } catch {
            lastError = "La modification n'a pas pu être enregistrée : \(error)"
        }
        reload()
    }

    private func reload() {
        guard let database else { return }
        do {
            groups = try database.allGroups()
        } catch {
            lastError = "Les groupes n'ont pas pu être lus : \(error)"
        }
    }
}
