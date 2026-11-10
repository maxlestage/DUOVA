//! The pages: the home page of Duova and the "not found" page, rendered on the server
//! with their SEO tags. The 3D scenes and the simulator are islands; the rest is HTML
//! and CSS (3D transforms, animations), driven by `client::motion` in the browser.

use paquets::active::{Hydrate, island_with};
use paquets::prelude::*;

use crate::islands;
use crate::split;

pub const NAME: &str = "Duova";
pub const TAGLINE: &str = "Qui doit quoi ?";
pub const DESCRIPTION: &str = "Duova fait les comptes de vos week-ends, colocs et vacances au centime près, \
     et vous dit qui rembourse qui en un minimum de virements. Une app iPhone, sans compte ni serveur.";

/// What the pages need from the request.
pub struct Ctx {
    /// The public address of the site, without a trailing slash.
    pub origin: String,
    /// `?v=` of the stylesheet, so that a new version is downloaded at once.
    pub css_version: String,
}

const LOGO: &str = r##"<svg viewBox="0 0 48 48" width="32" height="32" aria-hidden="true" focusable="false"><defs><linearGradient id="lg-a" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#ffc145"/><stop offset="1" stop-color="#ff7a59"/></linearGradient><linearGradient id="lg-b" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#7c5cff"/><stop offset="1" stop-color="#21c7a8"/></linearGradient></defs><circle cx="18" cy="24" r="14" fill="url(#lg-b)"/><circle cx="30" cy="24" r="14" fill="url(#lg-a)" fill-opacity=".92"/><path d="M24 12.6a14 14 0 0 1 0 22.8 14 14 0 0 1 0-22.8z" fill="#fff" fill-opacity=".35"/></svg>"##;

fn logo() -> Element {
    span().class("logo__mark").html(LOGO)
}

/// The page around `content`.
fn layout(ctx: &Ctx, title: &str, description: &str, content: Element, index: bool) -> Page {
    let mut seo = Seo::new()
        .site_name(NAME)
        .base_url(ctx.origin.clone())
        .image("/assets/og-image.png")
        .meta_property("og:image:width", "1200")
        .meta_property("og:image:height", "630")
        .meta_property(
            "og:image:alt",
            "Duova : qui doit quoi ? Les comptes du groupe, au centime près.",
        )
        .og_type("website")
        .locale("fr_FR")
        .meta_name("theme-color", "#0b0916")
        .meta_name("apple-mobile-web-app-title", NAME);
    if index {
        seo = seo.canonical("/").json_ld(json_ld(ctx));
    } else {
        seo = seo.noindex(true);
    }
    let head = r##"<meta name="color-scheme" content="dark"/><link rel="icon" href="/assets/favicon.svg" type="image/svg+xml"/><link rel="apple-touch-icon" href="/assets/favicon.svg"/>"##;
    Page::new(title)
        .lang("fr")
        .description(description)
        .seo(seo)
        .plain()
        .stylesheet(format!("/site.css?v={}", ctx.css_version))
        .head(head)
        .body(
            div()
                .class("site")
                .child(a().class("skip").href("#contenu").text("Aller au contenu"))
                .child(div().class("progress").attr("aria-hidden", "true"))
                .child(nav_bar())
                .child(content.id("contenu"))
                .child(footer_bar())
                .child(div().class("grain").attr("aria-hidden", "true")),
        )
}

fn json_ld(ctx: &Ctx) -> String {
    let value = paquets::serde_json::json!({
        "@context": "https://schema.org",
        "@type": "MobileApplication",
        "name": NAME,
        "alternateName": TAGLINE,
        "description": DESCRIPTION,
        "operatingSystem": "iOS 27",
        "applicationCategory": "FinanceApplication",
        "inLanguage": "fr",
        "url": format!("{}/", ctx.origin),
        "offers": { "@type": "Offer", "price": "0", "priceCurrency": "EUR" }
    });
    value.to_string()
}

fn nav_bar() -> Element {
    header().class("nav").child(
        div()
            .class("nav__inner")
            .child(
                a().class("logo")
                    .href("/")
                    .attr("aria-label", "Duova, accueil")
                    .child(logo())
                    .child(span().class("logo__word").text(NAME)),
            )
            .child(
                nav()
                    .class("nav__links")
                    .attr("aria-label", "Sections")
                    .child(a().href("#fonctionnement").text("Fonctionnement"))
                    .child(a().href("#simulateur").text("Simulateur"))
                    .child(a().href("#iphone").text("iPhone"))
                    .child(a().href("#faq").text("FAQ")),
            )
            .child(
                a().class("btn btn--small btn--primary nav__cta")
                    .href("#telecharger")
                    .text("L'app"),
            ),
    )
}

fn footer_bar() -> Element {
    footer().class("footer").child(
        div()
            .class("footer__inner")
            .child(
                div()
                    .class("footer__brand")
                    .child(
                        a().class("logo")
                            .href("/")
                            .child(logo())
                            .child(span().class("logo__word").text(NAME)),
                    )
                    .child(p().text("Les comptes du groupe, au centime près.")),
            )
            .child(
                p().class("footer__made")
                    .text("Site écrit en Rust avec Paquets · 3D en WebGL 2 · App iOS en Swift."),
            )
            .child(p().class("footer__copy").text("© 2026 Duova")),
    )
}

/// The words of a title, each in its own span, so that they rise one after the other.
fn words(text: &str) -> Vec<Element> {
    text.split(' ')
        .enumerate()
        .map(|(i, w)| {
            span()
                .class("word")
                .style(format!("--i:{i}"))
                .child(span().class("word__inner").text(w.to_owned()))
        })
        .collect()
}

fn section_head(eyebrow: &str, title: &str, lead: &str) -> Element {
    div()
        .class("section__head")
        .attr("data-reveal", "")
        .child(p().class("eyebrow").text(eyebrow.to_owned()))
        .child(h2().class("section__title").text(title.to_owned()))
        .child(p().class("section__lead").text(lead.to_owned()))
}

// ----- Home -----------------------------------------------------------------------------

pub fn home(ctx: &Ctx) -> Page {
    let content = main()
        .class("home")
        .child(hero())
        .child(marquee())
        .child(steps())
        .child(algorithm())
        .child(simulator())
        .child(features())
        .child(iphone())
        .child(faq())
        .child(download());
    layout(
        ctx,
        &format!("{NAME} · {TAGLINE} Le partage de dépenses au centime près"),
        DESCRIPTION,
        content,
        true,
    )
}

fn hero() -> Element {
    section()
        .class("hero")
        .attr("aria-labelledby", "hero-title")
        .child(div().class("hero__aurora").attr("aria-hidden", "true"))
        .child(div().class("hero__grid").attr("aria-hidden", "true"))
        .child(div().class("hero__scene").child(island("coins", "", islands::coins)))
        .child(
            div()
                .class("hero__content")
                .child(
                    p().class("pill")
                        .child(span().class("pill__dot"))
                        .text("Partage de dépenses · iPhone"),
                )
                .child(h1().class("hero__title").id("hero-title").children(words(TAGLINE)))
                .child(p().class("hero__lead").text(
                    "Duova fait les comptes du groupe au centime près et vous dit qui rembourse qui, \
                     en un minimum de virements.",
                ))
                .child(
                    div()
                        .class("hero__actions")
                        .child(
                            a().class("btn btn--primary btn--large")
                                .href("#simulateur")
                                .text("Essayer en direct"),
                        )
                        .child(
                            a().class("btn btn--ghost btn--large")
                                .href("#iphone")
                                .text("Voir l'app iPhone"),
                        ),
                )
                .child(
                    p().class("hero__hint")
                        .attr("aria-hidden", "true")
                        .text("Touchez les pièces ✦"),
                ),
        )
        .child(
            div()
                .class("hero__stats")
                .child(stat("0", "", "serveur : tout reste sur l'iPhone"))
                .child(stat("1", " ct", "de précision, jamais d'arrondi perdu"))
                .child(stat("2", "", "virements suffisent pour 3 personnes")),
        )
}

fn stat(value: &str, unit: &str, label: &str) -> Element {
    div()
        .class("stat")
        .child(
            strong()
                .class("stat__value")
                .child(span().text(value.to_owned()))
                .text(unit.to_owned()),
        )
        .child(span().class("stat__label").text(label.to_owned()))
}

fn marquee() -> Element {
    let groups = [
        "🏔️ Ski à Chamonix",
        "🏠 Coloc rue Oberkampf",
        "🌊 Vacances à Biarritz",
        "🎂 Anniversaire de Léa",
        "🍷 Week-end à Lyon",
        "🎪 Festival d'été",
        "🚐 Road trip en Bretagne",
        "🍕 Soirée pizza",
    ];
    let row = || {
        ul().class("marquee__row")
            .children(groups.iter().map(|g| li().text(*g)))
    };
    section()
        .class("marquee")
        .attr("aria-label", "Exemples de groupes")
        .child(
            div()
                .class("marquee__track")
                .child(row())
                .child(row().attr("aria-hidden", "true")),
        )
}

fn steps() -> Element {
    let items = [
        (
            "👥",
            "Créez un groupe",
            "« Week-end à Lyon », « Coloc »… autant de groupes que vous voulez.",
        ),
        (
            "➕",
            "Ajoutez les membres",
            "Un prénom suffit. Personne n'a besoin d'installer l'app.",
        ),
        (
            "🧾",
            "Saisissez les dépenses",
            "Description, montant, qui a payé et pour qui. En trois secondes.",
        ),
        (
            "💸",
            "Remboursez",
            "Duova calcule les soldes et la liste la plus courte de virements.",
        ),
    ];
    section()
        .class("section steps")
        .id("fonctionnement")
        .child(section_head(
            "Fonctionnement",
            "Quatre gestes, zéro tableur.",
            "Fini les calculs sur un coin de nappe : chacun voit ce qu'il a payé, sa part, et ce qu'il doit.",
        ))
        .child(ol().class("steps__list").children(items.iter().enumerate().map(
            |(i, (icon, title, text))| {
                li().class("step glass")
                    .attr("data-reveal", "")
                    .attr("data-tilt", "")
                    .style(format!("--i:{i}"))
                    .child(span().class("step__num").text(format!("0{}", i + 1)))
                    .child(span().class("step__icon").attr("aria-hidden", "true").text(*icon))
                    .child(h3().text(*title))
                    .child(p().text(*text))
            },
        )))
}

fn algorithm() -> Element {
    let (members, expenses) = split::example();
    let balances = split::balances(&members, &expenses);
    let transfers = split::settlements(&balances);
    let paid = |id: u32| -> i64 {
        expenses
            .iter()
            .filter(|e| e.paid_by == id)
            .map(|e| e.amount)
            .sum()
    };
    let share = |id: u32| paid(id) - balances[&id];
    let name = |id: u32| {
        members
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.name.clone())
            .unwrap_or_default()
    };
    let signed = |b: i64| {
        if b > 0 {
            format!("+{}", split::format_cents(b))
        } else {
            split::format_cents(b)
        }
    };
    section()
        .class("section algo")
        .child(section_head(
            "L'algorithme",
            "Le moins de virements possible.",
            "Chacun a un solde : ce qu'il a payé moins sa part. Puis, à chaque tour, le plus gros débiteur \
             rembourse le plus gros créancier. Au plus n − 1 virements pour n personnes.",
        ))
        .child(
            div()
                .class("algo__grid")
                .child(
                    figure()
                        .class("flow glass")
                        .attr("data-reveal", "")
                        .child(island_with("flow", "", islands::flow, Hydrate::Visible))
                        .child(
                            figcaption().class("flow__legend").children(members.iter().enumerate().map(
                                |(i, m)| {
                                    span().class("legend")
                                        .style(format!("--c:{}", islands::COLORS[i]))
                                        .text(format!("{} {}", m.name, signed(balances[&m.id])))
                                },
                            )),
                        ),
                )
                .child(
                    div()
                        .class("algo__text")
                        .attr("data-reveal", "")
                        .child(p().class("algo__story").text(
                            "Alice paie 90 € de courses pour Alice, Bob et Chloé. \
                             Bob paie 30 € d'essence pour Bob et Chloé.",
                        ))
                        .child(
                            div().class("table-wrap").child(
                                table()
                                    .class("ledger")
                                    .child(caption().class("sr-only").text("Soldes de l'exemple"))
                                    .child(thead().child(
                                        tr().child(th().attr("scope", "col").text("Personne"))
                                            .child(th().attr("scope", "col").text("A payé"))
                                            .child(th().attr("scope", "col").text("Sa part"))
                                            .child(th().attr("scope", "col").text("Solde")),
                                    ))
                                    .child(tbody().children(members.iter().enumerate().map(
                                        |(i, m)| {
                                            let b = balances[&m.id];
                                            tr().style(format!("--c:{}", islands::COLORS[i]))
                                                .child(th().attr("scope", "row").child(span().class("dot")).text(m.name.clone()))
                                                .child(td().text(split::format_cents(paid(m.id))))
                                                .child(td().text(split::format_cents(share(m.id))))
                                                .child(
                                                    td().class(if b >= 0 { "plus" } else { "minus" })
                                                        .text(signed(b)),
                                                )
                                        },
                                    ))),
                            ),
                        )
                        .child(ol().class("algo__transfers").children(transfers.iter().map(|t| {
                            li().child(strong().text(name(t.from)))
                                .text(" → ")
                                .child(strong().text(name(t.to)))
                                .child(span().text(split::format_cents(t.amount)))
                        })))
                        .child(p().class("algo__done").text(format!(
                            "{} virements, et tout le monde est quitte.",
                            transfers.len()
                        ))),
                ),
        )
}

fn simulator() -> Element {
    section()
        .class("section simulator")
        .id("simulateur")
        .child(section_head(
            "Simulateur",
            "Essayez, c'est le vrai calcul.",
            "Le même code que l'app, compilé de Rust en WebAssembly : ajoutez des amis, des dépenses, \
             et regardez les remboursements se recalculer.",
        ))
        .child(
            div()
                .class("simulator__frame")
                .attr("data-reveal", "")
                .child(island_with(
                    "simulator",
                    "Week-end à Lyon",
                    islands::simulator,
                    Hydrate::Visible,
                )),
        )
}

fn features() -> Element {
    let items: [(&str, &str, &str, &str); 6] = [
        (
            "feature--wide",
            "🎯",
            "Au centime près",
            "Les montants sont des centimes entiers : jamais de 0,1 + 0,2 = 0,30000000000000004. \
             10 € pour 3 ? 3,34 €, 3,33 € et 3,33 € : pas un centime ne se perd.",
        ),
        (
            "",
            "🔒",
            "Sans compte",
            "Ni inscription, ni serveur : vos groupes restent sur votre iPhone.",
        ),
        (
            "",
            "💬",
            "Récap à partager",
            "Un bouton, et le récap part sur WhatsApp, Messages ou par e-mail.",
        ),
        (
            "",
            "🗂️",
            "Plusieurs groupes",
            "Coloc, vacances, sorties : chaque groupe a ses membres et ses comptes.",
        ),
        (
            "",
            "⚡",
            "n − 1 virements",
            "L'algorithme glouton apparie les plus grosses dettes aux plus grosses créances.",
        ),
        (
            "feature--full",
            "📱",
            "Natif, en Swift",
            "SwiftUI, Swift 6.4 et iOS 27 : rapide, accessible, en mode sombre comme en clair, \
             avec le partage et les animations du système.",
        ),
    ];
    section()
        .class("section features")
        .child(section_head(
            "Pourquoi Duova",
            "Les bons comptes font les bons amis.",
            "Tout ce qu'il faut pour partager les frais, rien de ce qui complique.",
        ))
        .child(
            div().class("bento").children(items.iter().enumerate().map(
                |(i, (extra, icon, title, text))| {
                    article()
                        .class("feature glass")
                        .class(*extra)
                        .attr("data-reveal", "")
                        .attr("data-tilt", "")
                        .style(format!("--i:{i}"))
                        .child(span().class("feature__glow").attr("aria-hidden", "true"))
                        .child(
                            span()
                                .class("feature__icon")
                                .attr("aria-hidden", "true")
                                .text(*icon),
                        )
                        .child(h3().text(*title))
                        .child(p().text(*text))
                },
            )),
        )
}

/// The iPhone, in CSS 3D: a body with thickness, a screen showing the app, turning
/// with the scroll.
fn iphone() -> Element {
    let screen = div()
        .class("app")
        .child(
            div()
                .class("app__status")
                .child(span().text("9:41"))
                .child(span().class("app__island")),
        )
        .child(p().class("app__back").text("‹ Groupes"))
        .child(h4().class("app__title").text("Week-end à Lyon"))
        .child(p().class("app__total").text("Total · 120,00 €"))
        .child(
            ul().class("app__balances")
                .child(app_balance("A", "Alice", "+60,00 €", "plus", 0))
                .child(app_balance("B", "Bob", "−15,00 €", "minus", 1))
                .child(app_balance("C", "Chloé", "−45,00 €", "minus", 2)),
        )
        .child(p().class("app__label").text("Remboursements"))
        .child(
            ul().class("app__transfers")
                .child(li().text("Chloé → Alice").child(strong().text("45,00 €")))
                .child(li().text("Bob → Alice").child(strong().text("15,00 €"))),
        )
        .child(div().class("app__share").text("Partager le récap"));
    section()
        .class("section iphone")
        .id("iphone")
        .child(
            div()
                .class("iphone__grid")
                .child(
                    div()
                        .class("iphone__stage")
                        .attr("data-scroll3d", "")
                        .attr("aria-hidden", "true")
                        .child(div().class("iphone__shadow"))
                        .child(
                            div()
                                .class("phone")
                                .child(div().class("phone__side phone__side--right"))
                                .child(div().class("phone__side phone__side--left"))
                                .child(div().class("phone__back"))
                                .child(div().class("phone__front").child(screen))
                                .child(div().class("phone__glare")),
                        )
                        .child(div().class("orbit orbit--a").child(span().class("chip").text("+60,00 €")))
                        .child(div().class("orbit orbit--b").child(span().class("chip chip--violet").text("2 virements")))
                        .child(div().class("orbit orbit--c").child(span().class("chip chip--green").text("✓ Quitte"))),
                )
                .child(
                    div()
                        .class("iphone__text")
                        .attr("data-reveal", "")
                        .child(p().class("eyebrow").text("L'app iPhone"))
                        .child(h2().class("section__title").text("Pensée pour l'iPhone, écrite en Swift."))
                        .child(p().class("section__lead").text(
                            "Une app native en SwiftUI : vos groupes s'ouvrent instantanément, \
                             même en mode avion, et le récap se partage avec la feuille de partage d'iOS.",
                        ))
                        .child(
                            ul().class("checks")
                                .child(li().text("SwiftUI et Observation, Swift 6.4"))
                                .child(li().text("iOS 27, iPhone et iPad"))
                                .child(li().text("Données enregistrées sur l'appareil"))
                                .child(li().text("Même algorithme que ce site, testé au centime"))
                                .child(li().text("VoiceOver, Dynamic Type, mode sombre")),
                        ),
                ),
        )
}

fn app_balance(initial: &str, name: &str, amount: &str, sign: &str, i: usize) -> Element {
    li().style(format!("--c:{}", islands::COLORS[i]))
        .child(span().class("avatar").text(initial.to_owned()))
        .child(span().class("app__name").text(name.to_owned()))
        .child(strong().class(sign.to_owned()).text(amount.to_owned()))
}

fn faq() -> Element {
    let items = [
        (
            "Est-ce que tout le monde doit installer l'app ?",
            "Non. Une seule personne tient les comptes du groupe et partage le récap ; \
             les autres n'ont rien à installer.",
        ),
        (
            "Où sont stockées mes données ?",
            "Sur votre iPhone, et nulle part ailleurs : pas de compte, pas de serveur, pas de publicité.",
        ),
        (
            "Comment les centimes sont-ils répartis ?",
            "Les montants sont gardés en centimes entiers. Quand une dépense ne se divise pas \
             exactement, les premiers participants prennent un centime de plus : la somme des parts \
             est toujours exactement le montant.",
        ),
        (
            "Pourquoi si peu de virements ?",
            "Duova ne rembourse pas dépense par dépense : il calcule le solde de chacun, puis apparie \
             les plus grosses dettes aux plus grosses créances. Au plus n − 1 virements pour n personnes.",
        ),
        (
            "Et si quelqu'un ne participe pas à une dépense ?",
            "Pour chaque dépense, vous choisissez qui en profite : seules ces personnes en paient une part.",
        ),
    ];
    section()
        .class("section faq")
        .id("faq")
        .child(section_head(
            "Questions",
            "Tout ce que vous vous demandez.",
            "Et si la réponse n'est pas là, c'est que la question n'a pas encore été posée.",
        ))
        .child(
            div()
                .class("faq__list")
                .children(items.iter().map(|(q, a)| {
                    details()
                        .class("qa glass")
                        .attr("data-reveal", "")
                        .child(
                            summary()
                                .child(span().text(*q))
                                .child(span().class("qa__plus").attr("aria-hidden", "true")),
                        )
                        .child(p().text(*a))
                })),
        )
}

fn download() -> Element {
    let coin = div()
        .class("coin3d")
        .attr("aria-hidden", "true")
        .child(div().class("coin3d__face coin3d__face--front").text("€"))
        .children((0..10).map(|i| span().class("coin3d__edge").style(format!("--k:{i}"))))
        .child(div().class("coin3d__face coin3d__face--back").text("D"));
    section()
        .class("section download")
        .id("telecharger")
        .child(
            div()
                .class("download__card glass")
                .attr("data-reveal", "")
                .child(div().class("download__coin").child(coin))
                .child(h2().class("section__title").text("Les comptes sont bons."))
                .child(p().class("section__lead").text(
                    "Duova arrive sur l'App Store. En attendant, le simulateur ci-dessus fait exactement les mêmes calculs.",
                ))
                .child(
                    div()
                        .class("hero__actions")
                        .child(
                            span()
                                .class("btn btn--primary btn--large btn--store")
                                .attr("aria-disabled", "true")
                                .text("Bientôt sur l'App Store"),
                        )
                        .child(a().class("btn btn--ghost btn--large").href("#simulateur").text("Refaire un calcul")),
                ),
        )
}

// ----- Not found ------------------------------------------------------------------------

pub fn not_found(ctx: &Ctx) -> Page {
    let content = main()
        .class("nf")
        .child(div().class("hero__aurora").attr("aria-hidden", "true"))
        .child(
            div()
                .class("nf__box")
                .child(p().class("nf__code").text("404"))
                .child(
                    h1().class("section__title")
                        .text("Cette page ne doit rien à personne."),
                )
                .child(
                    p().class("section__lead")
                        .text("Elle n'existe pas, tout simplement."),
                )
                .child(
                    a().class("btn btn--primary btn--large")
                        .href("/")
                        .text("Retour à l'accueil"),
                ),
        );
    layout(
        ctx,
        &format!("Page introuvable · {NAME}"),
        DESCRIPTION,
        content,
        false,
    )
    .status(404)
}
