//! "How it works", in 2D: Alice, Bob and Chloé are three bubbles sized by their balance,
//! linked by the two transfers. Dashed paths march from those who owe to the one who
//! paid, coins fly along them with a trail, and Alice pulses at each arrival.

use std::f64::consts::TAU;

use super::canvas::{Surface, run};
use super::reduced_motion;

/// A member: where it sits (as a fraction of the canvas), its color, name and balance.
struct Person {
    at: (f64, f64),
    color: &'static str,
    glow: &'static str,
    name: &'static str,
    balance: &'static str,
    size: f64,
}

const ALICE: usize = 0;
const BOB: usize = 1;
const CHLOE: usize = 2;

const PEOPLE: [Person; 3] = [
    Person {
        at: (0.5, 0.26),
        color: "#ff7a59",
        glow: "rgba(255,122,89,",
        name: "Alice",
        balance: "+60 €",
        size: 1.0,
    },
    Person {
        at: (0.18, 0.74),
        color: "#7c5cff",
        glow: "rgba(124,92,255,",
        name: "Bob",
        balance: "−15 €",
        size: 0.72,
    },
    Person {
        at: (0.82, 0.74),
        color: "#21c7a8",
        glow: "rgba(33,199,168,",
        name: "Chloé",
        balance: "−45 €",
        size: 0.84,
    },
];

/// The two transfers: from, to, amount, coins in flight.
const ROUTES: [(usize, usize, &str, usize); 2] =
    [(CHLOE, ALICE, "45 €", 5), (BOB, ALICE, "15 €", 2)];

/// The control point of the curve from `a` to `b`: bent away from the center.
fn control(a: (f64, f64), b: (f64, f64), w: f64, h: f64) -> (f64, f64) {
    let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let center = (w / 2.0, h * 0.58);
    let (dx, dy) = (mid.0 - center.0, mid.1 - center.1);
    let d = (dx * dx + dy * dy).sqrt().max(1.0);
    let bend = w.min(h) * 0.16;
    (mid.0 + dx / d * bend, mid.1 + dy / d * bend)
}

/// The point at `t` of the quadratic curve.
fn on_curve(a: (f64, f64), c: (f64, f64), b: (f64, f64), t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    (
        u * u * a.0 + 2.0 * u * t * c.0 + t * t * b.0,
        u * u * a.1 + 2.0 * u * t * c.1 + t * t * b.1,
    )
}

pub fn start(canvas: &web_sys::Element) -> Box<dyn FnOnce()> {
    let Some(mut surface) = Surface::new(canvas) else {
        return Box::new(|| {});
    };
    let reduced = reduced_motion();
    run(canvas, move |_dt, now| {
        surface.fit();
        let (w, h) = (surface.width, surface.height);
        let t = if reduced { 1.3 } else { now };
        let unit = w.min(h);
        let ctx = &surface.ctx;
        surface.clear();
        let place = |i: usize| {
            let (fx, fy) = PEOPLE[i].at;
            // Each bubble floats a little.
            (
                w * fx,
                h * fy + (t * 1.1 + i as f64 * 2.0).sin() * unit * 0.012,
            )
        };

        // The paths: dashes marching towards the one who is owed.
        let mut arrival: f64 = 0.0;
        for &(from, to, amount, coins) in &ROUTES {
            let (a, b) = (place(from), place(to));
            let c = control(a, b, w, h);
            ctx.begin_path();
            ctx.move_to(a.0, a.1);
            ctx.quadratic_curve_to(c.0, c.1, b.0, b.1);
            let dashes = js_sys::Array::of2(&6.0.into(), &10.0.into());
            let _ = ctx.set_line_dash(&dashes);
            ctx.set_line_dash_offset(-t * 30.0);
            ctx.set_line_width(2.0);
            ctx.set_stroke_style_str("rgba(255,255,255,0.28)");
            ctx.stroke();
            let _ = ctx.set_line_dash(&js_sys::Array::new());

            // The amount, on the curve.
            let (mx, my) = on_curve(a, c, b, 0.5);
            ctx.begin_path();
            let _ = ctx.round_rect_with_f64(mx - 30.0, my - 14.0, 60.0, 28.0, 14.0);
            ctx.set_fill_style_str("rgba(255,193,69,0.16)");
            ctx.fill();
            ctx.set_stroke_style_str("rgba(255,193,69,0.5)");
            ctx.set_line_width(1.0);
            ctx.stroke();
            surface.label(
                amount,
                mx,
                my + 0.5,
                "800 13px system-ui, sans-serif",
                "#ffc145",
            );

            // The coins in flight, each with a fading trail.
            for k in 0..coins {
                let p = (t * 0.3 + k as f64 / coins as f64).fract();
                for trail in 1..6 {
                    let q = p - trail as f64 * 0.018;
                    if q <= 0.0 {
                        continue;
                    }
                    let (x, y) = on_curve(a, c, b, q);
                    let alpha = 0.22 * (1.0 - trail as f64 / 6.0);
                    surface.disc(x, y, unit * 0.016, &format!("rgba(255,193,69,{alpha:.3})"));
                }
                let (x, y) = on_curve(a, c, b, p);
                let grow = (p * 8.0).min(1.0) * ((1.0 - p) * 8.0).min(1.0);
                let r = unit * 0.03 * grow;
                surface.shaded_disc(
                    x,
                    y,
                    r,
                    &[(0.0, "#fff6cf"), (0.45, "#ffc145"), (1.0, "#b8860b")],
                );
                // Squashed horizontally as it turns: a flat coin flipping.
                let flip = (t * 6.0 + k as f64).cos().abs().max(0.25);
                ctx.save();
                let _ = ctx.translate(x, y);
                let _ = ctx.scale(flip, 1.0);
                surface.label(
                    "€",
                    0.0,
                    0.5,
                    &format!("900 {:.0}px system-ui", r * 1.2),
                    "#8a5a00",
                );
                ctx.restore();
                if p > 0.9 {
                    arrival = arrival.max((p - 0.9) * 10.0);
                }
            }
        }

        // The people.
        for (i, person) in PEOPLE.iter().enumerate() {
            let (x, y) = place(i);
            let mut r = unit * 0.11 * person.size;
            if i == ALICE {
                r *= 1.0 + (arrival * std::f64::consts::PI).sin() * 0.08;
            }
            // Rings that spread out and fade.
            for ring in 0..2 {
                let phase = (t * 0.6 + ring as f64 * 0.5 + i as f64 * 0.3).fract();
                ctx.begin_path();
                let _ = ctx.arc(x, y, r * (1.0 + phase * 0.9), 0.0, TAU);
                ctx.set_stroke_style_str(&format!("{}{:.3})", person.glow, 0.45 * (1.0 - phase)));
                ctx.set_line_width(2.0);
                ctx.stroke();
            }
            ctx.set_shadow_color(&format!("{}0.6)", person.glow));
            ctx.set_shadow_blur(r * 0.8);
            surface.shaded_disc(
                x,
                y,
                r,
                &[(0.0, "#ffffff"), (0.25, person.color), (1.0, person.color)],
            );
            ctx.set_shadow_blur(0.0);
            surface.label(
                &person.name[..1],
                x,
                y + 1.0,
                &format!("800 {:.0}px system-ui, sans-serif", r * 0.9),
                "rgba(18,12,31,0.85)",
            );
            surface.label(
                person.name,
                x,
                y + r + 16.0,
                "700 14px system-ui, sans-serif",
                "#f5f2ff",
            );
            let color = if person.balance.starts_with('+') {
                "#3ddc97"
            } else {
                "#ff6b81"
            };
            surface.label(
                person.balance,
                x,
                y + r + 34.0,
                "800 13px system-ui, sans-serif",
                color,
            );
        }
    })
}
