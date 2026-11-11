//! The interactive parts of the page. Each island renders on the server like any view
//! and, compiled to WebAssembly, hydrates in the browser from the same code.

use paquets::prelude::*;

use crate::split::{self, Expense, Member};

/// Every island, registered under the name used by [`island`].
pub const ISLANDS: [(&str, paquets::active::IslandFn); 3] =
    [("coins", coins), ("flow", flow), ("simulator", simulator)];

// ----- 2D scenes -------------------------------------------------------------------------

/// The hero: coins, notes and receipts falling in 2D, bouncing and piling up (a tap throws
/// them up, the pointer pushes them).
pub fn coins(_props: &str) -> Node {
    canvas()
        .class("coins")
        .attr("aria-hidden", "true")
        .use_action(start_coins)
        .into()
}

/// "How it works": three people, and the coins that go from those who owe to the one
/// who paid, along curves, in 2D.
pub fn flow(_props: &str) -> Node {
    canvas()
        .class("flow__canvas")
        .attr("aria-hidden", "true")
        .use_action(start_flow)
        .into()
}

#[cfg(target_arch = "wasm32")]
fn start_coins(canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    crate::client::coins::start(canvas)
}

#[cfg(target_arch = "wasm32")]
fn start_flow(canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    crate::client::flow::start(canvas)
}

#[cfg(not(target_arch = "wasm32"))]
fn start_coins(_canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    || {}
}

#[cfg(not(target_arch = "wasm32"))]
fn start_flow(_canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    || {}
}

// ----- The simulator --------------------------------------------------------------------

/// The colors of the members, in order (the same as in the app).
pub const COLORS: [&str; 8] = [
    "#ff7a59", "#7c5cff", "#21c7a8", "#ffc145", "#ff5fa2", "#3fa9ff", "#9be15d", "#c084fc",
];

fn color_of(members: &[Member], id: u32) -> &'static str {
    let index = members.iter().position(|m| m.id == id).unwrap_or(0);
    COLORS[index % COLORS.len()]
}

fn name_of(members: &[Member], id: u32) -> String {
    members
        .iter()
        .find(|m| m.id == id)
        .map_or_else(|| "?".to_owned(), |m| m.name.clone())
}

fn initial(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_default()
}

/// The whole group, in the browser: members, expenses, balances and transfers, computed
/// by the same Rust as the server (`split`). The props are the group's name.
pub fn simulator(props: &str) -> Node {
    let group = if props.trim().is_empty() {
        "Week-end à Lyon".to_owned()
    } else {
        props.trim().to_owned()
    };
    let (start_members, start_expenses) = split::example();
    let members = use_state(start_members);
    let expenses = use_state(start_expenses);
    let next_id = use_state(10u32);

    let new_member = use_state(String::new());
    let description = use_state(String::new());
    let amount = use_state(String::new());
    let payer = use_state(1u32);
    let chosen = use_state(vec![1u32, 2, 3]);
    let message = use_state(String::new());
    let copied = use_state(false);

    let add_member = move || {
        let name = new_member.get().trim().to_owned();
        if name.is_empty() {
            return;
        }
        if members.with(|m| m.len()) >= COLORS.len() {
            message.set(format!("{} membres au maximum dans la démo.", COLORS.len()));
            return;
        }
        if members.with(|m| m.iter().any(|x| x.name.eq_ignore_ascii_case(&name))) {
            message.set(format!("{name} est déjà dans le groupe."));
            return;
        }
        let id = next_id.get();
        next_id.set(id + 1);
        members.update(|m| m.push(Member { id, name }));
        chosen.update(|c| c.push(id));
        new_member.set(String::new());
        message.set(String::new());
    };

    let add_expense = move || {
        let Some(cents) = split::parse_amount(&amount.get()) else {
            message.set("Indique un montant positif, par exemple 12,50.".into());
            return;
        };
        let participants: Vec<u32> = members.with(|m| {
            m.iter()
                .map(|x| x.id)
                .filter(|id| chosen.with(|c| c.contains(id)))
                .collect()
        });
        if participants.is_empty() {
            message.set("Choisis au moins une personne qui profite de la dépense.".into());
            return;
        }
        let paid_by = payer.get();
        if !members.with(|m| m.iter().any(|x| x.id == paid_by)) {
            message.set("Choisis qui a payé.".into());
            return;
        }
        let text = description.get().trim().to_owned();
        let id = next_id.get();
        next_id.set(id + 1);
        expenses.update(|e| {
            e.insert(
                0,
                Expense {
                    id,
                    description: if text.is_empty() {
                        "Dépense".into()
                    } else {
                        text
                    },
                    amount: cents,
                    paid_by,
                    participants,
                },
            );
        });
        description.set(String::new());
        amount.set(String::new());
        message.set(String::new());
    };

    let remove_member = move |id: u32| {
        if expenses.with(|e| {
            e.iter()
                .any(|x| x.paid_by == id || x.participants.contains(&id))
        }) {
            message.set(format!(
                "{} a des dépenses : supprime-les d'abord.",
                members.with(|m| name_of(m, id))
            ));
            return;
        }
        members.update(|m| m.retain(|x| x.id != id));
        chosen.update(|c| c.retain(|x| *x != id));
        if payer.get() == id {
            payer.set(members.with(|m| m.first().map_or(0, |x| x.id)));
        }
        message.set(String::new());
    };

    let total = move || expenses.with(|e| e.iter().map(|x| x.amount).sum::<i64>());

    div()
        .class("sim")
        // Members
        .child(
            div()
                .class("sim__panel glass")
                .attr("data-tilt", "")
                .child(
                    h3().class("sim__title")
                        .child(span().text("1"))
                        .text("Membres"),
                )
                .child(ul().class("people").children_dyn(move || {
                    let list = members.get();
                    list.iter()
                        .map(|m| {
                            let id = m.id;
                            li().class("person")
                                .style(format!("--c:{}", color_of(&list, id)))
                                .child(span().class("avatar").text(initial(&m.name)))
                                .child(span().class("person__name").text(m.name.clone()))
                                .child(
                                    button()
                                        .class("person__remove")
                                        .attr("type", "button")
                                        .attr("aria-label", format!("Retirer {}", m.name))
                                        .on_click(move |_| remove_member(id))
                                        .text("×"),
                                )
                                .into()
                        })
                        .collect()
                }))
                .child(
                    form()
                        .class("inline")
                        .on("submit", move |e| {
                            e.prevent_default();
                            add_member();
                        })
                        .child(
                            input()
                                .attr("type", "text")
                                .attr("name", "member")
                                .attr("placeholder", "Prénom (ex. David)")
                                .attr("aria-label", "Prénom du nouveau membre")
                                .attr("maxlength", "24")
                                .attr("autocomplete", "off")
                                .attr_dyn("value", move || new_member.get())
                                .on_input(move |e| new_member.set(e.value())),
                        )
                        .child(
                            button()
                                .class("btn btn--small")
                                .attr("type", "submit")
                                .text("Ajouter"),
                        ),
                ),
        )
        // New expense
        .child(
            form()
                .class("sim__panel glass")
                .attr("data-tilt", "")
                .on("submit", move |e| {
                    e.prevent_default();
                    add_expense();
                })
                .child(
                    h3().class("sim__title")
                        .child(span().text("2"))
                        .text("Nouvelle dépense"),
                )
                .child(
                    div()
                        .class("fields")
                        .child(
                            label()
                                .class("field")
                                .child(span().text("Description"))
                                .child(
                                    input()
                                        .attr("type", "text")
                                        .attr("name", "description")
                                        .attr("placeholder", "Restaurant, essence…")
                                        .attr("maxlength", "40")
                                        .attr("autocomplete", "off")
                                        .attr_dyn("value", move || description.get())
                                        .on_input(move |e| description.set(e.value())),
                                ),
                        )
                        .child(
                            label()
                                .class("field")
                                .child(span().text("Montant (€)"))
                                .child(
                                    input()
                                        .attr("type", "text")
                                        .attr("name", "amount")
                                        .attr("inputmode", "decimal")
                                        .attr("placeholder", "45,50")
                                        .attr("autocomplete", "off")
                                        .attr_dyn("value", move || amount.get())
                                        .on_input(move |e| amount.set(e.value())),
                                ),
                        )
                        .child(
                            label().class("field").child(span().text("Payé par")).child(
                                select()
                                    .attr("name", "payer")
                                    .on("change", move |e| {
                                        payer.set(e.value().parse().unwrap_or(0));
                                    })
                                    .children_dyn(move || {
                                        members
                                            .get()
                                            .into_iter()
                                            .map(|m| {
                                                let id = m.id;
                                                option()
                                                    .attr("value", id.to_string())
                                                    .bool_attr("selected", move || {
                                                        payer.get() == id
                                                    })
                                                    .text(m.name)
                                                    .into()
                                            })
                                            .collect()
                                    }),
                            ),
                        ),
                )
                .child(p().class("field__label").text("Pour qui ?"))
                .child(div().class("toggles").children_dyn(move || {
                    let list = members.get();
                    list.iter()
                        .map(|m| {
                            let id = m.id;
                            button()
                                .class("toggle")
                                .attr("type", "button")
                                .style(format!("--c:{}", color_of(&list, id)))
                                .attr_dyn("aria-pressed", move || {
                                    chosen.with(|c| c.contains(&id)).to_string()
                                })
                                .on_click(move |_| {
                                    chosen.update(|c| {
                                        if let Some(i) = c.iter().position(|x| *x == id) {
                                            c.remove(i);
                                        } else {
                                            c.push(id);
                                        }
                                    });
                                })
                                .child(span().class("avatar").text(initial(&m.name)))
                                .text(m.name.clone())
                                .into()
                        })
                        .collect()
                }))
                .child(
                    p().class("sim__message")
                        .attr("role", "status")
                        .attr("aria-live", "polite")
                        .text_dyn(move || message.get()),
                )
                .child(
                    button()
                        .class("btn btn--primary btn--wide")
                        .attr("type", "submit")
                        .text("Ajouter la dépense"),
                ),
        )
        // Expenses
        .child(
            div()
                .class("sim__panel glass")
                .attr("data-tilt", "")
                .child(
                    h3().class("sim__title")
                        .child(span().text("3"))
                        .text("Dépenses")
                        .child(
                            small()
                                .class("sim__total")
                                .text_dyn(move || split::format_cents(total())),
                        ),
                )
                .child(ul().class("expenses").children_dyn(move || {
                    let list = expenses.get();
                    let people = members.get();
                    if list.is_empty() {
                        return vec![
                            li().class("empty")
                                .text("Aucune dépense pour l'instant.")
                                .into(),
                        ];
                    }
                    list.into_iter()
                        .map(|e| {
                            let id = e.id;
                            let who = e
                                .participants
                                .iter()
                                .map(|p| name_of(&people, *p))
                                .collect::<Vec<_>>()
                                .join(", ");
                            li().class("expense")
                                .style(format!("--c:{}", color_of(&people, e.paid_by)))
                                .child(
                                    span()
                                        .class("avatar")
                                        .text(initial(&name_of(&people, e.paid_by))),
                                )
                                .child(
                                    div()
                                        .class("expense__text")
                                        .child(strong().text(e.description.clone()))
                                        .child(small().text(format!(
                                            "{} pour {who}",
                                            name_of(&people, e.paid_by)
                                        ))),
                                )
                                .child(
                                    span()
                                        .class("expense__amount")
                                        .text(split::format_cents(e.amount)),
                                )
                                .child(
                                    button()
                                        .class("person__remove")
                                        .attr("type", "button")
                                        .attr("aria-label", format!("Supprimer {}", e.description))
                                        .on_click(move |_| {
                                            expenses.update(|l| l.retain(|x| x.id != id))
                                        })
                                        .text("×"),
                                )
                                .into()
                        })
                        .collect()
                })),
        )
        // Result
        .child(
            div()
                .class("sim__panel sim__result glass")
                .attr("data-tilt", "")
                .child(
                    h3().class("sim__title")
                        .child(span().text("4"))
                        .text("Soldes"),
                )
                .child(ul().class("bars").children_dyn(move || {
                    let people = members.get();
                    let balances = expenses.with(|e| split::balances(&people, e));
                    let max = balances.values().map(|b| b.abs()).max().unwrap_or(0).max(1);
                    people
                        .iter()
                        .map(|m| {
                            let b = balances.get(&m.id).copied().unwrap_or(0);
                            let width = (b.abs() * 100 / max).clamp(0, 100);
                            let side = match b.signum() {
                                1 => "bar--plus",
                                -1 => "bar--minus",
                                _ => "bar--zero",
                            };
                            let amount = if b > 0 {
                                format!("+{}", split::format_cents(b))
                            } else {
                                split::format_cents(b)
                            };
                            li().class("bar")
                                .class(side)
                                .style(format!("--c:{};--w:{width}%", color_of(&people, m.id)))
                                .child(span().class("bar__name").text(m.name.clone()))
                                .child(span().class("bar__track").child(span().class("bar__fill")))
                                .child(span().class("bar__amount").text(amount))
                                .into()
                        })
                        .collect()
                }))
                .child(
                    h3().class("sim__title sim__title--sub")
                        .text("Remboursements"),
                )
                .child(ol().class("transfers").children_dyn(move || {
                    let people = members.get();
                    let transfers =
                        expenses.with(|e| split::settlements(&split::balances(&people, e)));
                    if transfers.is_empty() {
                        return vec![
                            li().class("empty")
                                .text("✅ Tout le monde est quitte.")
                                .into(),
                        ];
                    }
                    transfers
                        .into_iter()
                        .enumerate()
                        .map(|(i, t)| {
                            li().class("transfer")
                                .style(format!("--i:{i}"))
                                .child(
                                    span()
                                        .class("transfer__avatars")
                                        .attr("aria-hidden", "true")
                                        .child(
                                            span()
                                                .class("avatar")
                                                .style(format!("--c:{}", color_of(&people, t.from)))
                                                .text(initial(&name_of(&people, t.from))),
                                        )
                                        .child(span().class("transfer__coin"))
                                        .child(
                                            span()
                                                .class("avatar")
                                                .style(format!("--c:{}", color_of(&people, t.to)))
                                                .text(initial(&name_of(&people, t.to))),
                                        ),
                                )
                                .child(
                                    span()
                                        .class("transfer__text")
                                        .child(strong().text(name_of(&people, t.from)))
                                        .text(" rembourse ")
                                        .child(strong().text(name_of(&people, t.to))),
                                )
                                .child(
                                    span()
                                        .class("transfer__amount")
                                        .text(split::format_cents(t.amount)),
                                )
                                .into()
                        })
                        .collect()
                }))
                .child(
                    div()
                        .class("sim__actions")
                        .child(
                            button()
                                .class("btn btn--primary")
                                .attr("type", "button")
                                .on_click(move |_| {
                                    let text = members
                                        .with(|m| expenses.with(|e| split::recap(&group, m, e)));
                                    #[cfg(target_arch = "wasm32")]
                                    crate::client::copy(&text);
                                    let _ = text;
                                    copied.set(true);
                                })
                                .text_dyn(move || {
                                    if copied.get() {
                                        "✓ Récap copié".to_owned()
                                    } else {
                                        "Copier le récap".to_owned()
                                    }
                                }),
                        )
                        .child(
                            button()
                                .class("btn btn--ghost")
                                .attr("type", "button")
                                .on_click(move |_| {
                                    let (m, e) = split::example();
                                    members.set(m);
                                    expenses.set(e);
                                    chosen.set(vec![1, 2, 3]);
                                    payer.set(1);
                                    copied.set(false);
                                    message.set(String::new());
                                })
                                .text("Recommencer"),
                        ),
                ),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_simulator_renders_the_example() {
        let html = simulator("").render();
        assert!(html.contains("Alice"));
        assert!(html.contains("Chloé"));
        assert!(html.contains("rembourse"));
        assert!(html.contains("45,00\u{a0}€"));
        assert!(html.contains("+60,00\u{a0}€"));
    }

    #[test]
    fn the_scenes_render_an_empty_canvas() {
        assert!(coins("").render().starts_with("<canvas"));
        assert!(flow("").render().starts_with("<canvas"));
    }

    #[test]
    fn initials() {
        assert_eq!(initial("élodie"), "É");
        assert_eq!(initial(""), "");
    }
}
