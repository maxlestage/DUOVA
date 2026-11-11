//! The browser bundle: hydrates the islands, starts the 2D scenes and the page motion.

pub mod canvas;
pub mod coins;
pub mod flow;
pub mod motion;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&crate::islands::ISLANDS);
    motion::init();
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

pub fn document() -> web_sys::Document {
    window().document().expect("document")
}

pub fn matches(query: &str) -> bool {
    window()
        .match_media(query)
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

pub fn reduced_motion() -> bool {
    matches("(prefers-reduced-motion: reduce)")
}

/// Puts `text` in the clipboard (the recap of the simulator).
pub fn copy(text: &str) {
    let _ = window().navigator().clipboard().write_text(text);
}

/// Adds an event listener for the page's lifetime.
pub fn on<E: JsCast + 'static>(
    target: &web_sys::EventTarget,
    event: &str,
    f: impl FnMut(E) + 'static,
) {
    let mut f = f;
    let closure =
        Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| f(e.unchecked_into()));
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(true);
    let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
        event,
        closure.as_ref().unchecked_ref(),
        &options,
    );
    closure.forget();
}

type FrameCallback = Closure<dyn FnMut(f64)>;

/// Calls `f(timestamp)` on every animation frame while it returns `true`.
pub fn request_frame(f: impl FnMut(f64) -> bool + 'static) {
    let f = Rc::new(RefCell::new(f));
    let slot: Rc<RefCell<Option<FrameCallback>>> = Rc::new(RefCell::new(None));
    let again = slot.clone();
    *slot.borrow_mut() = Some(Closure::new(move |now: f64| {
        if (f.borrow_mut())(now) {
            if let Some(cb) = again.borrow().as_ref() {
                let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
            }
        } else {
            again.borrow_mut().take();
        }
    }));
    if let Some(cb) = slot.borrow().as_ref() {
        let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
    }
}

/// Every element matching `selector`.
pub fn all(selector: &str) -> Vec<web_sys::HtmlElement> {
    let Ok(list) = document().query_selector_all(selector) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i)?.dyn_into().ok())
        .collect()
}

/// A flag that says whether `element` is on screen, kept up to date by an
/// `IntersectionObserver`: the 2D scenes draw nothing while they are scrolled away.
pub fn visibility(element: &web_sys::Element) -> Rc<Cell<bool>> {
    let visible = Rc::new(Cell::new(true));
    let flag = visible.clone();
    let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
        for entry in entries.iter() {
            if let Ok(entry) = entry.dyn_into::<web_sys::IntersectionObserverEntry>() {
                flag.set(entry.is_intersecting());
            }
        }
    });
    let options = web_sys::IntersectionObserverInit::new();
    options.set_root_margin("120px 0px");
    if let Ok(observer) =
        web_sys::IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options)
    {
        observer.observe(element);
    }
    callback.forget();
    visible
}

/// A small, fast random number generator (xorshift), seeded by the clock.
pub struct Rng(u64);

impl Rng {
    pub fn new() -> Rng {
        let seed = (js_sys::Date::now() as u64) ^ 0x9e37_79b9_7f4a_7c15;
        Rng(seed.max(1))
    }

    /// A number in `[0, 1)`.
    pub fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A number in `[a, b)`.
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.next()
    }
}
