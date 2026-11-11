//! The server: a `paquets::App` with the home page, the sitemap, `robots.txt` and the
//! "not found" page. Static files come from `public/` and the browser bundle from
//! `pkg/`, both found by Paquets in the current directory.

pub mod export;
pub mod pages;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use paquets::prelude::*;

pub struct Site {
    /// The public address (`SITE_URL`); `None`: taken from each request.
    pub site_url: Option<String>,
    /// Short hash of `public/site.css`, for `?v=` in its link.
    pub css_version: String,
}

/// The `site/` folder: `SITE_DIR`, the current directory when it has `public/`, else
/// the crate folder.
pub fn site_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("SITE_DIR") {
        return PathBuf::from(dir);
    }
    if Path::new("public").is_dir() {
        return PathBuf::from(".");
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// FNV-1a, enough to tell versions of a file apart.
pub fn version(bytes: &[u8]) -> String {
    let h = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{h:016x}")[..10].to_owned()
}

impl Site {
    pub fn load(site_url: Option<String>) -> Site {
        let css = std::fs::read(site_dir().join("public/site.css")).unwrap_or_default();
        Site {
            site_url: site_url
                .map(|u| u.trim().trim_end_matches('/').to_owned())
                .filter(|u| !u.is_empty()),
            css_version: version(&css),
        }
    }

    /// The public address of the site, from `SITE_URL` or from the request.
    fn origin(&self, req: &Request) -> String {
        if let Some(url) = &self.site_url {
            return url.clone();
        }
        let header = |name: &str| {
            req.header(name)
                .map(|v| v.split(',').next().unwrap_or("").trim().to_owned())
        };
        let host = header("x-forwarded-host")
            .or_else(|| header("host"))
            .unwrap_or_default();
        if host.is_empty()
            || !host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
        {
            return "http://localhost".into();
        }
        let proto = if header("x-forwarded-proto").as_deref() == Some("https") {
            "https"
        } else {
            "http"
        };
        format!("{proto}://{host}")
    }

    fn ctx(&self, req: &Request) -> pages::Ctx {
        pages::Ctx {
            origin: self.origin(req),
            css_version: self.css_version.clone(),
        }
    }
}

fn home(site: &Site, req: &Request) -> Response {
    pages::home(&site.ctx(req))
        .into_response()
        .header("cache-control", "no-cache")
}

fn not_found(site: &Site, req: &Request) -> Response {
    pages::not_found(&site.ctx(req)).into_response()
}

fn sitemap(site: &Site, req: &Request) -> Response {
    let origin = site.origin(req);
    Response::new()
        .header("content-type", "application/xml; charset=utf-8")
        .send(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n<url><loc>{origin}/</loc></url>\n</urlset>\n"
        ))
}

fn robots(site: &Site, req: &Request) -> Response {
    let origin = site.origin(req);
    Response::new()
        .header("content-type", "text/plain; charset=utf-8")
        .send(format!(
            "User-agent: *\nAllow: /\n\nSitemap: {origin}/sitemap.xml\n"
        ))
}

/// Routes a handler that only needs the site and the request.
fn route(app: &mut App, path: &str, site: &Arc<Site>, handler: fn(&Site, &Request) -> Response) {
    let site = site.clone();
    app.get(path, move |req: Request| {
        let site = site.clone();
        async move { handler(&site, &req) }
    });
}

/// The application, ready to run or to test.
pub fn app(site: Arc<Site>) -> App {
    let mut app = App::new();
    route(&mut app, "/", &site, home);
    route(&mut app, "/sitemap.xml", &site, sitemap);
    route(&mut app, "/robots.txt", &site, robots);
    // Installable on the home screen, readable offline.
    app.pwa(
        Pwa::new("Duova · Qui doit quoi ?")
            .short_name("Duova")
            .description("Le partage de dépenses au centime près.")
            .start_url("/")
            .theme_color("#0b0916")
            .background_color("#0b0916")
            .categories(["finance", "productivity"])
            .icon("/assets/icon-192.png", "192x192", "image/png")
            .icon("/assets/icon-512.png", "512x512", "image/png")
            .maskable_icon("/assets/icon-maskable-512.png", "512x512", "image/png"),
    );
    let fallback = site.clone();
    app.fallback(move |req: Request| {
        let site = fallback.clone();
        async move { not_found(&site, &req) }
    });
    app
}

/// `cargo run`: the server, on `PORT` (Heroku) or 3000.
pub fn run() -> io::Result<()> {
    let dir = site_dir();
    if dir != Path::new(".") {
        std::env::set_current_dir(&dir)?;
    }
    let site = Arc::new(Site::load(std::env::var("SITE_URL").ok()));
    app(site).run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_change_with_the_content() {
        assert_eq!(version(b"a").len(), 10);
        assert_ne!(version(b"a"), version(b"b"));
    }
}
