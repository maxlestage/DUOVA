//! `duova-site export <dir>`: the whole site as static files, for GitHub Pages or any
//! static host. Every file comes from the server itself, through Paquets' in-memory test
//! client (`app.test()`), so both are identical.
//!
//! A project site on GitHub Pages lives under a path (`/DUOVA`): every absolute link of
//! the pages, of the manifest and of the service worker gets it (`BASE_PATH`), and the
//! absolute URLs (canonical, Open Graph, sitemap) use the public address (`SITE_URL`).

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{Site, app, site_dir};

/// The public address used when `SITE_URL` is not set.
pub fn default_site_url(base: &str) -> String {
    if base.is_empty() {
        "http://localhost:3000".to_owned()
    } else {
        format!("https://maxlestage.github.io{base}")
    }
}

/// `"/DUOVA/"`, `"DUOVA"`, `"/"` → `"/DUOVA"`, `"/DUOVA"`, `""`.
pub fn normalize_base(base: &str) -> String {
    let trimmed = base.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

/// The site behind `base`: the attributes (`href="/…"`, `src="/…"`, `content="/…"`) and
/// the URLs Paquets writes in its scripts (the islands loader, the service worker).
pub fn with_base(html: &str, base: &str) -> String {
    if base.is_empty() {
        return html.to_owned();
    }
    html.replace("=\"/", &format!("=\"{base}/"))
        .replace("\"/pkg/", &format!("\"{base}/pkg/"))
        .replace("\"/sw.js\"", &format!("\"{base}/sw.js\""))
}

/// Every absolute URL of a JSON or JavaScript file (`"/…"`) behind `base`.
pub fn with_base_everywhere(text: &str, base: &str) -> String {
    if base.is_empty() {
        return text.to_owned();
    }
    text.replace("\"/", &format!("\"{base}/"))
}

/// Writes the site into `out`. Returns the number of files written.
pub async fn export(out: &Path, base: &str, site_url: Option<String>) -> io::Result<usize> {
    let base = normalize_base(base);
    let out = std::path::absolute(out)?;
    // Paquets finds `pkg/` (the browser bundle) in the current directory.
    let dir = std::path::absolute(site_dir())?;
    std::env::set_current_dir(&dir)?;
    if !dir.join("pkg").is_dir() {
        eprintln!("warning: site/pkg is missing: no 2D scenes and no simulator (see README.md)");
    }
    let site_url = site_url
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| default_site_url(&base));
    let mut app = app(Arc::new(Site::load(Some(site_url))));
    // Static files have no server to protect, and the export asks for every file at once
    // (the rate limit would stop it).
    app.security(None);
    let client = app.test().await;

    let mut written = 0;
    let mut write = |path: &str, bytes: &[u8]| -> io::Result<()> {
        let file = out.join(path.trim_start_matches('/'));
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, bytes)?;
        written += 1;
        Ok(())
    };
    let get = |path: &'static str, expected: u16| {
        let client = &client;
        async move {
            let res = client.get(path).await;
            if res.status().as_u16() != expected {
                return Err(io::Error::other(format!(
                    "GET {path}: {} instead of {expected}",
                    res.status()
                )));
            }
            Ok(res.text())
        }
    };

    write(
        "index.html",
        with_base(&get("/", 200).await?, &base).as_bytes(),
    )?;
    // GitHub Pages serves 404.html for every unknown address.
    let missing = get("/page-introuvable", 404).await?;
    write("404.html", with_base(&missing, &base).as_bytes())?;
    for file in ["/sitemap.xml", "/robots.txt"] {
        write(file, get(file, 200).await?.as_bytes())?;
    }
    for file in ["/manifest.webmanifest", "/sw.js"] {
        let body = get(file, 200).await?;
        write(file, with_base_everywhere(&body, &base).as_bytes())?;
    }
    for folder in ["public", "pkg"] {
        let root = dir.join(folder);
        let prefix = if folder == "pkg" { "pkg/" } else { "" };
        for file in files(&root) {
            let relative = file.strip_prefix(&root).map_err(io::Error::other)?;
            let url = format!("{prefix}{}", relative.to_string_lossy().replace('\\', "/"));
            // wasm-bindgen's own files, not needed to run the bundle.
            if url.ends_with(".gitignore") || url.ends_with("package.json") {
                continue;
            }
            write(&url, &std::fs::read(&file)?)?;
        }
    }
    // Files are served as they are: no Jekyll processing on GitHub Pages.
    write(".nojekyll", b"")?;
    Ok(written)
}

/// Every file under `dir`, recursively.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_go_behind_the_base_path() {
        let html = r##"<link href="/site.css"/><a href="/">x</a><link rel="canonical" href="https://a.b/"/><script type="module">import init from "/pkg/a.js";init({module_or_path:"/pkg/a_bg.wasm"});navigator.serviceWorker.register("/sw.js")</script><a href="#faq">"/"</a>"##;
        let out = with_base(html, "/DUOVA");
        assert!(out.contains(r#"href="/DUOVA/site.css""#));
        assert!(out.contains(r#"href="/DUOVA/""#));
        assert!(out.contains(r#"href="https://a.b/""#));
        assert_eq!(out.matches("\"/DUOVA/pkg/").count(), 2);
        assert!(out.contains(r#""/DUOVA/sw.js""#));
        assert!(out.contains(r##"href="#faq""##));
        assert_eq!(with_base(html, ""), html);
    }

    #[test]
    fn base_paths() {
        assert_eq!(normalize_base("/DUOVA/"), "/DUOVA");
        assert_eq!(normalize_base("DUOVA"), "/DUOVA");
        assert_eq!(normalize_base("/"), "");
        assert_eq!(normalize_base(""), "");
        assert_eq!(
            default_site_url("/DUOVA"),
            "https://maxlestage.github.io/DUOVA"
        );
        assert_eq!(
            with_base_everywhere(r#"{"start_url":"/","src":"/assets/i.png"}"#, "/DUOVA"),
            r#"{"start_url":"/DUOVA/","src":"/DUOVA/assets/i.png"}"#
        );
    }
}
