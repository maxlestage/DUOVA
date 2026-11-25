import Foundation

/// The groups, their members and their expenses, in an SQLite database on the device.
///
/// One table per kind of thing, linked by foreign keys: deleting a group deletes its
/// members and expenses, and a member who paid or shares an expense cannot be deleted.
/// Amounts are integer cents (`CHECK (amount > 0)`), ids are UUIDs, dates are seconds
/// since 1970. Groups come newest first, members in the order they were added, expenses
/// newest first (an edited expense keeps its place).
public final class GroupDatabase {
    public let database: Database

    /// The version of the schema written by this code.
    public static let schemaVersion = 1

    /// Opens (or creates) the database at `url`.
    public convenience init(url: URL) throws {
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try self.init(database: Database(path: url.path(percentEncoded: false)), journal: "WAL")
    }

    /// A database that lives in memory only (tests, previews).
    public static func inMemory() throws -> GroupDatabase {
        try GroupDatabase(database: Database(path: ":memory:"), journal: "MEMORY")
    }

    private init(database: Database, journal: String) throws {
        self.database = database
        try database.execute("PRAGMA journal_mode = \(journal); PRAGMA foreign_keys = ON;")
        try migrate()
    }

    private func migrate() throws {
        let version = try database.userVersion
        guard version < Self.schemaVersion else { return }
        try database.transaction {
            if version < 1 {
                try database.execute(
                    """
                    CREATE TABLE groups (
                        seq INTEGER PRIMARY KEY,
                        id TEXT NOT NULL UNIQUE,
                        name TEXT NOT NULL,
                        emoji TEXT NOT NULL,
                        created_at REAL NOT NULL
                    ) STRICT;
                    CREATE TABLE members (
                        seq INTEGER PRIMARY KEY,
                        id TEXT NOT NULL UNIQUE,
                        group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
                        name TEXT NOT NULL
                    ) STRICT;
                    CREATE INDEX members_by_group ON members(group_id, seq);
                    CREATE TABLE expenses (
                        seq INTEGER PRIMARY KEY,
                        id TEXT NOT NULL UNIQUE,
                        group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
                        title TEXT NOT NULL,
                        amount INTEGER NOT NULL CHECK (amount > 0),
                        paid_by TEXT NOT NULL REFERENCES members(id),
                        date REAL NOT NULL
                    ) STRICT;
                    CREATE INDEX expenses_by_group ON expenses(group_id, seq);
                    CREATE INDEX expenses_by_payer ON expenses(paid_by);
                    CREATE TABLE expense_participants (
                        expense_id TEXT NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
                        member_id TEXT NOT NULL REFERENCES members(id),
                        position INTEGER NOT NULL,
                        PRIMARY KEY (expense_id, member_id)
                    ) STRICT, WITHOUT ROWID;
                    CREATE INDEX participants_by_member ON expense_participants(member_id);
                    """
                )
            }
            try database.setUserVersion(Self.schemaVersion)
        }
    }

    // MARK: Reading

    /// Every group, with its members and expenses.
    public func allGroups() throws -> [ExpenseGroup] {
        var members: [UUID: [Member]] = [:]
        for row in try database.query("SELECT id, group_id, name FROM members ORDER BY seq") {
            guard let id = UUID(uuidString: row.text(0)), let group = UUID(uuidString: row.text(1)) else { continue }
            members[group, default: []].append(Member(id: id, name: row.text(2)))
        }

        var participants: [UUID: [Member.ID]] = [:]
        for row in try database.query(
            "SELECT expense_id, member_id FROM expense_participants ORDER BY expense_id, position"
        ) {
            guard let expense = UUID(uuidString: row.text(0)), let member = UUID(uuidString: row.text(1)) else { continue }
            participants[expense, default: []].append(member)
        }

        var expenses: [UUID: [Expense]] = [:]
        for row in try database.query(
            "SELECT id, group_id, title, amount, paid_by, date FROM expenses ORDER BY seq DESC"
        ) {
            guard let id = UUID(uuidString: row.text(0)),
                let group = UUID(uuidString: row.text(1)),
                let paidBy = UUID(uuidString: row.text(4))
            else { continue }
            expenses[group, default: []].append(
                Expense(
                    id: id,
                    title: row.text(2),
                    amount: row.int(3),
                    paidBy: paidBy,
                    participants: participants[id] ?? [],
                    date: Date(timeIntervalSince1970: row.double(5))
                )
            )
        }

        return try database.query("SELECT id, name, emoji, created_at FROM groups ORDER BY seq DESC").compactMap { row in
            guard let id = UUID(uuidString: row.text(0)) else { return nil }
            return ExpenseGroup(
                id: id,
                name: row.text(1),
                emoji: row.text(2),
                members: members[id] ?? [],
                expenses: expenses[id] ?? [],
                createdAt: Date(timeIntervalSince1970: row.double(3))
            )
        }
    }

    /// The number of groups.
    public func groupCount() throws -> Int {
        try database.query("SELECT count(*) FROM groups").first?.int(0) ?? 0
    }

    // MARK: Writing

    /// Adds a group with its members and expenses; it comes first.
    public func insert(_ group: ExpenseGroup) throws {
        try database.transaction {
            try database.run(
                "INSERT INTO groups (id, name, emoji, created_at) VALUES (?, ?, ?, ?)",
                [SQLValue(group.id), .text(group.name), .text(group.emoji), SQLValue(group.createdAt)]
            )
            for member in group.members {
                try insertMember(member, in: group.id)
            }
            // Oldest first, so that the newest has the highest `seq` and comes first.
            for expense in group.expenses.reversed() {
                try upsert(expense, in: group.id)
            }
        }
    }

    /// Renames a group, or changes its emoji.
    public func update(group id: ExpenseGroup.ID, name: String, emoji: String) throws {
        try database.run(
            "UPDATE groups SET name = ?, emoji = ? WHERE id = ?",
            [.text(name), .text(emoji), SQLValue(id)]
        )
    }

    /// Deletes a group, its members and its expenses.
    public func deleteGroup(_ id: ExpenseGroup.ID) throws {
        try database.run("DELETE FROM groups WHERE id = ?", [SQLValue(id)])
    }

    /// Adds a member at the end of a group.
    public func insert(_ member: Member, in group: ExpenseGroup.ID) throws {
        try insertMember(member, in: group)
    }

    /// Deletes a member. Returns `false`, and deletes nothing, when they paid or share an
    /// expense.
    @discardableResult
    public func deleteMember(_ id: Member.ID) throws -> Bool {
        let involved = try database.query(
            """
            SELECT EXISTS (SELECT 1 FROM expenses WHERE paid_by = ?1)
                OR EXISTS (SELECT 1 FROM expense_participants WHERE member_id = ?1)
            """,
            [SQLValue(id)]
        )
        guard involved.first?.int(0) == 0 else { return false }
        try database.run("DELETE FROM members WHERE id = ?", [SQLValue(id)])
        return true
    }

    /// Adds an expense at the top of its group, or changes it in place.
    public func save(_ expense: Expense, in group: ExpenseGroup.ID) throws {
        try database.transaction { try upsert(expense, in: group) }
    }

    /// Deletes an expense.
    public func deleteExpense(_ id: Expense.ID) throws {
        try database.run("DELETE FROM expenses WHERE id = ?", [SQLValue(id)])
    }

    /// Deletes everything.
    public func deleteAll() throws {
        try database.run("DELETE FROM groups")
    }

    // MARK: The JSON file of Duova 1.0

    /// Moves the groups of the JSON file written by the first version of the app into the
    /// database, then deletes the file. Returns the number of groups imported.
    @discardableResult
    public func importJSON(at url: URL) throws -> Int {
        guard FileManager.default.fileExists(atPath: url.path(percentEncoded: false)) else { return 0 }
        let groups = try JSONDecoder().decode([ExpenseGroup].self, from: Data(contentsOf: url))
        try database.transaction {
            // The file lists the newest group first.
            for group in groups.reversed() {
                if try !exists(group.id) { try insert(group) }
            }
        }
        try FileManager.default.removeItem(at: url)
        return groups.count
    }

    // MARK: Helpers

    private func exists(_ group: ExpenseGroup.ID) throws -> Bool {
        try database.query("SELECT 1 FROM groups WHERE id = ?", [SQLValue(group)]).isEmpty == false
    }

    private func insertMember(_ member: Member, in group: ExpenseGroup.ID) throws {
        try database.run(
            "INSERT INTO members (id, group_id, name) VALUES (?, ?, ?)",
            [SQLValue(member.id), SQLValue(group), .text(member.name)]
        )
    }

    private func upsert(_ expense: Expense, in group: ExpenseGroup.ID) throws {
        try database.run(
            """
            INSERT INTO expenses (id, group_id, title, amount, paid_by, date) VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT (id) DO UPDATE SET
                title = excluded.title, amount = excluded.amount,
                paid_by = excluded.paid_by, date = excluded.date
            """,
            [
                SQLValue(expense.id), SQLValue(group), .text(expense.title), SQLValue(expense.amount),
                SQLValue(expense.paidBy), SQLValue(expense.date),
            ]
        )
        try database.run("DELETE FROM expense_participants WHERE expense_id = ?", [SQLValue(expense.id)])
        for (position, member) in expense.participants.enumerated() {
            try database.run(
                "INSERT INTO expense_participants (expense_id, member_id, position) VALUES (?, ?, ?)",
                [SQLValue(expense.id), SQLValue(member), SQLValue(position)]
            )
        }
    }
}
