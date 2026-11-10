import Foundation

/// An amount of money in euro cents. Always an integer: `0.1 + 0.2` never happens.
public typealias Cents = Int

/// A member of a group.
public struct Member: Identifiable, Hashable, Codable, Sendable {
    public var id: UUID
    public var name: String

    public init(id: UUID = UUID(), name: String) {
        self.id = id
        self.name = name
    }
}

/// An expense: `amount` cents paid by `paidBy` for `participants`.
public struct Expense: Identifiable, Hashable, Codable, Sendable {
    public var id: UUID
    public var title: String
    public var amount: Cents
    public var paidBy: Member.ID
    public var participants: [Member.ID]
    public var date: Date

    public init(
        id: UUID = UUID(),
        title: String,
        amount: Cents,
        paidBy: Member.ID,
        participants: [Member.ID],
        date: Date = .now
    ) {
        self.id = id
        self.title = title
        self.amount = amount
        self.paidBy = paidBy
        self.participants = participants
        self.date = date
    }
}

/// A transfer that settles part of the debts.
public struct Transfer: Hashable, Sendable {
    public var from: Member.ID
    public var to: Member.ID
    public var amount: Cents

    public init(from: Member.ID, to: Member.ID, amount: Cents) {
        self.from = from
        self.to = to
        self.amount = amount
    }
}

/// A group ("Week-end à Lyon", "Coloc"…): its members and their expenses.
///
/// Named `ExpenseGroup` so that it never hides SwiftUI's `Group`.
public struct ExpenseGroup: Identifiable, Hashable, Codable, Sendable {
    public var id: UUID
    public var name: String
    public var emoji: String
    public var members: [Member]
    public var expenses: [Expense]
    public var createdAt: Date

    public init(
        id: UUID = UUID(),
        name: String,
        emoji: String = "🧾",
        members: [Member] = [],
        expenses: [Expense] = [],
        createdAt: Date = .now
    ) {
        self.id = id
        self.name = name
        self.emoji = emoji
        self.members = members
        self.expenses = expenses
        self.createdAt = createdAt
    }

    /// The member with this id.
    public func member(_ id: Member.ID) -> Member? {
        members.first { $0.id == id }
    }

    /// The name of the member with this id, `"?"` if they left.
    public func name(of id: Member.ID) -> String {
        member(id)?.name ?? "?"
    }

    /// The sum of every expense.
    public var total: Cents {
        expenses.reduce(0) { $0 + $1.amount }
    }

    /// Whether the member paid or shares an expense (then they cannot be removed).
    public func isInvolved(_ id: Member.ID) -> Bool {
        expenses.contains { $0.paidBy == id || $0.participants.contains(id) }
    }

    /// Every member and their balance, in the members' order.
    public var balances: [(member: Member, balance: Cents)] {
        let byID = Splitter.balances(members: members.map(\.id), expenses: expenses)
        return members.map { member in (member: member, balance: byID[member.id] ?? 0) }
    }

    /// The shortest list of transfers that settles the group.
    public var settlements: [Transfer] {
        Splitter.settlements(
            Splitter.balances(members: members.map(\.id), expenses: expenses),
            order: members.map(\.id)
        )
    }

    /// The recap to paste in the group chat.
    public var recap: String {
        Recap.text(for: self)
    }
}
