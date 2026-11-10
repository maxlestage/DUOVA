import Foundation

/// Euros as French writes them, and amounts typed by people.
public enum Money {
    /// `4550` → `"45,50 €"`, `-123456` → `"−1 234,56 €"`: narrow no-break spaces
    /// between thousands, a no-break space before the sign. The same text everywhere,
    /// whatever the device's region (it is the website's too).
    public static func format(_ cents: Cents) -> String {
        let sign = cents < 0 ? "−" : ""
        let magnitude = cents.magnitude
        let euros = String(magnitude / 100)
        var grouped = ""
        for (index, character) in euros.enumerated() {
            if index > 0 && (euros.count - index) % 3 == 0 {
                grouped.append("\u{202F}")
            }
            grouped.append(character)
        }
        let rest = magnitude % 100
        let decimals = rest < 10 ? "0\(rest)" : "\(rest)"
        return "\(sign)\(grouped),\(decimals)\u{00A0}€"
    }

    /// `+60,00 €` for a credit, `−15,00 €` for a debt, `0,00 €` for nothing.
    public static func signed(_ cents: Cents) -> String {
        cents > 0 ? "+" + format(cents) : format(cents)
    }

    /// `"45,50"`, `"45.5"`, `"1 200 €"` → cents; `nil` when the text is not a positive
    /// amount (or has more than two decimals, or is absurdly large).
    public static func parse(_ input: String) -> Cents? {
        let cleaned = String(
            input
                .filter { !$0.isWhitespace && $0 != "€" }
                .map { $0 == "," ? "." : $0 }
        )
        let parts = cleaned.split(separator: ".", omittingEmptySubsequences: false)
        guard parts.count <= 2 else { return nil }
        let units = parts[0].isEmpty ? "0" : String(parts[0])
        let decimals = parts.count == 2 ? String(parts[1]) : ""
        guard units.allSatisfy(\.isPlainDigit),
            decimals.allSatisfy(\.isPlainDigit),
            decimals.count <= 2,
            units.count <= 9,
            let euros = Int(units)
        else { return nil }
        let padded = decimals.padding(toLength: 2, withPad: "0", startingAt: 0)
        guard let cents = Int(padded) else { return nil }
        let total = euros * 100 + cents
        return total > 0 ? total : nil
    }
}

extension Character {
    fileprivate var isPlainDigit: Bool {
        ("0"..."9").contains(self)
    }
}
