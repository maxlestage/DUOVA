//! `cargo run`: the website on `http://localhost:3000` (`PORT` on Heroku). Run it from
//! `site/`, after building the browser bundle (see `README.md`).
//!
//! `cargo run -- export dist`: the whole site as static files in `dist/`, for GitHub
//! Pages. `BASE_PATH` (e.g. `/DUOVA`) prefixes every link, `SITE_URL` (e.g.
//! `https://maxlestage.github.io/DUOVA`) makes the absolute URLs.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("export") => {
            let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| "dist".into()));
            // The output folder is relative to where the command starts, not to `site/`.
            let dir = std::path::absolute(dir)?;
            let base = std::env::var("BASE_PATH").unwrap_or_default();
            let site_url = std::env::var("SITE_URL").ok();
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            let files =
                runtime.block_on(duova_site::server::export::export(&dir, &base, site_url))?;
            println!("📦 {files} files written to {}", dir.display());
            Ok(())
        }
        Some(other) => {
            eprintln!("unknown command {other:?}: run without arguments, or `export <dir>`");
            std::process::exit(2);
        }
        None => duova_site::server::run(),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
