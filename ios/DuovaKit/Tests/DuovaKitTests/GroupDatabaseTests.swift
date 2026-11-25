import Foundation
import Testing

@testable import DuovaKit

/// The same group, with every date rounded to the millisecond (SQLite keeps seconds since
/// 1970 as a double, which can move the last bits of a `Date`).
private func rounded(_ group: ExpenseGroup) -> ExpenseGroup {
    func round(_ date: Date) -> Date {
        Date(timeIntervalSince1970: (date.timeIntervalSince1970 * 1000).rounded() / 1000)
    }
    var copy = group
    copy.createdAt = round(group.createdAt)
    copy.expenses = group.expenses.map { expense in
        var expense = expense
        expense.date = round(expense.date)
        return expense
    }
    return copy
}

@Suite("SQLite")
struct GroupDatabaseTests {
    @Test func theLatestSQLite() {
        #expect(Database.sqliteVersion == "3.54.1")
    }

    @Test func aGroupComesBackAsItWent() throws {
        let db = try GroupDatabase.inMemory()
        let group = ExpenseGroup.example
        try db.insert(group)
        let loaded = try db.allGroups()
        #expect(loaded.count == 1)
        #expect(rounded(loaded[0]) == rounded(group))
        #expect(loaded[0].settlements == group.settlements)
    }

    @Test func newestGroupsAndExpensesFirstMembersInOrder() throws {
        let db = try GroupDatabase.inMemory()
        let first = ExpenseGroup(name: "Coloc")
        let second = ExpenseGroup(name: "Week-end")
        try db.insert(first)
        try db.insert(second)
        #expect(try db.allGroups().map(\.name) == ["Week-end", "Coloc"])

        let alice = Member(name: "Alice")
        let bob = Member(name: "Bob")
        try db.insert(alice, in: first.id)
        try db.insert(bob, in: first.id)
        let bread = Expense(title: "Pain", amount: 250, paidBy: alice.id, participants: [bob.id, alice.id])
        let wine = Expense(title: "Vin", amount: 1200, paidBy: bob.id, participants: [alice.id, bob.id])
        try db.save(bread, in: first.id)
        try db.save(wine, in: first.id)

        var coloc = try #require(try db.allGroups().first { $0.id == first.id })
        #expect(coloc.members.map(\.name) == ["Alice", "Bob"])
        #expect(coloc.expenses.map(\.title) == ["Vin", "Pain"])
        #expect(coloc.expenses[1].participants == [bob.id, alice.id])

        // An edited expense keeps its place.
        var edited = bread
        edited.title = "Baguettes"
        edited.amount = 300
        edited.participants = [alice.id]
        try db.save(edited, in: first.id)
        coloc = try #require(try db.allGroups().first { $0.id == first.id })
        #expect(coloc.expenses.map(\.title) == ["Vin", "Baguettes"])
        #expect(coloc.expenses[1].amount == 300)
        #expect(coloc.expenses[1].participants == [alice.id])
    }

    @Test func aMemberWhoPaidOrSharesStays() throws {
        let db = try GroupDatabase.inMemory()
        let group = ExpenseGroup.example
        try db.insert(group)
        let alice = group.members[0]
        #expect(try db.deleteMember(alice.id) == false)

        let dan = Member(name: "Dan")
        try db.insert(dan, in: group.id)
        #expect(try db.deleteMember(dan.id))
        #expect(try db.allGroups()[0].members.map(\.name) == ["Alice", "Bob", "Chloé"])

        // The foreign keys say the same, even without the check.
        #expect(throws: SQLiteError.self) {
            try db.database.run("DELETE FROM members WHERE id = ?", [.text(alice.id.uuidString)])
        }
    }

    @Test func deletingAGroupDeletesEverythingInIt() throws {
        let db = try GroupDatabase.inMemory()
        let group = ExpenseGroup.example
        try db.insert(group)
        try db.deleteExpense(group.expenses[0].id)
        #expect(try db.allGroups()[0].expenses.count == 1)
        try db.deleteGroup(group.id)
        #expect(try db.groupCount() == 0)
        for table in ["members", "expenses", "expense_participants"] {
            #expect(try db.database.query("SELECT count(*) FROM \(table)")[0].int(0) == 0)
        }
    }

    @Test func amountsAreCentsAboveZero() throws {
        let db = try GroupDatabase.inMemory()
        let group = ExpenseGroup.example
        try db.insert(group)
        var expense = group.expenses[0]
        expense.amount = 0
        #expect(throws: SQLiteError.self) { try db.save(expense, in: group.id) }
        // The failed save changed nothing.
        #expect(try db.allGroups()[0].expenses.map(\.amount) == [3000, 9000])
    }

    @Test func renameAGroup() throws {
        let db = try GroupDatabase.inMemory()
        let group = ExpenseGroup(name: "Lyon")
        try db.insert(group)
        try db.update(group: group.id, name: "Lyon 2026", emoji: "🍷")
        let loaded = try db.allGroups()[0]
        #expect(loaded.name == "Lyon 2026")
        #expect(loaded.emoji == "🍷")
    }

    @Test func aFileOnDiskKeepsTheGroups() throws {
        let folder = FileManager.default.temporaryDirectory.appending(path: "duova-\(UUID().uuidString)")
        defer { try? FileManager.default.removeItem(at: folder) }
        let url = folder.appending(path: "duova.sqlite")
        let group = ExpenseGroup.example
        do {
            let db = try GroupDatabase(url: url)
            try db.insert(group)
        }
        let reopened = try GroupDatabase(url: url)
        #expect(try reopened.database.userVersion == GroupDatabase.schemaVersion)
        #expect(try reopened.allGroups().map(rounded) == [rounded(group)])
    }

    @Test func theJSONFileOfTheFirstVersionMovesIn() throws {
        let folder = FileManager.default.temporaryDirectory.appending(path: "duova-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: folder) }
        let json = folder.appending(path: "groups.json")
        let groups = [ExpenseGroup.example, ExpenseGroup(name: "Coloc", members: [Member(name: "Zoé")])]
        try JSONEncoder().encode(groups).write(to: json)

        let db = try GroupDatabase.inMemory()
        #expect(try db.importJSON(at: json) == 2)
        #expect(!FileManager.default.fileExists(atPath: json.path(percentEncoded: false)))
        #expect(try db.allGroups().map(rounded) == groups.map(rounded))
        #expect(try db.importJSON(at: json) == 0)
    }
}
