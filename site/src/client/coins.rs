//! The hero scene, in 2D: coins, notes and receipts rain down, bounce, knock into each
//! other and pile up at the bottom of the canvas. A tap throws them all up; the pointer
//! pushes the ones it passes over. A small physics engine of discs: gravity, walls,
//! collisions with restitution and friction, a spin that follows the rolling.

use std::cell::Cell;
use std::f64::consts::TAU;
use std::rc::Rc;

use super::canvas::{Surface, run};
use super::{Rng, on, reduced_motion};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Gold,
    Silver,
    Copper,
    GreenNote,
    VioletNote,
    Receipt,
}

const KINDS: [Kind; 7] = [
    Kind::Gold,
    Kind::Silver,
    Kind::GreenNote,
    Kind::Copper,
    Kind::Gold,
    Kind::VioletNote,
    Kind::Receipt,
];

/// A piece: a disc for the physics, drawn as a coin, a note or a receipt.
struct Piece {
    kind: Kind,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    angle: f64,
    spin: f64,
    r: f64,
    mass: f64,
}

const GRAVITY: f64 = 1500.0;
const RESTITUTION: f64 = 0.35;
const FRICTION: f64 = 0.985;

impl Piece {
    fn new(kind: Kind, unit: f64, rng: &mut Rng) -> Piece {
        let r = match kind {
            Kind::Gold | Kind::Silver | Kind::Copper => unit * f64::from(rng.range(0.85, 1.15)),
            Kind::GreenNote | Kind::VioletNote => unit * 1.25,
            Kind::Receipt => unit * 1.1,
        };
        Piece {
            kind,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            angle: f64::from(rng.range(0.0, 1.0)) * TAU,
            spin: 0.0,
            r,
            mass: r * r,
        }
    }

    /// Back above the canvas, somewhere across its width.
    fn drop_from_above(&mut self, width: f64, rng: &mut Rng, height_above: f64) {
        self.x = f64::from(rng.range(0.1, 0.9)) * width;
        self.y = -self.r - height_above;
        self.vx = f64::from(rng.range(-60.0, 60.0));
        self.vy = f64::from(rng.range(0.0, 120.0));
        self.spin = f64::from(rng.range(-4.0, 4.0));
    }
}

/// One step of the physics.
fn step(pieces: &mut [Piece], dt: f64, width: f64, floor: f64, pointer: Option<(f64, f64)>) {
    for p in pieces.iter_mut() {
        p.vy += GRAVITY * dt;
        if let Some((px, py)) = pointer {
            // The pointer pushes what it passes over, like a hand in a pile of coins.
            let (dx, dy) = (p.x - px, p.y - py);
            let d2 = dx * dx + dy * dy;
            let reach = p.r * 4.0;
            if d2 < reach * reach && d2 > 1.0 {
                let d = d2.sqrt();
                let push = (1.0 - d / reach) * 4200.0 * dt;
                p.vx += dx / d * push;
                p.vy += dy / d * push;
            }
        }
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.angle += p.spin * dt;

        // The walls and the floor.
        if p.x < p.r {
            p.x = p.r;
            p.vx = p.vx.abs() * RESTITUTION;
        } else if p.x > width - p.r {
            p.x = width - p.r;
            p.vx = -p.vx.abs() * RESTITUTION;
        }
        if p.y > floor - p.r {
            p.y = floor - p.r;
            if p.vy > 0.0 {
                p.vy = -p.vy * RESTITUTION;
                if p.vy.abs() < 40.0 {
                    p.vy = 0.0;
                }
            }
            // Rolling on the floor: it slows down, and turns as it rolls.
            p.vx *= FRICTION;
            p.spin = p.vx / p.r;
        }
    }

    // Collisions between pieces: they are pushed apart, and bounce.
    let n = pieces.len();
    for i in 0..n {
        for j in i + 1..n {
            let (a, b) = pieces.split_at_mut(j);
            let (p, q) = (&mut a[i], &mut b[0]);
            let (dx, dy) = (q.x - p.x, q.y - p.y);
            let min = p.r + q.r;
            let d2 = dx * dx + dy * dy;
            if d2 >= min * min || d2 < 1e-6 {
                continue;
            }
            let d = d2.sqrt();
            let (nx, ny) = (dx / d, dy / d);
            let overlap = min - d;
            let total = p.mass + q.mass;
            p.x -= nx * overlap * q.mass / total;
            p.y -= ny * overlap * q.mass / total;
            q.x += nx * overlap * p.mass / total;
            q.y += ny * overlap * p.mass / total;
            let along = (q.vx - p.vx) * nx + (q.vy - p.vy) * ny;
            if along < 0.0 {
                let impulse = -(1.0 + RESTITUTION) * along / (1.0 / p.mass + 1.0 / q.mass);
                p.vx -= impulse * nx / p.mass;
                p.vy -= impulse * ny / p.mass;
                q.vx += impulse * nx / q.mass;
                q.vy += impulse * ny / q.mass;
                // The rub turns them a little.
                let tangent = (q.vx - p.vx) * -ny + (q.vy - p.vy) * nx;
                p.spin += tangent / p.r * 0.05;
                q.spin -= tangent / q.r * 0.05;
            }
        }
    }
}

fn draw_coin(s: &Surface, p: &Piece) {
    let (stops, ink): (&[(f32, &str)], &str) = match p.kind {
        Kind::Gold => (
            &[
                (0.0, "#fff6cf"),
                (0.35, "#ffd56b"),
                (0.85, "#e6a521"),
                (1.0, "#b9800f"),
            ],
            "#8a5a00",
        ),
        Kind::Silver => (
            &[
                (0.0, "#ffffff"),
                (0.4, "#e3e6f0"),
                (0.85, "#a9aec2"),
                (1.0, "#7d8299"),
            ],
            "#555a70",
        ),
        _ => (
            &[
                (0.0, "#ffd9c7"),
                (0.4, "#ff9d7a"),
                (0.85, "#d8613e"),
                (1.0, "#a8452a"),
            ],
            "#7a2a12",
        ),
    };
    s.shaded_disc(p.x, p.y, p.r, stops);
    let ctx = &s.ctx;
    ctx.begin_path();
    let _ = ctx.arc(p.x, p.y, p.r * 0.78, 0.0, TAU);
    ctx.set_stroke_style_str("rgba(0,0,0,0.18)");
    ctx.set_line_width((p.r * 0.07).max(1.0));
    ctx.stroke();
    ctx.save();
    let _ = ctx.translate(p.x, p.y);
    let _ = ctx.rotate(p.angle);
    s.label(
        "€",
        0.0,
        p.r * 0.04,
        &format!("900 {:.0}px system-ui, sans-serif", p.r * 1.05),
        ink,
    );
    ctx.restore();
}

fn draw_note(s: &Surface, p: &Piece) {
    let (fill, seal) = if p.kind == Kind::GreenNote {
        ("#21c7a8", "#0f8f78")
    } else {
        ("#7c5cff", "#5536e0")
    };
    let ctx = &s.ctx;
    ctx.save();
    let _ = ctx.translate(p.x, p.y);
    let _ = ctx.rotate(p.angle);
    let (w, h) = (p.r * 2.2, p.r * 1.2);
    s.rounded(w, h, p.r * 0.14);
    ctx.set_fill_style_str(fill);
    ctx.fill();
    s.rounded(w * 0.86, h * 0.72, p.r * 0.1);
    ctx.set_stroke_style_str("rgba(255,255,255,0.45)");
    ctx.set_line_width(1.2);
    ctx.stroke();
    s.disc(0.0, 0.0, h * 0.26, seal);
    s.label(
        "€",
        0.0,
        h * 0.02,
        &format!("800 {:.0}px system-ui, sans-serif", h * 0.36),
        "#fff",
    );
    ctx.restore();
}

fn draw_receipt(s: &Surface, p: &Piece) {
    let ctx = &s.ctx;
    ctx.save();
    let _ = ctx.translate(p.x, p.y);
    let _ = ctx.rotate(p.angle);
    let (w, h) = (p.r * 1.3, p.r * 1.9);
    // Paper with a torn (zigzag) bottom edge.
    ctx.begin_path();
    ctx.move_to(-w / 2.0, -h / 2.0);
    ctx.line_to(w / 2.0, -h / 2.0);
    ctx.line_to(w / 2.0, h / 2.0);
    let teeth = 6;
    for k in 0..teeth {
        let x0 = w / 2.0 - w * f64::from(k) / f64::from(teeth);
        ctx.line_to(x0 - w / f64::from(teeth) / 2.0, h / 2.0 - p.r * 0.12);
        ctx.line_to(x0 - w / f64::from(teeth), h / 2.0);
    }
    ctx.close_path();
    ctx.set_fill_style_str("#f4f1ea");
    ctx.fill();
    ctx.set_fill_style_str("rgba(20,16,40,0.35)");
    for k in 0..4 {
        let y = -h / 2.0 + h * (0.2 + 0.15 * f64::from(k));
        let length = if k == 3 { 0.4 } else { 0.7 };
        ctx.fill_rect(-w * 0.35, y, w * length, 1.6);
    }
    ctx.restore();
}

/// Starts the scene in `canvas`; the returned function stops it.
pub fn start(canvas: &web_sys::Element) -> Box<dyn FnOnce()> {
    let Some(mut surface) = Surface::new(canvas) else {
        return Box::new(|| {});
    };
    let reduced = reduced_motion();
    let mut rng = Rng::new();
    let unit = (surface.width / 26.0).clamp(13.0, 24.0);
    let count = if surface.width < 640.0 { 22 } else { 34 };
    let mut pieces: Vec<Piece> = (0..count)
        .map(|i| Piece::new(KINDS[i % KINDS.len()], unit, &mut rng))
        .collect();
    // The first pieces are already falling across the canvas when it appears.
    for p in pieces.iter_mut() {
        p.drop_from_above(surface.width, &mut rng, 0.0);
        p.y = f64::from(rng.range(-0.4, 0.55)) * surface.height;
    }
    if reduced {
        // No rain: the pile, already settled.
        let floor = surface.height - 8.0;
        for _ in 0..900 {
            step(&mut pieces, 1.0 / 120.0, surface.width, floor, None);
        }
    }

    // A tap throws everything up; the mouse pushes what it passes over.
    let throw = Rc::new(Cell::new(false));
    let pointer = Rc::new(Cell::new(None::<(f64, f64)>));
    {
        let throw = throw.clone();
        on(canvas, "pointerdown", move |_e: web_sys::PointerEvent| {
            throw.set(true)
        });
        let target = canvas.clone();
        let hover = pointer.clone();
        on(canvas, "pointermove", move |e: web_sys::PointerEvent| {
            if e.pointer_type() == "mouse" {
                let rect = target.get_bounding_client_rect();
                hover.set(Some((
                    f64::from(e.client_x()) - rect.left(),
                    f64::from(e.client_y()) - rect.top(),
                )));
            }
        });
        let gone = pointer.clone();
        on(canvas, "pointerleave", move |_e: web_sys::PointerEvent| {
            gone.set(None)
        });
    }

    let mut since_drop = 0.0;
    let mut next = 0usize;
    run(canvas, move |dt, t| {
        surface.fit();
        let (width, height) = (surface.width, surface.height);
        let floor = height - 8.0;
        if throw.replace(false) {
            for p in &mut pieces {
                p.vy = -f64::from(rng.range(650.0, 1050.0));
                p.vx = f64::from(rng.range(-260.0, 260.0));
                p.spin = f64::from(rng.range(-12.0, 12.0));
            }
        }
        if !reduced {
            // The rain: one piece goes back up regularly, so the pile keeps moving.
            since_drop += dt;
            if since_drop > 0.7 {
                since_drop = 0.0;
                let i = next % pieces.len();
                next += 1;
                pieces[i].drop_from_above(width, &mut rng, 0.0);
            }
            let pointer = pointer.get();
            for _ in 0..2 {
                step(&mut pieces, dt / 2.0, width, floor, pointer);
            }
        }

        surface.clear();
        let ctx = &surface.ctx;
        // A glow on the floor, where the pile lies.
        if let Ok(glow) = ctx.create_radial_gradient(
            width / 2.0,
            floor,
            0.0,
            width / 2.0,
            floor,
            (width * 0.6).min(height * 0.55),
        ) {
            let _ = glow.add_color_stop(0.0, "rgba(255,193,69,0.22)");
            let _ = glow.add_color_stop(1.0, "rgba(255,193,69,0)");
            ctx.set_fill_style_canvas_gradient(&glow);
            ctx.fill_rect(0.0, 0.0, width, height);
        }
        let shimmer = 0.5 + 0.5 * (t * 1.4).sin();
        ctx.set_fill_style_str(&format!("rgba(33,199,168,{:.3})", 0.35 + 0.35 * shimmer));
        ctx.fill_rect(width * 0.08, floor, width * 0.84, 2.0);
        // Soft shadows under the pieces close to the floor, then the pieces.
        for p in &pieces {
            let near = (1.0 - ((floor - p.y - p.r) / (height * 0.6)).clamp(0.0, 1.0)) * 0.35;
            if near > 0.02 {
                ctx.begin_path();
                let _ = ctx.ellipse(p.x, floor + 1.0, p.r * 1.1, p.r * 0.22, 0.0, 0.0, TAU);
                ctx.set_fill_style_str(&format!("rgba(0,0,0,{near:.3})"));
                ctx.fill();
            }
        }
        for p in &pieces {
            match p.kind {
                Kind::Gold | Kind::Silver | Kind::Copper => draw_coin(&surface, p),
                Kind::GreenNote | Kind::VioletNote => draw_note(&surface, p),
                Kind::Receipt => draw_receipt(&surface, p),
            }
        }
    })
}
