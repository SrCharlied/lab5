//! Regression guards for the new bonus routes; feature RED/GREEN receipts live separately.
use lab5::{
    assets::primitives::uv_sphere,
    extras::{self, Body},
    math::{Transform, Vec2, Vec3},
    procedural::direction,
    renderer::{framebuffer::Framebuffer, pipeline::draw_model, shader::FragmentInput},
    scene::camera::Camera,
};
const BODIES: [Body; 3] = [Body::Volcanic, Body::Desert, Body::Temperate];
fn sample(u: f64, v: f64) -> FragmentInput {
    let uv = Vec2::new(u, v);
    let p = direction(uv);
    FragmentInput {
        uv,
        normal: p,
        world_position: p,
        screen_position: Vec3::ZERO,
    }
}
#[test]
fn seams_poles_and_local_fields_are_stable() {
    for body in BODIES {
        for layer in 1..=4 {
            for j in 0..=24 {
                let a = extras::shade(body, &sample(0.0, j as f64 / 24.0), layer);
                let b = extras::shade(body, &sample(1.0, j as f64 / 24.0), layer);
                assert!(a.is_finite() && (a - b).length() < 1e-10);
            }
            for v in [0.0, 1.0] {
                for i in 0..32 {
                    assert!(
                        (extras::shade(body, &sample(0.0, v), layer)
                            - extras::shade(body, &sample(i as f64 / 32.0, v), layer))
                        .length()
                            < 1e-10
                    );
                }
            }
            for i in 0..32 {
                let a = sample(i as f64 / 32.0, 0.37);
                let b = FragmentInput {
                    world_position: a.world_position + Vec3::new(17.0, -3.0, 8.0),
                    ..a
                };
                assert_eq!(
                    extras::shade(body, &a, layer),
                    extras::shade(body, &b, layer)
                );
                assert_eq!(extras::moon_shade(&a), extras::moon_shade(&b));
            }
        }
    }
}
#[test]
fn every_layer_changes_surface_pixels_not_geometry() {
    for body in BODIES {
        let (mut previous, reference) = extras::render(192, 128, 0.0, 1, body, false).unwrap();
        for layer in 2..=4 {
            let (next, stats) = extras::render(192, 128, 0.0, layer, body, false).unwrap();
            assert_eq!(stats, reference);
            assert!(
                previous
                    .pixels
                    .iter()
                    .zip(&next.pixels)
                    .filter(|(a, b)| a != b)
                    .count()
                    > 80,
                "{body:?} layer {layer}"
            );
            for (a, b) in previous.pixels.iter().zip(&next.pixels) {
                assert_eq!(*a == lab5::BACKGROUND, *b == lab5::BACKGROUND);
            }
            previous = next;
        }
    }
}
#[test]
fn invalid_library_inputs_are_rejected() {
    for body in BODIES {
        for (w, h) in [(0, 32), (32, 0), (usize::MAX, 2), (4097, 4097)] {
            assert!(extras::render(w, h, 0.0, 4, body, false).is_err());
        }
        for angle in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(extras::render(32, 32, angle, 4, body, false).is_err());
        }
        for layers in [0, 5, 255] {
            assert!(extras::render(32, 32, 0.0, layers, body, false).is_err());
        }
    }
    for body in [Body::Volcanic, Body::Temperate] {
        assert!(extras::render(32, 32, 0.0, 4, body, true).is_err());
    }
}
#[test]
fn repeated_render_is_exact_and_rotation_moves_surface_pattern() {
    for body in BODIES {
        let (a, sa) = extras::render(160, 120, 0.0, 4, body, false).unwrap();
        let (b, sb) = extras::render(160, 120, 0.0, 4, body, false).unwrap();
        assert_eq!(sa, sb);
        assert_eq!(a.pixels, b.pixels);
        let (c, _) = extras::render(160, 120, 0.8, 4, body, false).unwrap();
        assert!(
            a.pixels
                .iter()
                .zip(&c.pixels)
                .filter(|(a, b)| a != b)
                .count()
                > 500
        );
    }
}
#[test]
fn landscape_portrait_and_narrow_frames_do_not_clip_planets_or_moon() {
    for (w, h) in [(320, 240), (240, 320), (96, 384)] {
        for (body, moon) in [
            (Body::Volcanic, false),
            (Body::Temperate, false),
            (Body::Desert, true),
        ] {
            let (fb, _) = extras::render(w, h, 0.0, 4, body, moon).unwrap();
            let points: Vec<_> = fb
                .pixels
                .iter()
                .enumerate()
                .filter(|(_, p)| **p != lab5::BACKGROUND)
                .map(|(i, _)| (i % w, i / w))
                .collect();
            assert!(
                points
                    .iter()
                    .all(|(x, y)| *x > 0 && *y > 0 && *x < w - 1 && *y < h - 1)
            );
            assert!(points.len() > 200);
        }
    }
}
#[test]
fn combined_scene_matches_two_separate_model_draws_and_complete_stats() {
    let mut fb = Framebuffer::new(192, 128, lab5::BACKGROUND).unwrap();
    let mut camera = Camera::new(45_f64.to_radians(), 192.0 / 128.0, 0.1, 100.0).unwrap();
    camera.position = Vec3::new(0.0, 0.0, 4.8);
    camera.target = Vec3::ZERO;
    let planet = Transform {
        rotation: Vec3::new(0.0, 0.4, 0.0),
        ..Transform::IDENTITY
    };
    let moon = Transform {
        position: Vec3::new(1.45, 0.72, 0.35),
        scale: Vec3::splat(0.22),
        rotation: Vec3::new(0.0, 0.4, 0.0),
    };
    let mut stats = draw_model(&mut fb, &uv_sphere(64, 128), &planet, &camera, true, |f| {
        extras::shade(Body::Desert, f, 4)
    });
    stats += draw_model(
        &mut fb,
        &uv_sphere(32, 64),
        &moon,
        &camera,
        true,
        extras::moon_shade,
    );
    let (actual, actual_stats) = extras::render(192, 128, 0.4, 4, Body::Desert, true).unwrap();
    assert_eq!(stats, actual_stats);
    assert_eq!(fb.pixels, actual.pixels);
    // Order independence for the shipped separated arrangement, not an overlap sweep.
    let mut reversed = Framebuffer::new(192, 128, lab5::BACKGROUND).unwrap();
    draw_model(
        &mut reversed,
        &uv_sphere(32, 64),
        &moon,
        &camera,
        true,
        extras::moon_shade,
    );
    draw_model(
        &mut reversed,
        &uv_sphere(64, 128),
        &planet,
        &camera,
        true,
        |f| extras::shade(Body::Desert, f, 4),
    );
    assert_eq!(reversed.pixels, actual.pixels);
}
