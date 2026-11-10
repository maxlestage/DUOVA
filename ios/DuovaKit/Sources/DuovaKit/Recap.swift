import Foundation

/// The text shared in the group chat.
public enum Recap {
    public static func text(for group: ExpenseGroup) -> String {
        var lines = ["🧾 \(group.name)", "Total : \(Money.format(group.total))", ""]
        let transfers = group.settlements
        if transfers.isEmpty {
            lines.append("✅ Tout le monde est quitte.")
        } else {
            lines.append("Remboursements :")
            for transfer in transfers {
                lines.append(
                    "• \(group.name(of: transfer.from)) → \(group.name(of: transfer.to)) : \(Money.format(transfer.amount))"
                )
            }
        }
        lines.append("")
        lines.append("Calculé avec Duova")
        return lines.joined(separator: "\n")
    }
}

extension ExpenseGroup {
    /// The example of the website: Alice pays 90 € of groceries for three, Bob 30 € of
    /// fuel for two.
    public static var example: ExpenseGroup {
        let alice = Member(name: "Alice")
        let bob = Member(name: "Bob")
        let chloe = Member(name: "Chloé")
        return ExpenseGroup(
            name: "Week-end à Lyon",
            emoji: "🍷",
            members: [alice, bob, chloe],
            expenses: [
                Expense(title: "Essence", amount: 3000, paidBy: bob.id, participants: [bob.id, chloe.id]),
                Expense(title: "Courses", amount: 9000, paidBy: alice.id, participants: [alice.id, bob.id, chloe.id]),
            ]
        )
    }
}
