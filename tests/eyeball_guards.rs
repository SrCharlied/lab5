//! Preservation guards for the new candidate, not human aesthetic approval.
use lab5::{
    eyeball,
    math::{Vec2, Vec3},
    procedural::direction,
    renderer::shader::FragmentInput,
};
use std::f64::consts::{PI, TAU};
fn fragment(uv: Vec2) -> FragmentInput {
    let p = direction(uv);
    FragmentInput {
        screen_position: Vec3::ZERO,
        world_position: p,
        normal: p,
        uv,
    }
}
fn uv(p: Vec3) -> Vec2 {
    Vec2::new(
        p.z.atan2(p.x).rem_euclid(TAU) / TAU,
        p.y.clamp(-1.0, 1.0).acos() / PI,
    )
}
#[test]
fn longitude_seam_and_geometric_poles_are_continuous() {
    for layer in 1..=4 {
        for y in 0..=32 {
            let v = y as f64 / 32.0;
            let a = eyeball::shade(&fragment(Vec2::new(0.0, v)), Vec3::Z * 3.5, layer);
            let b = eyeball::shade(&fragment(Vec2::new(1.0, v)), Vec3::Z * 3.5, layer);
            assert!(a.is_finite() && b.is_finite());
            assert!((a - b).length() < 1e-10, "longitude seam, layer {layer}");
        }
        for v in [0.0, 1.0] {
            let a = eyeball::shade(&fragment(Vec2::new(0.0, v)), Vec3::Z * 3.5, layer);
            for x in 0..=32 {
                let b = eyeball::shade(
                    &fragment(Vec2::new(x as f64 / 32.0, v)),
                    Vec3::Z * 3.5,
                    layer,
                );
                assert!(
                    (a - b).length() < 1e-10,
                    "polar discontinuity, layer {layer}"
                );
            }
        }
    }
}
#[test]
fn ocean_is_one_connected_cap_on_sampled_radial_sections() {
    let axis = Vec3::new(-0.37, 0.24, 0.90).normalized();
    let tangent = Vec3::Y.cross(axis).normalized();
    let bitangent = axis.cross(tangent);
    for azimuth in 0..96 {
        let phi = TAU * azimuth as f64 / 96.0;
        let radial = tangent * phi.cos() + bitangent * phi.sin();
        let mut exited = false;
        let mut interiors = 0;
        for step in 0..=240 {
            let rho = PI * step as f64 / 240.0;
            let p = axis * rho.cos() + radial * rho.sin();
            let mask = eyeball::ocean_mask(uv(p));
            assert!(mask.is_finite() && (0.0..=1.0).contains(&mask));
            if rho < 0.35 {
                assert!(mask > 0.99);
            }
            if rho > 0.65 {
                assert_eq!(mask, 0.0, "unexpected second ocean outside the cap");
            }
            if mask > 0.5 {
                assert!(!exited, "disconnected ocean along radial section {azimuth}");
                interiors += 1;
            } else {
                exited = true;
            }
        }
        assert!(interiors > 20 && exited);
    }
}
#[test]
fn every_layer_changes_real_pixels_without_changing_geometry_or_silhouette() {
    let (mut previous, stats) = eyeball::render(192, 128, 0.0, 1).unwrap();
    let (_, diagnostic_stats) = lab5::render_diagnostic(192, 128, 0.0).unwrap();
    assert_eq!(stats, diagnostic_stats);
    for layer in 2..=4 {
        let (next, actual) = eyeball::render(192, 128, 0.0, layer).unwrap();
        assert_eq!(actual, stats);
        let changed = next
            .pixels
            .iter()
            .zip(&previous.pixels)
            .filter(|(a, b)| a != b)
            .count();
        assert!(changed > 100, "layer {layer} changed only {changed} pixels");
        for (a, b) in next.pixels.iter().zip(&previous.pixels) {
            assert_eq!(*a == lab5::BACKGROUND, *b == lab5::BACKGROUND);
        }
        previous = next;
    }
}
#[test]
fn fields_stay_local_and_illumination_does_not_become_emission() {
    let mut sample = fragment(Vec2::new(0.24, 0.4));
    let shift = Vec3::new(17.0, -2.0, 4.0);
    let moved = FragmentInput {
        world_position: sample.world_position + shift,
        ..sample
    };
    for layer in 1..=4 {
        assert_eq!(
            eyeball::shade(&sample, Vec3::Z * 3.5, layer),
            eyeball::shade(&moved, Vec3::Z * 3.5 + shift, layer)
        );
    }
    sample.normal = Vec3::new(-0.75, 0.60, 0.75).normalized();
    let lit = eyeball::shade(&sample, Vec3::Z * 3.5, 4);
    sample.normal = -sample.normal;
    let dark = eyeball::shade(&sample, Vec3::Z * 3.5, 4);
    assert!(dark.length() > 0.0 && dark.length() < lit.length() * 0.1);
}
#[test]
fn renders_repeat_exactly_and_turn_the_eye_out_of_view() {
    let (a, stats) = eyeball::render(192, 128, 0.0, 4).unwrap();
    let (b, other_stats) = eyeball::render(192, 128, 0.0, 4).unwrap();
    assert!(a.pixels == b.pixels);
    assert_eq!(stats, other_stats);
    let (back, _) = eyeball::render(192, 128, PI, 4).unwrap();
    let blue = |p: &u32| {
        let r = (p >> 16) & 255;
        let b = p & 255;
        *p != lab5::BACKGROUND && b > r + 30
    };
    let front = a.pixels.iter().filter(|p| blue(p)).count();
    let rear = back.pixels.iter().filter(|p| blue(p)).count();
    assert!(front > 500, "front ocean not legible: {front}");
    assert_eq!(
        rear, 0,
        "the backside must remain frozen, not contain a second eye"
    );
}
#[test]
fn invalid_render_inputs_are_rejected_and_wide_and_tall_views_fit() {
    for (w, h) in [(0, 32), (32, 0), (usize::MAX, 2)] {
        assert!(eyeball::render(w, h, 0.0, 4).is_err());
    }
    for angle in [f64::NAN, f64::INFINITY] {
        assert!(eyeball::render(32, 32, angle, 4).is_err());
    }
    for layers in [0, 5, 255] {
        assert!(eyeball::render(32, 32, 0.0, layers).is_err());
    }
    for (w, h) in [(192, 128), (128, 192)] {
        let (fb, _) = eyeball::render(w, h, 0.0, 4).unwrap();
        let points: Vec<_> = fb
            .pixels
            .iter()
            .enumerate()
            .filter(|(_, p)| **p != lab5::BACKGROUND)
            .map(|(i, _)| (i % w, i / w))
            .collect();
        let min_x = points.iter().map(|p| p.0).min().unwrap();
        let max_x = points.iter().map(|p| p.0).max().unwrap();
        let min_y = points.iter().map(|p| p.1).min().unwrap();
        let max_y = points.iter().map(|p| p.1).max().unwrap();
        assert!(min_x > 0 && max_x < w - 1 && min_y > 0 && max_y < h - 1);
        assert!((max_x - min_x).abs_diff(max_y - min_y) <= 2);
    }
}
