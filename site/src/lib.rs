//! The website of Duova ("Qui doit quoi ?"), written in Rust with Paquets: the
//! arithmetic and the islands shared by the server and the browser, the server
//! (native) and the browser bundle (wasm32: islands, 2D scenes, motion).

pub mod islands;
pub mod split;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(target_arch = "wasm32")]
mod client;
