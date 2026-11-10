import Foundation

/// The arithmetic of a group.
public enum Splitter {
    /// Splits `amount` cents into equal shares, without losing a cent: the first
    /// participants get the remaining cents (1000 for 3 → 334, 333, 333).
    public static func split(_ amount: Cents, among participants: [Member.ID]) -> [(id: Member.ID, share: Cents)] {
        let count = participants.count
        guard count > 0 else { return [] }
        // Floor division, also for a negative amount (a refund).
        var base = amount / count
        if amount % count != 0 && amount < 0 { base -= 1 }
        var remainder = amount - base * count
        return participants.map { id -> (id: Member.ID, share: Cents) in
            let extra = remainder > 0 ? 1 : 0
            remainder -= extra
            return (id: id, share: base + extra)
        }
    }

    /// The balance of every member: what they paid minus their share. Positive: the
    /// group owes them money; negative: they owe money. The balances add up to zero.
    public static func balances(members: [Member.ID], expenses: [Expense]) -> [Member.ID: Cents] {
        var balances = Dictionary(uniqueKeysWithValues: members.map { ($0, 0) })
        for expense in expenses where !expense.participants.isEmpty {
            balances[expense.paidBy, default: 0] += expense.amount
            for (id, share) in split(expense.amount, among: expense.participants) {
                balances[id, default: 0] -= share
            }
        }
        return balances
    }

    /// The shortest list of transfers that settles every balance: at each turn, the
    /// biggest debtor pays the biggest creditor. At most `n − 1` transfers for `n`
    /// people. `order` breaks the ties (the members' order), so the result never changes.
    public static func settlements(_ balances: [Member.ID: Cents], order: [Member.ID]) -> [Transfer] {
        let rank = Dictionary(uniqueKeysWithValues: order.enumerated().map { ($1, $0) })
        let ordered = balances.keys.sorted { (rank[$0] ?? .max, $0.uuidString) < (rank[$1] ?? .max, $1.uuidString) }
        typealias Entry = (id: Member.ID, amount: Cents)
        func biggestFirst(_ list: [Entry]) -> [Entry] {
            list.enumerated()
                .sorted { ($1.element.amount, $0.offset) < ($0.element.amount, $1.offset) }
                .map(\.element)
        }
        var creditors = biggestFirst(ordered.compactMap { id -> Entry? in
            let balance = balances[id] ?? 0
            return balance > 0 ? (id: id, amount: balance) : nil
        })
        var debtors = biggestFirst(ordered.compactMap { id -> Entry? in
            let balance = balances[id] ?? 0
            return balance < 0 ? (id: id, amount: -balance) : nil
        })

        var transfers: [Transfer] = []
        var i = 0
        var j = 0
        while i < debtors.count && j < creditors.count {
            let amount = min(debtors[i].amount, creditors[j].amount)
            transfers.append(Transfer(from: debtors[i].id, to: creditors[j].id, amount: amount))
            debtors[i].amount -= amount
            creditors[j].amount -= amount
            if debtors[i].amount == 0 { i += 1 }
            if creditors[j].amount == 0 { j += 1 }
        }
        return transfers
    }
}
