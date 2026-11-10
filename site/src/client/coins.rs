//! The hero scene: coins, notes and receipts that rain down and pile up, with real
//! physics (mass, gravity, bounces, friction). A tap throws everything back up.

use std::cell::Cell;
use std::f32::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

use paquets::physics::BodyKind;
use paquets::scene::*;

use super::{Rng, on, reduced_motion, visibility};

/// What falls: a geometry, a color, how metallic and how rough it is.
struct Kind {
    geometry: fn() -> Geometry,
    color: u32,
    metalness: f32,
    roughness: f32,
}

const KINDS: [Kind; 6] = [
    // Gold, silver and copper coins.
    Kind {
        geometry: coin,
        color: 0xffc145,
        metalness: 0.55,
        roughness: 0.3,
    },
    Kind {
        geometry: coin,
        color: 0xdfe3ef,
        metalness: 0.5,
        roughness: 0.25,
    },
    Kind {
        geometry: coin,
        color: 0xff7a59,
        metalness: 0.45,
        roughness: 0.32,
    },
    // Notes.
    Kind {
        geometry: note,
        color: 0x21c7a8,
        metalness: 0.0,
        roughness: 0.65,
    },
    Kind {
        geometry: note,
        color: 0x7c5cff,
        metalness: 0.0,
        roughness: 0.6,
    },
    // A receipt.
    Kind {
        geometry: receipt,
        color: 0xf4f1ea,
        metalness: 0.0,
        roughness: 0.85,
    },
];

fn coin() -> Geometry {
    CylinderGeometry::new(0.36, 0.36, 0.08, 40)
}

fn note() -> Geometry {
    BoxGeometry::new(0.95, 0.025, 0.5)
}

fn receipt() -> Geometry {
    BoxGeometry::new(0.48, 0.02, 0.85)
}

/// `hex` darkened to `f` of its brightness.
fn dim(hex: u32, f: f32) -> u32 {
    let c = |shift: u32| (((hex >> shift) & 0xff) as f32 * f) as u32;
    (c(16) << 16) | (c(8) << 8) | c(0)
}

/// Where the pile lies, between invisible walls.
const HALF_WIDTH: f32 = 2.6;
const HALF_DEPTH: f32 = 1.4;

/// Puts `object` above the pile, turned at random, and lets it fall.
fn drop_from_above(scene: &mut Scene, physics: &mut Physics, rng: &mut Rng, id: ObjectId, y: f32) {
    let object = &mut scene[id];
    object.position.set(
        rng.range(-HALF_WIDTH + 0.5, HALF_WIDTH - 0.5),
        y,
        rng.range(-HALF_DEPTH + 0.4, HALF_DEPTH - 0.4),
    );
    object.rotation.x = rng.range(0.0, TAU);
    object.rotation.y = rng.range(0.0, TAU);
    object.rotation.z = rng.range(0.0, TAU);
    if let Some(body) = physics.body_mut(id) {
        body.linear_velocity = Vector3::new(0.0, rng.range(-1.5, 0.0), 0.0);
        body.angular_velocity = Vector3::new(
            rng.range(-6.0, 6.0),
            rng.range(-6.0, 6.0),
            rng.range(-6.0, 6.0),
        );
    }
}

/// Starts the scene in `canvas`; the returned function stops it.
pub fn start(canvas: &web_sys::Element) -> impl FnOnce() + use<> {
    let reduced = reduced_motion();
    let narrow = canvas.client_width() < 640;
    let mut rng = Rng::new();
    let mut scene = Scene::new();

    // A round stage with a rim of light, on an invisible ground, between four walls
    // nobody sees.
    let floor = scene.add(Mesh::new(
        PlaneGeometry::new(40.0, 40.0),
        MeshBasicMaterial::new(),
    ));
    scene[floor].rotation.x = -FRAC_PI_2;
    scene[floor].visible = false;
    let stage = scene.add(Mesh::new(
        CylinderGeometry::new(3.1, 3.4, 0.4, 96),
        MeshStandardMaterial::new()
            .color(0x1d1736)
            .roughness(0.45)
            .metalness(0.2),
    ));
    scene[stage].position.set(0.0, -0.2, 0.0);
    scene[stage].receive_shadow = true;
    let rim = scene.add(Mesh::new(
        TorusGeometry::new(3.12, 0.035, 12, 160),
        MeshStandardMaterial::new()
            .color(0x21c7a8)
            .emissive(0x21c7a8)
            .roughness(0.3),
    ));
    scene[rim].rotation.x = FRAC_PI_2;
    scene[rim].position.set(0.0, 0.0, 0.0);
    let mut walls = Vec::new();
    for (w, d, x, z) in [
        (0.2, HALF_DEPTH * 2.0 + 0.4, -HALF_WIDTH - 0.1, 0.0),
        (0.2, HALF_DEPTH * 2.0 + 0.4, HALF_WIDTH + 0.1, 0.0),
        (HALF_WIDTH * 2.0 + 0.4, 0.2, 0.0, -HALF_DEPTH - 0.1),
        (HALF_WIDTH * 2.0 + 0.4, 0.2, 0.0, HALF_DEPTH + 0.1),
    ] {
        let wall = scene.add(Mesh::new(
            BoxGeometry::new(w, 6.0, d),
            MeshBasicMaterial::new(),
        ));
        scene[wall].position.set(x, 3.0, z);
        scene[wall].visible = false;
        walls.push(wall);
    }

    // A ring of light floating behind the pile, which turns.
    let ring = scene.add(Mesh::new(
        TorusGeometry::new(2.3, 0.05, 24, 160),
        MeshStandardMaterial::new()
            .color(0x7c5cff)
            .emissive(0x5b3dff)
            .roughness(0.3),
    ));
    scene[ring].position.set(0.0, 2.6, -3.6);
    let halo = scene.add(Mesh::new(
        TorusGeometry::new(1.6, 0.025, 16, 120),
        MeshStandardMaterial::new()
            .color(0x21c7a8)
            .emissive(0x0f8f78)
            .roughness(0.3),
    ));
    scene[halo].position.set(0.0, 2.6, -3.6);

    // The coins, notes and receipts.
    let count = if narrow { 20 } else { 30 };
    let mut pieces = Vec::with_capacity(count);
    for i in 0..count {
        let kind = &KINDS[if i % 7 == 6 { 5 } else { i % 5 }];
        let id = scene.add(Mesh::new(
            (kind.geometry)(),
            MeshStandardMaterial::new()
                .color(kind.color)
                .emissive(dim(kind.color, 0.12))
                .metalness(kind.metalness)
                .roughness(kind.roughness),
        ));
        scene[id].cast_shadow = true;
        scene[id].receive_shadow = true;
        pieces.push(id);
    }

    // Light: a warm sun with shadows, a violet and a green light on the sides.
    scene.add(HemisphereLight::new(0xb9a8ff, 0x140c26, 0.9));
    let sun = scene.add(DirectionalLight::new(0xfff1d6, 3.2));
    scene[sun].position.set(3.0, 9.0, 5.0);
    scene[sun].cast_shadow = true;
    scene[sun].shadow.map_size = if narrow { 1024 } else { 2048 };
    scene[sun].shadow.radius = 3.0;
    scene[sun].shadow.extent = Some(6.0);
    let violet = scene.add(PointLight::new(0x7c5cff, 40.0));
    scene[violet].position.set(-3.5, 2.5, 2.5);
    let green = scene.add(PointLight::new(0x21c7a8, 30.0));
    scene[green].position.set(3.5, 1.5, 2.5);

    // Every mesh gets its body at the first step: then the walls, the ring and the halo
    // are made to stay where they are.
    let mut physics = Physics::new();
    physics.step(&mut scene, 1e-4);
    for &id in walls.iter().chain([&stage, &rim, &ring, &halo]) {
        if let Some(body) = physics.body_mut(id) {
            body.kind = if id == ring || id == halo || id == rim {
                BodyKind::Kinematic
            } else {
                BodyKind::Static
            };
        }
    }
    for (i, &id) in pieces.iter().enumerate() {
        let y = 2.0 + i as f32 * 0.45;
        drop_from_above(&mut scene, &mut physics, &mut rng, id, y);
        if let Some(body) = physics.body_mut(id) {
            body.restitution = 0.25;
            body.friction = 0.6;
            body.set_density(if i % 5 < 3 { 8.0 } else { 0.8 });
        }
    }
    if reduced {
        // No rain: the pile, already settled.
        for _ in 0..360 {
            physics.step(&mut scene, 1.0 / 60.0);
        }
    }

    let mut camera = PerspectiveCamera::new(if narrow { 52.0 } else { 42.0 }, 0.0, 0.1, 100.0);
    // Further away on a phone, so that the whole stage fits; aside on a wide screen.
    let (eye, target) = if narrow {
        (Vector3::new(0.0, 4.3, 10.4), Vector3::new(0.0, 1.0, 0.0))
    } else {
        (Vector3::new(-1.4, 4.4, 11.8), Vector3::new(-1.4, 1.5, 0.0))
    };
    camera.position = eye;
    camera.look_at(target);

    // A tap throws everything up; the pointer moves the camera a little.
    let throw = Rc::new(Cell::new(false));
    let pointer = Rc::new(Cell::new((0.0f32, 0.0f32)));
    {
        let throw = throw.clone();
        on(canvas, "pointerdown", move |_e: web_sys::PointerEvent| {
            throw.set(true);
        });
        let pointer = pointer.clone();
        on(
            &super::window(),
            "pointermove",
            move |e: web_sys::PointerEvent| {
                let w = super::window();
                let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1.0);
                let height = w
                    .inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                pointer.set((
                    (e.client_x() as f64 / width * 2.0 - 1.0) as f32,
                    (e.client_y() as f64 / height * 2.0 - 1.0) as f32,
                ));
            },
        );
    }

    let visible = visibility(canvas);
    let looping = Renderer::new(canvas).ok().map(|mut renderer| {
        renderer.set_pixel_ratio(device_pixel_ratio().min(2.0));
        renderer.tone_mapping = ToneMapping::AcesFilmic;
        renderer.tone_mapping_exposure = 1.1;
        renderer.shadows = true;
        let mut last: Option<f64> = None;
        let mut next_drop = 0usize;
        let mut since_drop = 0.0f32;
        let mut sway = (0.0f32, 0.0f32);
        animation_loop(move |time| {
            if !visible.get() {
                last = None;
                return;
            }
            let dt = last
                .map_or(1.0 / 60.0, |l| ((time - l) / 1000.0) as f32)
                .clamp(0.0, 1.0 / 30.0);
            last = Some(time);
            let t = (time / 1000.0) as f32;

            if throw.replace(false) {
                for &id in &pieces {
                    if let Some(body) = physics.body_mut(id) {
                        body.linear_velocity = Vector3::new(
                            rng.range(-1.5, 1.5),
                            rng.range(5.0, 8.0),
                            rng.range(-0.8, 0.8),
                        );
                        body.angular_velocity = Vector3::new(
                            rng.range(-10.0, 10.0),
                            rng.range(-10.0, 10.0),
                            rng.range(-10.0, 10.0),
                        );
                        body.wake_up();
                    }
                }
            }
            if !reduced {
                // The rain: one piece goes back up regularly.
                since_drop += dt;
                if since_drop > 0.55 {
                    since_drop = 0.0;
                    let id = pieces[next_drop % pieces.len()];
                    next_drop += 1;
                    drop_from_above(&mut scene, &mut physics, &mut rng, id, 7.5);
                }
                scene[ring].rotation.y = t * 0.35;
                scene[ring].rotation.x = 0.3 + (t * 0.5).sin() * 0.15;
                scene[halo].rotation.x = -t * 0.45;
                scene[halo].rotation.y = 0.4 + (t * 0.3).cos() * 0.2;
                physics.step(&mut scene, dt);
            }
            // Anything that left the pile comes back from above.
            for &id in &pieces {
                if scene[id].position.y < -2.0 {
                    drop_from_above(&mut scene, &mut physics, &mut rng, id, 7.0);
                }
            }

            let (px, py) = pointer.get();
            sway.0 += (px - sway.0) * 0.04;
            sway.1 += (py - sway.1) * 0.04;
            let drift = if reduced { 0.0 } else { (t * 0.2).sin() * 0.4 };
            camera
                .position
                .set(eye.x + drift + sway.0 * 0.9, eye.y - sway.1 * 0.5, eye.z);
            camera.look_at(target);
            renderer.render(&scene, &camera);
        })
    });
    move || drop(looping)
}
