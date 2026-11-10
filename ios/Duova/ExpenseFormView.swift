import DuovaKit
import SwiftUI

/// A new expense, or a change to an existing one: what, how much, who paid, for whom.
struct ExpenseFormView: View {
    let group: ExpenseGroup
    let onSave: (Expense) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var expense: Expense
    @State private var amountText: String
    @FocusState private var focused: Field?

    private enum Field { case title, amount }

    init(group: ExpenseGroup, expense: Expense?, onSave: @escaping (Expense) -> Void) {
        self.group = group
        self.onSave = onSave
        let first = group.members.first?.id ?? UUID()
        let start = expense ?? Expense(title: "", amount: 0, paidBy: first, participants: group.members.map(\.id))
        _expense = State(initialValue: start)
        _amountText = State(initialValue: expense.map { Money.format($0.amount).replacingOccurrences(of: "\u{00A0}€", with: "") } ?? "")
    }

    private var cents: Cents? { Money.parse(amountText) }

    private var isValid: Bool {
        cents != nil && !expense.participants.isEmpty && group.member(expense.paidBy) != nil
    }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("Description (ex. Restaurant)", text: $expense.title)
                        .focused($focused, equals: .title)
                        .submitLabel(.next)
                        .onSubmit { focused = .amount }
                    HStack {
                        TextField("Montant", text: $amountText)
                            .keyboardType(.decimalPad)
                            .focused($focused, equals: .amount)
                            .font(.title2.weight(.bold))
                            .monospacedDigit()
                        Text("€").foregroundStyle(.secondary)
                    }
                }

                Section("Payé par") {
                    Picker("Payé par", selection: $expense.paidBy) {
                        ForEach(group.members) { member in
                            Text(member.name).tag(member.id)
                        }
                    }
                    .pickerStyle(.menu)
                }

                Section {
                    ForEach(group.members) { member in
                        let selected = expense.participants.contains(member.id)
                        Button {
                            toggle(member.id)
                        } label: {
                            HStack(spacing: 12) {
                                AvatarView(name: member.name, color: Theme.color(of: member.id, in: group), size: 30)
                                    .saturation(selected ? 1 : 0)
                                Text(member.name)
                                    .foregroundStyle(.primary)
                                Spacer()
                                if let cents, selected {
                                    let share = Splitter.split(cents, among: orderedParticipants)
                                        .first { $0.id == member.id }?.share ?? 0
                                    AmountText(cents: share)
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                }
                                Image(systemName: selected ? "checkmark.circle.fill" : "circle")
                                    .foregroundStyle(selected ? Theme.coral : .secondary)
                                    .symbolEffect(.bounce, value: selected)
                                    .contentTransition(.symbolEffect(.replace))
                            }
                        }
                        .accessibilityAddTraits(selected ? .isSelected : [])
                    }
                } header: {
                    Text("Pour qui ?")
                } footer: {
                    Text("Le montant est partagé à parts égales, au centime près.")
                }
            }
            .navigationTitle(group.expenses.contains { $0.id == expense.id } ? "Modifier" : "Nouvelle dépense")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Annuler", role: .cancel) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Enregistrer") {
                        guard let cents else { return }
                        var saved = expense
                        saved.amount = cents
                        saved.participants = orderedParticipants
                        let title = saved.title.trimmingCharacters(in: .whitespacesAndNewlines)
                        saved.title = title.isEmpty ? "Dépense" : title
                        onSave(saved)
                        dismiss()
                    }
                    .disabled(!isValid)
                }
            }
            .onAppear { if expense.title.isEmpty { focused = .title } }
        }
        .presentationDetents([.large])
    }

    /// The participants, in the members' order (the extra cents go to the first ones).
    private var orderedParticipants: [Member.ID] {
        group.members.map(\.id).filter { expense.participants.contains($0) }
    }

    private func toggle(_ id: Member.ID) {
        withAnimation(.snappy) {
            if let index = expense.participants.firstIndex(of: id) {
                expense.participants.remove(at: index)
            } else {
                expense.participants.append(id)
            }
        }
    }
}
