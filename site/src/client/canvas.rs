//! What the 2D scenes share: a canvas sized to its box (times the pixel ratio, at most 2),
//! an animation loop that pauses while the canvas is scrolled away and stops when the
//! island goes, and a few drawing helpers.

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::{request_frame, visibility, window};

/// A canvas and its 2D context, in CSS pixels.
pub struct Surface {
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,
    /// The size in CSS pixels, after the last [`Surface::fit`].
    pub width: f64,
    pub height: f64,
}

impl Surface {
    pub fn new(element: &web_sys::Element) -> Option<Surface> {
        let canvas: HtmlCanvasElement = element.clone().dyn_into().ok()?;
        let ctx: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
        let mut surface = Surface {
            canvas,
            ctx,
            width: 0.0,
            height: 0.0,
        };
        surface.fit();
        Some(surface)
    }

    /// Follows the size the page gives the canvas; `true` when it changed.
    pub fn fit(&mut self) -> bool {
        let width = f64::from(self.canvas.client_width());
        let height = f64::from(self.canvas.client_height());
        if width == self.width && height == self.height {
            return false;
        }
        let ratio = window().device_pixel_ratio().clamp(1.0, 2.0);
        self.canvas.set_width((width * ratio).round() as u32);
        self.canvas.set_height((height * ratio).round() as u32);
        let _ = self.ctx.set_transform(ratio, 0.0, 0.0, ratio, 0.0, 0.0);
        self.width = width;
        self.height = height;
        true
    }

    pub fn clear(&self) {
        self.ctx.clear_rect(0.0, 0.0, self.width, self.height);
    }

    /// A filled circle.
    pub fn disc(&self, x: f64, y: f64, r: f64, color: &str) {
        self.ctx.begin_path();
        let _ = self.ctx.arc(x, y, r.max(0.0), 0.0, std::f64::consts::TAU);
        self.ctx.set_fill_style_str(color);
        self.ctx.fill();
    }

    /// A circle filled with a radial gradient, lit from the top left.
    pub fn shaded_disc(&self, x: f64, y: f64, r: f64, stops: &[(f32, &str)]) {
        let ctx = &self.ctx;
        let Ok(gradient) =
            ctx.create_radial_gradient(x - r * 0.35, y - r * 0.4, r * 0.1, x, y, r.max(0.1))
        else {
            return;
        };
        for (offset, color) in stops {
            let _ = gradient.add_color_stop(*offset, color);
        }
        ctx.begin_path();
        let _ = ctx.arc(x, y, r.max(0.0), 0.0, std::f64::consts::TAU);
        ctx.set_fill_style_canvas_gradient(&gradient);
        ctx.fill();
    }

    /// A rounded rectangle centered on the origin of the current transform.
    pub fn rounded(&self, w: f64, h: f64, radius: f64) {
        self.ctx.begin_path();
        let _ = self
            .ctx
            .round_rect_with_f64(-w / 2.0, -h / 2.0, w, h, radius);
    }

    /// Centered text.
    pub fn label(&self, text: &str, x: f64, y: f64, font: &str, color: &str) {
        self.ctx.set_font(font);
        self.ctx.set_text_align("center");
        self.ctx.set_text_baseline("middle");
        self.ctx.set_fill_style_str(color);
        let _ = self.ctx.fill_text(text, x, y);
    }
}

/// Calls `frame(dt, t)` (seconds) on every animation frame while the canvas is on screen,
/// until the returned function is called.
pub fn run(
    element: &web_sys::Element,
    mut frame: impl FnMut(f64, f64) + 'static,
) -> Box<dyn FnOnce()> {
    let stopped = Rc::new(Cell::new(false));
    let visible = visibility(element);
    let flag = stopped.clone();
    let mut last: Option<f64> = None;
    request_frame(move |now| {
        if flag.get() {
            return false;
        }
        if !visible.get() {
            last = None;
            return true;
        }
        let dt = last.map_or(1.0 / 60.0, |l| ((now - l) / 1000.0).clamp(0.0, 1.0 / 30.0));
        last = Some(now);
        frame(dt, now / 1000.0);
        true
    });
    Box::new(move || stopped.set(true))
}
