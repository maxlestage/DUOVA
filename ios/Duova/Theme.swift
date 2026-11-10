import DuovaKit
import SwiftUI

/// The colors of Duova, the same as the website's.
enum Theme {
    static let coral = Color(red: 1.0, green: 0.478, blue: 0.349)
    static let violet = Color(red: 0.486, green: 0.361, blue: 1.0)
    static let teal = Color(red: 0.129, green: 0.78, blue: 0.659)
    static let gold = Color(red: 1.0, green: 0.757, blue: 0.271)
    static let plus = Color(red: 0.239, green: 0.863, blue: 0.592)
    static let minus = Color(red: 1.0, green: 0.42, blue: 0.506)

    /// One color per member, in the order they joined.
    static let members: [Color] = [
        coral, violet, teal, gold,
        Color(red: 1.0, green: 0.373, blue: 0.635),
        Color(red: 0.247, green: 0.663, blue: 1.0),
        Color(red: 0.608, green: 0.882, blue: 0.365),
        Color(red: 0.753, green: 0.518, blue: 0.988),
    ]

    static func color(of member: Member.ID, in group: ExpenseGroup) -> Color {
        let index = group.members.firstIndex { $0.id == member } ?? 0
        return members[index % members.count]
    }

    static let gradient = LinearGradient(
        colors: [gold, coral, Color(red: 1.0, green: 0.373, blue: 0.635)],
        startPoint: .topLeading,
        endPoint: .bottomTrailing
    )
}

/// A member's initial in their color.
struct AvatarView: View {
    let name: String
    let color: Color
    var size: CGFloat = 32

    var body: some View {
        Text(name.prefix(1).uppercased())
            .font(.system(size: size * 0.45, weight: .heavy, design: .rounded))
            .foregroundStyle(.black.opacity(0.8))
            .frame(width: size, height: size)
            .background(color.gradient, in: .circle)
            .shadow(color: color.opacity(0.5), radius: 6, y: 3)
            .accessibilityHidden(true)
    }
}

/// An amount that rolls its digits when it changes.
struct AmountText: View {
    let cents: Cents
    var signed = false

    var body: some View {
        Text(signed ? Money.signed(cents) : Money.format(cents))
            .monospacedDigit()
            .contentTransition(.numericText(value: Double(cents)))
            .animation(.snappy, value: cents)
    }
}
