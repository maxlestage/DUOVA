//! The whole site through Paquets' in-memory test client: every route, the SEO tags,
//! the islands, the Content-Security-Policy and the "no sideways scroll" rules.

use std::sync::Arc;

use duova_site::server::{Site, app};

fn site() -> Arc<Site> {
    Arc::new(Site::load(Some("https://duova.example".into())))
}

#[tokio::test]
async fn the_home_page() {
    let client = app(site()).test().await;
    let res = client.get("/").await;
    assert_eq!(res.status(), 200);
    let csp = res
        .header("content-security-policy")
        .expect("a Content-Security-Policy")
        .to_owned();
    assert!(csp.contains("'wasm-unsafe-eval'"), "{csp}");
    let html = res.text();
    assert!(html.contains(r#"<html lang="fr""#));
    assert!(html.contains("<title>Duova · Qui doit quoi ?"));
    assert!(html.contains(r#"<link rel="canonical" href="https://duova.example/"/>"#));
    assert!(
        html.contains(r#"<meta name="viewport" content="width=device-width, initial-scale=1"/>"#)
    );
    assert!(html.contains("application/ld+json"));
    // The three islands: the two 2D scenes and the simulator.
    for name in ["coins", "flow", "simulator"] {
        assert!(
            html.contains(&format!(r#"data-island="{name}""#)),
            "island {name}"
        );
    }
    // The example, computed by the server.
    assert!(html.contains("+60,00\u{a0}€"));
    assert!(html.contains("−45,00\u{a0}€"));
    assert!(html.contains("2 virements, et tout le monde est quitte."));
    // Every section the navigation points to.
    for id in [
        "fonctionnement",
        "simulateur",
        "iphone",
        "faq",
        "telecharger",
        "contenu",
    ] {
        assert!(html.contains(&format!(r#"id="{id}""#)), "#{id}");
    }
}

#[tokio::test]
async fn an_unknown_page() {
    let client = app(site()).test().await;
    let res = client.get("/nulle-part").await;
    assert_eq!(res.status(), 404);
    let html = res.text();
    assert!(html.contains("404"));
    assert!(html.contains(r#"name="robots" content="noindex"#));
}

#[tokio::test]
async fn sitemap_and_robots() {
    let client = app(site()).test().await;
    let sitemap = client.get("/sitemap.xml").await;
    assert_eq!(sitemap.status(), 200);
    assert!(sitemap.text().contains("<loc>https://duova.example/</loc>"));
    let robots = client.get("/robots.txt").await.text();
    assert!(robots.contains("Sitemap: https://duova.example/sitemap.xml"));
}

#[test]
fn the_stylesheet_never_scrolls_sideways() {
    let css = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/public/site.css"))
        .expect("public/site.css");
    // The page clips what goes past its sides, mobile first (no max-width query).
    assert!(css.contains("html {\n  overflow-x: clip;"));
    assert!(
        css.contains(
            "body {\n  margin: 0;\n  width: 100%;\n  max-width: 100%;\n  overflow-x: clip;"
        )
    );
    assert!(
        !css.contains("@media (max-width"),
        "mobile first: widen with min-width"
    );
    assert!(
        !css.contains("100vw"),
        "100vw counts the scrollbar and overflows"
    );
    // Motion can be turned off.
    assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
}

#[test]
fn the_stylesheet_is_2d_only() {
    let css = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/public/site.css"))
        .expect("public/site.css");
    for word in [
        "perspective",
        "rotateX",
        "rotateY",
        "translateZ",
        "translate3d",
        "rotate3d",
        "scale3d",
        "matrix3d",
        "preserve-3d",
        "backface-visibility",
    ] {
        assert!(!css.contains(word), "no 3D: {word}");
    }
}

#[tokio::test]
async fn the_static_export_for_github_pages() {
    let out = std::env::temp_dir().join(format!("duova-export-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let files = duova_site::server::export::export(
        &out,
        "/DUOVA",
        Some("https://maxlestage.github.io/DUOVA".into()),
    )
    .await
    .expect("export");
    assert!(files >= 10, "{files} files");
    let read = |path: &str| std::fs::read_to_string(out.join(path)).expect(path);

    let index = read("index.html");
    assert!(index.contains(r#"href="/DUOVA/site.css?v="#));
    assert!(index.contains(r#"href="/DUOVA/assets/favicon.svg""#));
    assert!(
        index.contains(r#"<link rel="canonical" href="https://maxlestage.github.io/DUOVA/"/>"#)
    );
    assert!(index.contains("https://maxlestage.github.io/DUOVA/assets/og-image.png"));
    assert!(
        !index.contains(r#"href="/site.css"#),
        "a link without the base path"
    );
    assert!(read("404.html").contains("404"));
    assert!(read("manifest.webmanifest").contains(r#""start_url":"/DUOVA/""#));
    assert!(read("manifest.webmanifest").contains("/DUOVA/assets/icon-512.png"));
    assert!(read("sitemap.xml").contains("<loc>https://maxlestage.github.io/DUOVA/</loc>"));
    assert!(out.join(".nojekyll").is_file());
    assert!(out.join("site.css").is_file());
    assert!(out.join("assets/og-image.png").is_file());
    let _ = std::fs::remove_dir_all(&out);
}
