import Foundation
import Testing

@testable import DuovaKit

@Suite("Shares")
struct SharesTests {
    @Test func neverLoseACent() {
        let ids = (0..<3).map { _ in UUID() }
        #expect(Splitter.split(1000, among: ids).map(\.share) == [334, 333, 333])
        for amount in [1, 7, 99, 100, 101, 4550, 123_457] {
            for count in 1...7 {
                let shares = Splitter.split(amount, among: (0..<count).map { _ in UUID() }).map(\.share)
                #expect(shares.reduce(0, +) == amount)
                #expect(shares.max()! - shares.min()! <= 1)
            }
        }
        #expect(Splitter.split(100, among: []).isEmpty)
    }
}

@Suite("Balances and transfers")
struct SettlementTests {
    @Test func theExampleOfTheWebsite() {
        let group = ExpenseGroup.example
        let balances = group.balances.map(\.balance)
        #expect(balances == [6000, -1500, -4500])
        #expect(balances.reduce(0, +) == 0)
        let names = group.settlements.map { "\(group.name(of: $0.from))→\(group.name(of: $0.to)) \($0.amount)" }
        #expect(names == ["Chloé→Alice 4500", "Bob→Alice 1500"])
    }

    @Test func atMostNMinusOneTransfersAndEveryoneSettled() {
        let members = (1...6).map { Member(name: "M\($0)") }
        let expenses = (1...20).map { k in
            Expense(
                title: "",
                amount: k * 1337 % 9001 + 1,
                paidBy: members[k % 6].id,
                participants: members.enumerated().filter { ($0.offset + 1 + k) % 3 != 0 }.map(\.element.id)
            )
        }
        let group = ExpenseGroup(name: "Test", members: members, expenses: expenses)
        var after = Dictionary(uniqueKeysWithValues: group.balances.map { ($0.member.id, $0.balance) })
        #expect(after.values.reduce(0, +) == 0)
        let transfers = group.settlements
        #expect(transfers.count < members.count)
        for transfer in transfers {
            #expect(transfer.amount > 0)
            after[transfer.from, default: 0] += transfer.amount
            after[transfer.to, default: 0] -= transfer.amount
        }
        #expect(after.values.allSatisfy { $0 == 0 })
    }

    @Test func nothingToSettle() {
        let group = ExpenseGroup(name: "Vide", members: [Member(name: "A"), Member(name: "B")])
        #expect(group.settlements.isEmpty)
        #expect(group.recap.contains("Tout le monde est quitte"))
    }
}

@Suite("Money")
struct MoneyTests {
    @Test func inFrench() {
        #expect(Money.format(4550) == "45,50\u{00A0}€")
        #expect(Money.format(5) == "0,05\u{00A0}€")
        #expect(Money.format(-123_456) == "−1\u{202F}234,56\u{00A0}€")
        #expect(Money.format(100_000_000) == "1\u{202F}000\u{202F}000,00\u{00A0}€")
        #expect(Money.signed(6000) == "+60,00\u{00A0}€")
    }

    @Test(arguments: [
        ("45,50", 4550), ("45.5", 4550), (" 1 200 € ", 120_000), (",5", 50), ("0.1", 10),
    ])
    func typedByPeople(input: String, cents: Int) {
        #expect(Money.parse(input) == cents)
    }

    @Test(arguments: ["", "0", "-3", "abc", "1,234", "1.2.3", "9999999999"])
    func rejected(input: String) {
        #expect(Money.parse(input) == nil)
    }
}

@Suite("Recap")
struct RecapTests {
    @Test func listsTheTransfers() {
        let text = ExpenseGroup.example.recap
        #expect(text.hasPrefix("🧾 Week-end à Lyon\nTotal : 120,00\u{00A0}€"))
        #expect(text.contains("• Chloé → Alice : 45,00\u{00A0}€"))
        #expect(text.contains("• Bob → Alice : 15,00\u{00A0}€"))
    }

    @Test func roundTripsAsJSON() throws {
        let group = ExpenseGroup.example
        let data = try JSONEncoder().encode(group)
        #expect(try JSONDecoder().decode(ExpenseGroup.self, from: data) == group)
    }
}
