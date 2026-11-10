//! The motion of the page: sections that appear as they come into view, cards that
//! tilt in 3D under the pointer, objects that turn with the scroll, counters that count.
//! Everything is driven by CSS variables; `prefers-reduced-motion` turns it off.

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use super::{all, document, matches, on, reduced_motion, request_frame, window};

pub fn init() {
    let root = document().document_element().expect("html");
    let _ = root.class_list().add_1("motion");
    if reduced_motion() {
        let _ = root.class_list().add_1("still");
        for el in all("[data-reveal]") {
            let _ = el.class_list().add_1("is-in");
        }
        return;
    }
    reveal();
    scroll();
    if matches("(hover: hover) and (pointer: fine)") {
        tilt();
    }
}

/// `[data-reveal]` elements get `is-in` when they enter the screen (once), and their
/// `[data-count]` numbers count up.
fn reveal() {
    let callback = Closure::<dyn FnMut(js_sys::Array, web_sys::IntersectionObserver)>::new(
        |entries: js_sys::Array, observer: web_sys::IntersectionObserver| {
            for entry in entries.iter() {
                let Ok(entry) = entry.dyn_into::<web_sys::IntersectionObserverEntry>() else {
                    continue;
                };
                if !entry.is_intersecting() {
                    continue;
                }
                let target = entry.target();
                let _ = target.class_list().add_1("is-in");
                observer.unobserve(&target);
                if let Ok(list) = target.query_selector_all("[data-count]") {
                    for i in 0..list.length() {
                        if let Some(el) = list.item(i).and_then(|n| n.dyn_into().ok()) {
                            count(el);
                        }
                    }
                }
            }
        },
    );
    let options = web_sys::IntersectionObserverInit::new();
    options.set_threshold(&JsValue::from_f64(0.15));
    options.set_root_margin("0px 0px -8% 0px");
    let Ok(observer) = web_sys::IntersectionObserver::new_with_options(
        callback.as_ref().unchecked_ref(),
        &options,
    ) else {
        for el in all("[data-reveal]") {
            let _ = el.class_list().add_1("is-in");
        }
        return;
    };
    callback.forget();
    for el in all("[data-reveal]") {
        observer.observe(&el);
    }
}

/// `1 234` as French writes it.
fn group_digits(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('\u{202f}');
        }
        out.push(c);
    }
    out
}

/// Counts from 0 to the element's `data-count`, with an ease-out, in 1.4 s.
fn count(el: web_sys::HtmlElement) {
    let Some(target) = el
        .get_attribute("data-count")
        .and_then(|v| v.parse::<f64>().ok())
    else {
        return;
    };
    let start = Rc::new(Cell::new(None::<f64>));
    request_frame(move |now| {
        let t0 = start.get().unwrap_or(now);
        start.set(Some(t0));
        let p = ((now - t0) / 1400.0).min(1.0);
        let eased = 1.0 - (1.0 - p).powi(4);
        el.set_text_content(Some(&group_digits((target * eased).round() as i64)));
        p < 1.0
    });
}

/// The scroll, as CSS variables: `--scroll` (0 → 1) on the page, `--p` (−1 → 1, 0 when
/// centered) on every `[data-scroll3d]` element, and `is-scrolled` on the navigation.
fn scroll() {
    let pending = Rc::new(Cell::new(false));
    let update = || {
        let w = window();
        let y = w.scroll_y().unwrap_or(0.0);
        let height = w
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let root: web_sys::HtmlElement = document()
            .document_element()
            .and_then(|e| e.dyn_into().ok())
            .expect("html");
        let total = (f64::from(root.scroll_height()) - height).max(1.0);
        let _ = root
            .style()
            .set_property("--scroll", &format!("{:.4}", (y / total).clamp(0.0, 1.0)));
        for nav in all(".nav") {
            let _ = nav.class_list().toggle_with_force("is-scrolled", y > 24.0);
        }
        for el in all("[data-scroll3d]") {
            let rect = el.get_bounding_client_rect();
            let center = rect.top() + rect.height() / 2.0;
            let p =
                ((center - height / 2.0) / (height / 2.0 + rect.height() / 2.0)).clamp(-1.0, 1.0);
            let _ = el.style().set_property("--p", &format!("{p:.4}"));
        }
    };
    update();
    let schedule = move || {
        if pending.replace(true) {
            return;
        }
        let pending = pending.clone();
        request_frame(move |_| {
            pending.set(false);
            update();
            false
        });
    };
    on(&window(), "scroll", move |_e: web_sys::Event| schedule());
    on(&window(), "resize", move |_e: web_sys::Event| update());
}

/// `[data-tilt]` cards lean towards the pointer, in 3D, with a light that follows it.
fn tilt() {
    for el in all("[data-tilt]") {
        let target = el.clone();
        on(&el, "pointermove", move |e: web_sys::PointerEvent| {
            let rect = target.get_bounding_client_rect();
            let x = (f64::from(e.client_x()) - rect.left()) / rect.width().max(1.0);
            let y = (f64::from(e.client_y()) - rect.top()) / rect.height().max(1.0);
            let style = target.style();
            let _ = style.set_property("--ry", &format!("{:.2}deg", (x - 0.5) * 14.0));
            let _ = style.set_property("--rx", &format!("{:.2}deg", (0.5 - y) * 12.0));
            let _ = style.set_property("--mx", &format!("{:.1}%", x * 100.0));
            let _ = style.set_property("--my", &format!("{:.1}%", y * 100.0));
            let _ = target.class_list().add_1("is-tilting");
        });
        let target = el.clone();
        on(&el, "pointerleave", move |_e: web_sys::PointerEvent| {
            let style = target.style();
            let _ = style.set_property("--ry", "0deg");
            let _ = style.set_property("--rx", "0deg");
            let _ = target.class_list().remove_1("is-tilting");
        });
    }
}
