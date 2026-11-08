//! Guards born green; targeted mutation sensitivity is recorded separately.
#[test]
fn pulse_remains_visible_in_encoded_pixels_at_a_fixed_pose() {
    let initial = LiveState::new(0.0).unwrap();
    let mut bright = PulsarRenderer::new(320, 240, 4).unwrap();
    bright.render(&initial).unwrap();
    let time = 1.8;
    let mut minimum = LiveState::new(-0.42 * time).unwrap();
    minimum.advance(time, Controls::default()).unwrap();
    assert!(minimum.angle().abs() < 1e-12);
    let mut dim = PulsarRenderer::new(320, 240, 4).unwrap();
    dim.render(&minimum).unwrap();
    let mut count = 0;
    let mut drop = 0.0;
    for (a, b) in bright
        .framebuffer()
        .pixels
        .iter()
        .zip(&dim.framebuffer().pixels)
    {
        let red = (a >> 16) & 255;
        let green = (a >> 8) & 255;
        if red >= 205 && green >= 215 {
            count += 1;
            drop += red as f64 - ((b >> 16) & 255) as f64;
        }
    }
    assert!(count > 50, "missing bright polar ROI: {count}");
    let mean = drop / count as f64;
    println!("fixed-pose polar ROI={count}, mean red drop={mean}");
    assert!(mean > 20.0, "pulse disappears after tone mapping: {mean}");
}

use lab5::{
    math::{Vec2, Vec3},
    neutron_live::{Controls, LiveState},
    procedural::direction,
    pulsar::{PulsarRenderer, light_visible, shade_surface},
    renderer::shader::FragmentInput,
};
#[test]
fn opaque_star_hides_rear_light_but_not_front_or_lateral_light() {
    let eye = Vec3::Z * 7.4;
    assert!(light_visible(eye, Vec3::Z * 1.01));
    assert!(
        !light_visible(eye, -Vec3::Z * 1.01),
        "rear light must be occluded by the sphere"
    );
    assert!(light_visible(eye, Vec3::X * 2.0));
    assert!(!light_visible(Vec3::ZERO, Vec3::Y));
    assert!(!light_visible(eye, Vec3::new(f64::NAN, 0.0, 0.0)));
    assert!(!light_visible(Vec3::splat(f64::MAX), Vec3::Y));
}
#[test]
fn surface_is_periodic_at_longitude_and_at_geometric_poles() {
    let sample = |u, v| {
        let uv = Vec2::new(u, v);
        let p = direction(uv);
        FragmentInput {
            uv,
            normal: p,
            world_position: p,
            screen_position: Vec3::ZERO,
        }
    };
    for time in [0.0, 1.8, 5.0] {
        for y in 0..=24 {
            let v = y as f64 / 24.0;
            let a = shade_surface(&sample(0.0, v), Vec3::Z * 7.4, 0.2, time);
            let b = shade_surface(&sample(1.0, v), Vec3::Z * 7.4, 0.2, time);
            assert!(a.is_finite() && b.is_finite());
            assert!((a - b).length() < 1e-9);
        }
        for v in [0.0, 1.0] {
            let a = shade_surface(&sample(0.0, v), Vec3::Z * 7.4, 1.0, time);
            for x in 0..=24 {
                let b = shade_surface(&sample(x as f64 / 24.0, v), Vec3::Z * 7.4, 1.0, time);
                assert!((a - b).length() < 1e-9);
            }
        }
    }
}
#[test]
fn cached_render_repeats_pauses_orbits_and_resets_to_identical_pixels() {
    let mut state = LiveState::new(0.0).unwrap();
    let mut viewer = PulsarRenderer::new(192, 144, 4).unwrap();
    let first_stats = viewer.render(&state).unwrap();
    let first = viewer.framebuffer().pixels.clone();
    viewer.render(&state).unwrap();
    assert!(first == viewer.framebuffer().pixels);
    state.advance(0.8, Controls::default()).unwrap();
    viewer.render(&state).unwrap();
    assert!(first != viewer.framebuffer().pixels);
    state
        .advance(
            0.0,
            Controls {
                toggle_pause: true,
                ..Default::default()
            },
        )
        .unwrap();
    viewer.render(&state).unwrap();
    let paused = viewer.framebuffer().pixels.clone();
    state.advance(5.0, Controls::default()).unwrap();
    viewer.render(&state).unwrap();
    assert!(paused == viewer.framebuffer().pixels);
    state
        .advance(
            0.5,
            Controls {
                orbit_x: 1.0,
                orbit_y: 0.3,
                zoom: 0.2,
                ..Default::default()
            },
        )
        .unwrap();
    viewer.render(&state).unwrap();
    assert!(paused != viewer.framebuffer().pixels);
    state
        .advance(
            0.0,
            Controls {
                reset: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(viewer.render(&state).unwrap(), first_stats);
    assert!(first == viewer.framebuffer().pixels);
}
#[test]
fn the_external_magnetic_curves_rotate_instead_of_being_a_fixed_overlay() {
    let mut state = LiveState::new(0.0).unwrap();
    let mut viewer = PulsarRenderer::new(240, 180, 2).unwrap();
    viewer.render(&state).unwrap();
    let before = viewer.framebuffer().pixels.clone();
    state.advance(2.0, Controls::default()).unwrap();
    viewer.render(&state).unwrap();
    let after = &viewer.framebuffer().pixels;
    let changed = before
        .iter()
        .zip(after)
        .enumerate()
        .filter(|(i, (a, b))| {
            let x = (i % 240) as f64 + 0.5 - 120.0;
            let y = (i / 240) as f64 + 0.5 - 90.0;
            x * x + y * y > 45.0 * 45.0 && a != b
        })
        .count();
    assert!(
        changed > 300,
        "magnetic arcs stay frozen: {changed} changed external pixels"
    );
}
#[test]
fn beam_stage_adds_light_on_both_opposite_sides() {
    let state = LiveState::new(0.0).unwrap();
    let mut a = PulsarRenderer::new(320, 240, 2).unwrap();
    let mut b = PulsarRenderer::new(320, 240, 3).unwrap();
    a.render(&state).unwrap();
    b.render(&state).unwrap();
    let mut top = 0;
    let mut bottom = 0;
    for (i, (x, y)) in a
        .framebuffer()
        .pixels
        .iter()
        .zip(&b.framebuffer().pixels)
        .enumerate()
    {
        let px = i % 320;
        let py = i / 320;
        if x == y {
            continue;
        }
        if px > 175 && py < 80 {
            top += 1;
        }
        if px < 145 && py > 160 {
            bottom += 1;
        }
    }
    assert!(
        top > 50 && bottom > 50,
        "opposed beams missing: top={top} bottom={bottom}"
    );
}
#[test]
fn invalid_dimensions_and_stages_are_rejected() {
    for (w, h) in [(0, 32), (32, 0), (usize::MAX, 2)] {
        assert!(PulsarRenderer::new(w, h, 4).is_err());
    }
    for layers in [0, 5, 255] {
        assert!(PulsarRenderer::new(32, 32, layers).is_err());
    }
}
#[test]
fn base_sphere_fits_wide_and_tall_frames_and_orbit_extremes_remain_valid() {
    for (w, h) in [(240, 160), (160, 240)] {
        let mut viewer = PulsarRenderer::new(w, h, 1).unwrap();
        let state = LiveState::new(0.0).unwrap();
        viewer.render(&state).unwrap();
        let fb = viewer.framebuffer();
        assert!((0..h).all(|y| fb.pixels[y * w] == lab5::BACKGROUND
            && fb.pixels[y * w + w - 1] == lab5::BACKGROUND));
        assert!(
            (0..w).all(|x| fb.pixels[x] == lab5::BACKGROUND
                && fb.pixels[(h - 1) * w + x] == lab5::BACKGROUND)
        );
        assert_ne!(fb.pixels[(h / 2) * w + w / 2], lab5::BACKGROUND);
        let mut viewer = PulsarRenderer::new(w, h, 4).unwrap();
        for (pitch, zoom) in [(-1.0, 1.0), (1.0, -1.0)] {
            let mut state = LiveState::new(0.0).unwrap();
            state
                .advance(
                    4.0,
                    Controls {
                        orbit_x: 1.0,
                        orbit_y: pitch,
                        zoom,
                        ..Default::default()
                    },
                )
                .unwrap();
            assert!(viewer.render(&state).unwrap().pixels_written > 100);
        }
    }
}
