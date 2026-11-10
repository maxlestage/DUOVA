//! The arithmetic of Duova, exactly as the iOS app does it (`ios/DuovaKit`): amounts in
//! cents (integers, never floats), equal shares without losing a cent, balances and the
//! shortest list of transfers.
//!
//! Shared by the server (the example of the page) and the browser (the simulator island).

use std::collections::BTreeMap;

/// A member of a group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub id: u32,
    pub name: String,
}

/// An expense: `amount` cents paid by `paid_by` for `participants`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expense {
    pub id: u32,
    pub description: String,
    pub amount: i64,
    pub paid_by: u32,
    pub participants: Vec<u32>,
}

/// A transfer that settles part of the debts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub from: u32,
    pub to: u32,
    pub amount: i64,
}

/// Splits `amount` cents into equal shares, without losing a cent: the first
/// participants get the remaining cents (1000 for 3 → 334, 333, 333).
pub fn split_amount(amount: i64, participants: &[u32]) -> Vec<(u32, i64)> {
    let n = participants.len() as i64;
    if n == 0 {
        return Vec::new();
    }
    let base = amount.div_euclid(n);
    let mut remainder = amount - base * n;
    participants
        .iter()
        .map(|&id| {
            let extra = i64::from(remainder > 0);
            remainder -= extra;
            (id, base + extra)
        })
        .collect()
}

/// The balance of every member: what they paid minus their share. Positive: the group
/// owes them money; negative: they owe money. The balances always add up to zero.
pub fn balances(members: &[Member], expenses: &[Expense]) -> BTreeMap<u32, i64> {
    let mut balances: BTreeMap<u32, i64> = members.iter().map(|m| (m.id, 0)).collect();
    for expense in expenses {
        if expense.participants.is_empty() {
            continue;
        }
        *balances.entry(expense.paid_by).or_default() += expense.amount;
        for (id, share) in split_amount(expense.amount, &expense.participants) {
            *balances.entry(id).or_default() -= share;
        }
    }
    balances
}

/// The shortest list of transfers that settles every balance: at each turn, the biggest
/// debtor pays the biggest creditor. At most `n − 1` transfers for `n` people.
pub fn settlements(balances: &BTreeMap<u32, i64>) -> Vec<Transfer> {
    let mut creditors: Vec<(u32, i64)> = balances
        .iter()
        .filter(|(_, b)| **b > 0)
        .map(|(id, b)| (*id, *b))
        .collect();
    let mut debtors: Vec<(u32, i64)> = balances
        .iter()
        .filter(|(_, b)| **b < 0)
        .map(|(id, b)| (*id, -*b))
        .collect();
    // Biggest first; ties keep the members' order, so the result never changes.
    creditors.sort_by_key(|c| std::cmp::Reverse(c.1));
    debtors.sort_by_key(|d| std::cmp::Reverse(d.1));

    let mut transfers = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < debtors.len() && j < creditors.len() {
        let amount = debtors[i].1.min(creditors[j].1);
        transfers.push(Transfer {
            from: debtors[i].0,
            to: creditors[j].0,
            amount,
        });
        debtors[i].1 -= amount;
        creditors[j].1 -= amount;
        if debtors[i].1 == 0 {
            i += 1;
        }
        if creditors[j].1 == 0 {
            j += 1;
        }
    }
    transfers
}

/// `4550` → `"45,50 €"`, `-123456` → `"−1 234,56 €"`, as French writes it (narrow
/// no-break spaces between thousands, a no-break space before the sign).
pub fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "−" } else { "" };
    let abs = cents.unsigned_abs();
    let euros = (abs / 100).to_string();
    let mut grouped = String::new();
    for (i, c) in euros.chars().enumerate() {
        if i > 0 && (euros.len() - i).is_multiple_of(3) {
            grouped.push('\u{202f}');
        }
        grouped.push(c);
    }
    format!("{sign}{grouped},{:02}\u{a0}€", abs % 100)
}

/// `"45,50"`, `"45.5"`, `"1 200"` → cents; `None` when the input is not a positive amount
/// (or has more than two decimals, or is absurdly large).
pub fn parse_amount(input: &str) -> Option<i64> {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '€')
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    let (units, decimals) = match cleaned.split_once('.') {
        Some((u, d)) => (u, d),
        None => (cleaned.as_str(), ""),
    };
    let units = if units.is_empty() { "0" } else { units };
    if !units.chars().all(|c| c.is_ascii_digit())
        || !decimals.chars().all(|c| c.is_ascii_digit())
        || decimals.len() > 2
        || units.len() > 9
    {
        return None;
    }
    let cents_part = format!("{decimals:0<2}").parse::<i64>().ok()?;
    let cents = units.parse::<i64>().ok()? * 100 + cents_part;
    (cents > 0).then_some(cents)
}

/// The recap to paste in a group chat.
pub fn recap(group: &str, members: &[Member], expenses: &[Expense]) -> String {
    let name = |id: u32| {
        members
            .iter()
            .find(|m| m.id == id)
            .map_or("?", |m| m.name.as_str())
    };
    let total: i64 = expenses.iter().map(|e| e.amount).sum();
    let transfers = settlements(&balances(members, expenses));
    let mut out = format!("🧾 {group}\nTotal : {}\n\n", format_cents(total));
    if transfers.is_empty() {
        out.push_str("✅ Tout le monde est quitte.\n");
    } else {
        out.push_str("Remboursements :\n");
        for t in transfers {
            out.push_str(&format!(
                "• {} → {} : {}\n",
                name(t.from),
                name(t.to),
                format_cents(t.amount)
            ));
        }
    }
    out.push_str("\nCalculé avec Duova");
    out
}

/// The example of the page: Alice pays 90 € of groceries for three, Bob 30 € of fuel for
/// two.
pub fn example() -> (Vec<Member>, Vec<Expense>) {
    let members = vec![
        Member {
            id: 1,
            name: "Alice".into(),
        },
        Member {
            id: 2,
            name: "Bob".into(),
        },
        Member {
            id: 3,
            name: "Chloé".into(),
        },
    ];
    let expenses = vec![
        Expense {
            id: 1,
            description: "Courses".into(),
            amount: 9000,
            paid_by: 1,
            participants: vec![1, 2, 3],
        },
        Expense {
            id: 2,
            description: "Essence".into(),
            amount: 3000,
            paid_by: 2,
            participants: vec![2, 3],
        },
    ];
    (members, expenses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_never_lose_a_cent() {
        assert_eq!(
            split_amount(1000, &[1, 2, 3]),
            vec![(1, 334), (2, 333), (3, 333)]
        );
        for amount in [1, 7, 99, 100, 101, 4550, 123_457] {
            for n in 1..=7u32 {
                let ids: Vec<u32> = (1..=n).collect();
                let shares = split_amount(amount, &ids);
                assert_eq!(shares.iter().map(|s| s.1).sum::<i64>(), amount);
                let max = shares.iter().map(|s| s.1).max().unwrap();
                let min = shares.iter().map(|s| s.1).min().unwrap();
                assert!(max - min <= 1);
            }
        }
        assert!(split_amount(100, &[]).is_empty());
    }

    #[test]
    fn the_example_of_the_page() {
        let (members, expenses) = example();
        let b = balances(&members, &expenses);
        assert_eq!(b[&1], 6000);
        assert_eq!(b[&2], -1500);
        assert_eq!(b[&3], -4500);
        assert_eq!(b.values().sum::<i64>(), 0);
        assert_eq!(
            settlements(&b),
            vec![
                Transfer {
                    from: 3,
                    to: 1,
                    amount: 4500
                },
                Transfer {
                    from: 2,
                    to: 1,
                    amount: 1500
                },
            ]
        );
    }

    #[test]
    fn at_most_n_minus_one_transfers_and_everyone_settled() {
        let members: Vec<Member> = (1..=6)
            .map(|id| Member {
                id,
                name: format!("M{id}"),
            })
            .collect();
        let expenses: Vec<Expense> = (1..=20)
            .map(|k: u32| Expense {
                id: k,
                description: String::new(),
                amount: i64::from(k * 1337 % 9001 + 1),
                paid_by: k % 6 + 1,
                participants: (1..=6).filter(|m| !(m + k).is_multiple_of(3)).collect(),
            })
            .collect();
        let b = balances(&members, &expenses);
        assert_eq!(b.values().sum::<i64>(), 0);
        let transfers = settlements(&b);
        assert!(transfers.len() < members.len());
        let mut after = b.clone();
        for t in &transfers {
            assert!(t.amount > 0);
            *after.get_mut(&t.from).unwrap() += t.amount;
            *after.get_mut(&t.to).unwrap() -= t.amount;
        }
        assert!(after.values().all(|v| *v == 0));
    }

    #[test]
    fn money_in_french() {
        assert_eq!(format_cents(4550), "45,50\u{a0}€");
        assert_eq!(format_cents(5), "0,05\u{a0}€");
        assert_eq!(format_cents(-123_456), "−1\u{202f}234,56\u{a0}€");
        assert_eq!(
            format_cents(100_000_000),
            "1\u{202f}000\u{202f}000,00\u{a0}€"
        );
    }

    #[test]
    fn amounts_typed_by_people() {
        assert_eq!(parse_amount("45,50"), Some(4550));
        assert_eq!(parse_amount("45.5"), Some(4550));
        assert_eq!(parse_amount(" 1 200 € "), Some(120_000));
        assert_eq!(parse_amount(",5"), Some(50));
        assert_eq!(parse_amount("0.1"), Some(10));
        for bad in ["", "0", "-3", "abc", "1,234", "1.2.3", "9999999999"] {
            assert_eq!(parse_amount(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn the_recap_lists_the_transfers() {
        let (members, expenses) = example();
        let text = recap("Week-end à Lyon", &members, &expenses);
        assert!(text.starts_with("🧾 Week-end à Lyon\nTotal : 120,00\u{a0}€"));
        assert!(text.contains("• Chloé → Alice : 45,00\u{a0}€"));
        assert!(text.contains("• Bob → Alice : 15,00\u{a0}€"));
        assert!(recap("Vide", &members, &[]).contains("Tout le monde est quitte"));
    }
}
