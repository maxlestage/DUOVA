//! "How it works", in 3D: Alice, Bob and Chloé are three planets, and the coins of the
//! two transfers fly along arcs from those who owe to the one who paid.

use std::cell::Cell;
use std::f32::consts::TAU;
use std::rc::Rc;

use paquets::scene::*;

use super::{on, reduced_motion, visibility};

/// A member: where it floats, its color, its size (its balance).
struct Planet {
    at: Vector3,
    color: u32,
    radius: f32,
}

const ALICE: usize = 0;
const BOB: usize = 1;
const CHLOE: usize = 2;

const PLANETS: [Planet; 3] = [
    Planet {
        at: Vector3::new(0.0, 1.15, 0.0),
        color: 0xff7a59,
        radius: 0.62,
    },
    Planet {
        at: Vector3::new(-2.15, -0.85, 0.4),
        color: 0x7c5cff,
        radius: 0.42,
    },
    Planet {
        at: Vector3::new(2.15, -0.85, 0.4),
        color: 0x21c7a8,
        radius: 0.5,
    },
];

/// The two transfers: (from, to, coins in flight).
const ROUTES: [(usize, usize, usize); 2] = [(CHLOE, ALICE, 6), (BOB, ALICE, 2)];

/// `hex` darkened to `f` of its brightness.
fn dim(hex: u32, f: f32) -> u32 {
    let c = |shift: u32| (((hex >> shift) & 0xff) as f32 * f) as u32;
    (c(16) << 16) | (c(8) << 8) | c(0)
}

/// A point of the arc from `a` to `b` at `t` in `[0, 1]`, raised towards the viewer.
fn arc(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    let mid = Vector3::new(
        (a.x + b.x) / 2.0,
        (a.y + b.y) / 2.0 + 0.9,
        (a.z + b.z) / 2.0 + 1.3,
    );
    let u = 1.0 - t;
    Vector3::new(
        u * u * a.x + 2.0 * u * t * mid.x + t * t * b.x,
        u * u * a.y + 2.0 * u * t * mid.y + t * t * b.y,
        u * u * a.z + 2.0 * u * t * mid.z + t * t * b.z,
    )
}

pub fn start(canvas: &web_sys::Element) -> impl FnOnce() + use<> {
    let reduced = reduced_motion();
    let mut scene = Scene::new();
    let world = scene.add(Group::new());

    let mut planets = Vec::new();
    let mut rings = Vec::new();
    for planet in &PLANETS {
        let id = scene.add_to(
            world,
            Mesh::new(
                SphereGeometry::new(planet.radius, 48, 32),
                MeshStandardMaterial::new()
                    .color(planet.color)
                    .roughness(0.25)
                    .metalness(0.1)
                    .emissive(dim(planet.color, 0.18)),
            ),
        );
        scene[id].position = planet.at;
        planets.push(id);
        let ring = scene.add_to(
            world,
            Mesh::new(
                TorusGeometry::new(planet.radius * 1.55, 0.012, 12, 96),
                MeshBasicMaterial::new()
                    .color(planet.color)
                    .opacity(0.7)
                    .transparent(true),
            ),
        );
        scene[ring].position = planet.at;
        scene[ring].rotation.x = 1.2;
        rings.push(ring);
    }

    // The dotted paths of the transfers.
    for &(from, to, _) in &ROUTES {
        for k in 1..24 {
            let dot = scene.add_to(
                world,
                Mesh::new(
                    SphereGeometry::new(0.022, 8, 6),
                    MeshBasicMaterial::new()
                        .color(0xffffff)
                        .opacity(0.35)
                        .transparent(true),
                ),
            );
            scene[dot].position = arc(PLANETS[from].at, PLANETS[to].at, k as f32 / 24.0);
        }
    }

    // The coins in flight.
    let mut coins = Vec::new();
    for (route, &(_, _, n)) in ROUTES.iter().enumerate() {
        for k in 0..n {
            let id = scene.add_to(
                world,
                Mesh::new(
                    CylinderGeometry::new(0.17, 0.17, 0.04, 32),
                    MeshStandardMaterial::new()
                        .color(0xffc145)
                        .metalness(1.0)
                        .roughness(0.25)
                        .emissive(0x3a2600),
                ),
            );
            coins.push((id, route, k as f32 / n as f32));
        }
    }

    scene.add(HemisphereLight::new(0xc8bfff, 0x1a1030, 1.1));
    let sun = scene.add(DirectionalLight::new(0xffffff, 2.6));
    scene[sun].position.set(2.0, 4.0, 6.0);
    let rim = scene.add(PointLight::new(0x7c5cff, 25.0));
    scene[rim].position.set(-3.0, 2.0, -2.0);

    let narrow = canvas.client_width() < 520;
    let mut camera = PerspectiveCamera::new(if narrow { 58.0 } else { 42.0 }, 0.0, 0.1, 100.0);
    camera
        .position
        .set(0.0, 0.3, if narrow { 6.9 } else { 7.4 });
    camera.look_at(Vector3::ZERO);

    let pointer = Rc::new(Cell::new(0.0f32));
    {
        let pointer = pointer.clone();
        on(
            &super::window(),
            "pointermove",
            move |e: web_sys::PointerEvent| {
                let width = super::window()
                    .inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                pointer.set((e.client_x() as f64 / width * 2.0 - 1.0) as f32);
            },
        );
    }

    let visible = visibility(canvas);
    let looping = Renderer::new(canvas).ok().map(|mut renderer| {
        renderer.set_pixel_ratio(device_pixel_ratio().min(2.0));
        renderer.tone_mapping = ToneMapping::AcesFilmic;
        let mut sway = 0.0f32;
        animation_loop(move |time| {
            if !visible.get() {
                return;
            }
            let t = if reduced { 1.2 } else { (time / 1000.0) as f32 };
            sway += (pointer.get() - sway) * 0.05;
            scene[world].rotation.y = (t * 0.25).sin() * 0.35 + sway * 0.4;
            scene[world].rotation.x = (t * 0.2).cos() * 0.08;

            // The coins: along their arc, turning; Alice swells a little at each arrival.
            let mut arrival = 0.0f32;
            for &(id, route, phase) in &coins {
                let (from, to, _) = ROUTES[route];
                let speed = 0.32;
                let p = (t * speed + phase).fract();
                let point = arc(PLANETS[from].at, PLANETS[to].at, p);
                let coin = &mut scene[id];
                coin.position = point;
                coin.rotation.x = 1.3 + p * TAU;
                coin.rotation.z = t * 3.0 + phase * TAU;
                let s = (p * 10.0).min(1.0) * ((1.0 - p) * 10.0).min(1.0);
                coin.scale.set(s, s, s);
                if p > 0.9 {
                    arrival = arrival.max((p - 0.9) * 10.0);
                }
            }
            let pulse = 1.0 + (arrival * std::f32::consts::PI).sin() * 0.08;
            scene[planets[ALICE]].scale.set(pulse, pulse, pulse);
            for (i, &ring) in rings.iter().enumerate() {
                scene[ring].rotation.z = t * (0.4 + i as f32 * 0.15);
            }
            for (i, &planet) in planets.iter().enumerate() {
                scene[planet].position.y =
                    PLANETS[i].at.y + (t * 1.1 + i as f32 * 2.0).sin() * 0.08;
                scene[rings[i]].position.y = scene[planet].position.y;
            }
            renderer.render(&scene, &camera);
        })
    });
    move || drop(looping)
}
