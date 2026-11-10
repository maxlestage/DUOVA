import DuovaKit
import SwiftUI

/// A group: its balances, the transfers that settle it, its expenses and its members.
struct GroupDetailView: View {
    let groupID: ExpenseGroup.ID

    @Environment(GroupStore.self) private var store
    @State private var editing: Expense?
    @State private var isAddingExpense = false
    @State private var isAddingMember = false
    @State private var newMember = ""
    @State private var blockedMember: Member?

    var body: some View {
        if let group = store.group(groupID) {
            content(group)
        } else {
            ContentUnavailableView("Groupe supprimé", systemImage: "trash")
        }
    }

    private func content(_ group: ExpenseGroup) -> some View {
        List {
            Section {
                SummaryCard(group: group)
                    .listRowInsets(EdgeInsets())
                    .listRowBackground(Color.clear)
            }

            if !group.members.isEmpty {
                Section("Remboursements") {
                    let transfers = group.settlements
                    if transfers.isEmpty {
                        Label("Tout le monde est quitte", systemImage: "checkmark.seal.fill")
                            .foregroundStyle(Theme.plus)
                    } else {
                        ForEach(Array(transfers.enumerated()), id: \.offset) { _, transfer in
                            TransferRow(group: group, transfer: transfer)
                        }
                    }
                }
            }

            Section {
                if group.expenses.isEmpty {
                    Text("Aucune dépense pour l'instant.")
                        .foregroundStyle(.secondary)
                }
                ForEach(group.expenses) { expense in
                    Button {
                        editing = expense
                    } label: {
                        ExpenseRow(group: group, expense: expense)
                    }
                    .foregroundStyle(.primary)
                    .swipeActions {
                        Button("Supprimer", systemImage: "trash", role: .destructive) {
                            withAnimation { store.deleteExpense(expense.id, in: group.id) }
                        }
                    }
                }
            } header: {
                Text("Dépenses")
            } footer: {
                if group.members.count < 2 {
                    Text("Ajoutez au moins deux membres pour partager une dépense.")
                }
            }

            Section("Membres") {
                ForEach(group.members) { member in
                    HStack(spacing: 12) {
                        AvatarView(name: member.name, color: Theme.color(of: member.id, in: group))
                        Text(member.name)
                    }
                    .swipeActions {
                        Button("Retirer", systemImage: "person.fill.xmark", role: .destructive) {
                            if !store.removeMember(member.id, from: group.id) {
                                blockedMember = member
                            }
                        }
                    }
                }
                Button("Ajouter un membre", systemImage: "person.badge.plus") {
                    isAddingMember = true
                }
            }
        }
        .animation(.snappy, value: group)
        .navigationTitle("\(group.emoji) \(group.name)")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                ShareLink(item: group.recap, subject: Text(group.name), preview: SharePreview("Récap · \(group.name)"))
                    .disabled(group.expenses.isEmpty)
            }
            ToolbarItem(placement: .bottomBar) {
                Button {
                    isAddingExpense = true
                } label: {
                    Label("Ajouter une dépense", systemImage: "plus.circle.fill")
                        .labelStyle(.titleAndIcon)
                        .font(.headline)
                }
                .buttonStyle(.borderedProminent)
                .disabled(group.members.count < 2)
            }
        }
        .sheet(isPresented: $isAddingExpense) {
            ExpenseFormView(group: group, expense: nil) { store.save($0, in: group.id) }
        }
        .sheet(item: $editing) { expense in
            ExpenseFormView(group: group, expense: expense) { store.save($0, in: group.id) }
        }
        .alert("Nouveau membre", isPresented: $isAddingMember) {
            TextField("Prénom", text: $newMember)
                .textContentType(.givenName)
            Button("Ajouter") {
                let name = newMember.trimmingCharacters(in: .whitespacesAndNewlines)
                newMember = ""
                guard !name.isEmpty else { return }
                withAnimation { store.addMember(named: name, to: group.id) }
            }
            Button("Annuler", role: .cancel) { newMember = "" }
        }
        .alert(
            "Impossible de retirer \(blockedMember?.name ?? "ce membre")",
            isPresented: Binding(get: { blockedMember != nil }, set: { if !$0 { blockedMember = nil } })
        ) {
            Button("OK", role: .cancel) {}
        } message: {
            Text("Ce membre a payé ou partage des dépenses : supprimez-les d'abord.")
        }
        .sensoryFeedback(.success, trigger: group.expenses.count)
    }
}

/// The total, and every balance as a bar from the center: right for a credit, left for
/// a debt.
private struct SummaryCard: View {
    let group: ExpenseGroup

    var body: some View {
        let balances = group.balances
        let largest = max(balances.map { abs($0.balance) }.max() ?? 0, 1)
        VStack(alignment: .leading, spacing: 14) {
            VStack(alignment: .leading, spacing: 2) {
                Text("Total")
                    .font(.subheadline)
                    .foregroundStyle(.white.opacity(0.75))
                AmountText(cents: group.total)
                    .font(.system(.largeTitle, design: .rounded, weight: .heavy))
                    .foregroundStyle(.white)
            }
            ForEach(balances, id: \.member.id) { entry in
                BalanceBar(
                    name: entry.member.name,
                    color: Theme.color(of: entry.member.id, in: group),
                    balance: entry.balance,
                    fraction: Double(abs(entry.balance)) / Double(largest)
                )
            }
        }
        .padding(20)
        .background {
            RoundedRectangle(cornerRadius: 26)
                .fill(
                    MeshGradient(
                        width: 2,
                        height: 2,
                        points: [[0, 0], [1, 0], [0, 1], [1, 1]],
                        colors: [Theme.violet, Theme.coral, Color(red: 0.1, green: 0.08, blue: 0.2), Theme.teal.opacity(0.8)]
                    )
                )
        }
        .padding(.vertical, 6)
        .accessibilityElement(children: .contain)
    }
}

private struct BalanceBar: View {
    let name: String
    let color: Color
    let balance: Cents
    let fraction: Double

    var body: some View {
        HStack(spacing: 10) {
            Text(name)
                .font(.subheadline.weight(.semibold))
                .lineLimit(1)
                .frame(width: 72, alignment: .leading)
            GeometryReader { proxy in
                let half = proxy.size.width / 2
                ZStack(alignment: .leading) {
                    Capsule().fill(.white.opacity(0.15))
                    Capsule()
                        .fill(balance >= 0 ? Theme.plus : Theme.minus)
                        .frame(width: max(half * fraction, 2))
                        .offset(x: balance >= 0 ? half : half - half * fraction)
                }
            }
            .frame(height: 10)
            AmountText(cents: balance, signed: true)
                .font(.subheadline.weight(.bold))
                .frame(minWidth: 86, alignment: .trailing)
        }
        .foregroundStyle(.white)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(name) : \(Money.signed(balance))")
    }
}

private struct TransferRow: View {
    let group: ExpenseGroup
    let transfer: Transfer
    @State private var hop = false

    var body: some View {
        HStack(spacing: 10) {
            AvatarView(name: group.name(of: transfer.from), color: Theme.color(of: transfer.from, in: group), size: 28)
            Image(systemName: "arrow.right")
                .font(.caption.weight(.bold))
                .foregroundStyle(Theme.gold)
                .offset(x: hop ? 3 : -3)
            AvatarView(name: group.name(of: transfer.to), color: Theme.color(of: transfer.to, in: group), size: 28)
            Text("\(group.name(of: transfer.from)) rembourse \(group.name(of: transfer.to))")
                .font(.subheadline)
                .lineLimit(2)
            Spacer(minLength: 6)
            AmountText(cents: transfer.amount)
                .font(.subheadline.weight(.heavy))
                .foregroundStyle(Theme.gold)
        }
        .onAppear {
            withAnimation(.easeInOut(duration: 0.8).repeatForever()) { hop = true }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(
            "\(group.name(of: transfer.from)) rembourse \(Money.format(transfer.amount)) à \(group.name(of: transfer.to))"
        )
    }
}

private struct ExpenseRow: View {
    let group: ExpenseGroup
    let expense: Expense

    var body: some View {
        HStack(spacing: 12) {
            AvatarView(name: group.name(of: expense.paidBy), color: Theme.color(of: expense.paidBy, in: group))
            VStack(alignment: .leading, spacing: 2) {
                Text(expense.title)
                    .font(.body.weight(.semibold))
                Text("\(group.name(of: expense.paidBy)) pour \(expense.participants.map(group.name(of:)).formatted(.list(type: .and)))")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }
            Spacer(minLength: 8)
            AmountText(cents: expense.amount)
                .font(.body.weight(.bold))
        }
        .accessibilityElement(children: .combine)
    }
}
