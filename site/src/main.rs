//! `cargo run`: the website on `http://localhost:3000` (`PORT` on Heroku). Run it from
//! `site/`, after building the browser bundle (see `README.md`).

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    duova_site::server::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
