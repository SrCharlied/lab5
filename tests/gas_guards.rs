//! Preservation/robustness guards; born green, mutation evidence kept separately.
use lab5::{
    gas,
    math::{Vec2, Vec3},
    procedural::direction,
    renderer::shader::FragmentInput,
};
fn sample(uv: Vec2) -> FragmentInput {
    let p = direction(uv);
    FragmentInput {
        uv,
        normal: p,
        world_position: p,
        screen_position: Vec3::ZERO,
    }
}
#[test]
fn longitude_seam_and_both_poles_are_continuous() {
    for layer in 1..=4 {
        for j in 0..=32 {
            let v = j as f64 / 32.0;
            let a = gas::shade(&sample(Vec2::new(0.0, v)), Vec3::Z * 3.5, layer);
            let b = gas::shade(&sample(Vec2::new(1.0, v)), Vec3::Z * 3.5, layer);
            assert!(a.is_finite() && b.is_finite());
            assert!((a - b).length() < 1e-10);
        }
        for v in [0.0, 1.0] {
            let a = gas::shade(&sample(Vec2::new(0.0, v)), Vec3::Z * 3.5, layer);
            for i in 0..=32 {
                let b = gas::shade(&sample(Vec2::new(i as f64 / 32.0, v)), Vec3::Z * 3.5, layer);
                assert!((a - b).length() < 1e-10);
            }
        }
    }
}
#[test]
fn every_layer_changes_pixels_without_changing_geometry_or_silhouette() {
    let (mut previous, reference) = gas::render(192, 128, 0.0, 1).unwrap();
    for layer in 2..=4 {
        let (next, stats) = gas::render(192, 128, 0.0, layer).unwrap();
        assert_eq!(stats, reference);
        let changed = next
            .pixels
            .iter()
            .zip(&previous.pixels)
            .filter(|(a, b)| a != b)
            .count();
        assert!(changed > 100, "layer {layer} changes only {changed} pixels");
        for (a, b) in next.pixels.iter().zip(&previous.pixels) {
            assert_eq!(*a == lab5::BACKGROUND, *b == lab5::BACKGROUND);
        }
        previous = next;
    }
}
#[test]
fn procedural_identity_stays_in_local_coordinates_when_the_body_translates() {
    let shift = Vec3::new(13.0, -4.0, 8.0);
    for j in 1..16 {
        for i in 0..32 {
            let f = sample(Vec2::new(i as f64 / 32.0, j as f64 / 16.0));
            let moved = FragmentInput {
                world_position: f.world_position + shift,
                ..f
            };
            for layer in 1..=4 {
                let a = gas::shade(&f, Vec3::Z * 3.5, layer);
                let b = gas::shade(&moved, Vec3::Z * 3.5 + shift, layer);
                if layer < 4 {
                    assert_eq!(a, b, "local atmospheric fields must remain bit-identical");
                } else {
                    // (camera+shift)-(point+shift) can round differently from camera-point.
                    // Only the view-dependent limb admits floating-point roundoff.
                    assert!(
                        (a - b).length() < 1e-12,
                        "procedural pattern swims when translated"
                    );
                }
            }
        }
    }
}
#[test]
fn atmosphere_is_lit_not_emissive_like_the_star() {
    let mut f = sample(Vec2::new(0.25, 0.4));
    f.normal = Vec3::new(-0.65, 0.8, 1.0).normalized();
    let lit = gas::shade(&f, Vec3::Z * 3.5, 4);
    f.normal = -f.normal;
    let dark = gas::shade(&f, Vec3::Z * 3.5, 4);
    assert!(dark.length() > 0.0 && dark.length() < lit.length() * 0.20);
}
#[test]
fn render_is_repeatable_and_rotation_changes_the_weather_map() {
    let (a, sa) = gas::render(192, 128, 0.0, 4).unwrap();
    let (b, sb) = gas::render(192, 128, 0.0, 4).unwrap();
    assert_eq!(sa, sb);
    assert!(a.pixels == b.pixels);
    let (turned, _) = gas::render(192, 128, 0.9, 4).unwrap();
    assert!(
        a.pixels
            .iter()
            .zip(&turned.pixels)
            .filter(|(a, b)| a != b)
            .count()
            > 1000
    );
}
#[test]
fn invalid_inputs_are_rejected_and_wide_and_tall_frames_keep_the_sphere() {
    for (w, h) in [(0, 32), (32, 0), (usize::MAX, 2)] {
        assert!(gas::render(w, h, 0.0, 4).is_err());
    }
    for angle in [f64::NAN, f64::INFINITY] {
        assert!(gas::render(32, 32, angle, 4).is_err());
    }
    for layers in [0, 5, 255] {
        assert!(gas::render(32, 32, 0.0, layers).is_err());
    }
    for (w, h) in [(192, 128), (128, 192)] {
        let (fb, _) = gas::render(w, h, 0.0, 4).unwrap();
        let points: Vec<_> = fb
            .pixels
            .iter()
            .enumerate()
            .filter(|(_, p)| **p != lab5::BACKGROUND)
            .map(|(i, _)| (i % w, i / w))
            .collect();
        let minx = points.iter().map(|p| p.0).min().unwrap();
        let maxx = points.iter().map(|p| p.0).max().unwrap();
        let miny = points.iter().map(|p| p.1).min().unwrap();
        let maxy = points.iter().map(|p| p.1).max().unwrap();
        assert!(minx > 0 && miny > 0 && maxx < w - 1 && maxy < h - 1);
        assert!((maxx - minx).abs_diff(maxy - miny) <= 2);
    }
}
