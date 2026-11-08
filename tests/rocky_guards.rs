//! Preservation/robustness checks for copied math and the implemented rocky path.
use lab5::{
    math::{Vec2, Vec3},
    procedural::{direction, field, noise},
    renderer::shader::FragmentInput,
    rocky,
};
fn f(uv: Vec2) -> FragmentInput {
    let p = direction(uv);
    FragmentInput {
        screen_position: Vec3::ZERO,
        world_position: p,
        normal: p,
        uv,
    }
}
#[test]
fn sphere_seam_and_both_poles_are_continuous() {
    for layer in 1..=4 {
        for j in 0..=24 {
            let v = j as f64 / 24.0;
            let a = rocky::shade(&f(Vec2::new(0.0, v)), Vec3::Z * 3.5, layer);
            let b = rocky::shade(&f(Vec2::new(1.0, v)), Vec3::Z * 3.5, layer);
            assert!((a - b).length() < 1e-10);
        }
        for v in [0.0, 1.0] {
            let a = rocky::shade(&f(Vec2::new(0.0, v)), Vec3::Z * 3.5, layer);
            for i in 0..=24 {
                let b = rocky::shade(&f(Vec2::new(i as f64 / 24.0, v)), Vec3::Z * 3.5, layer);
                assert!((a - b).length() < 1e-10);
            }
        }
    }
}
#[test]
fn procedural_fields_are_finite_bounded_and_repeatable_on_the_used_domain() {
    for j in 0..=24 {
        for i in 0..=48 {
            let p = direction(Vec2::new(i as f64 / 48.0, j as f64 / 24.0));
            assert!((p.length() - 1.0).abs() < 1e-12);
            for scale in [1.0, 7.5, 31.0] {
                let q = p * scale + Vec3::splat(2.4);
                for value in [noise(q), field(q)] {
                    assert!(value.is_finite() && (-1e-12..=1.0 + 1e-12).contains(&value));
                }
                assert_eq!(field(q), field(q));
            }
        }
    }
    assert!(
        (noise(Vec3::new(-1.0 - 1e-7, 0.37, 2.41)) - noise(Vec3::new(-1.0 + 1e-7, 0.37, 2.41)))
            .abs()
            < 1e-5
    );
}
#[test]
fn all_layers_change_the_render_without_changing_geometry_statistics() {
    let (mut previous, reference) = rocky::render(192, 128, 0.0, 1).unwrap();
    for layer in 2..=4 {
        let (next, stats) = rocky::render(192, 128, 0.0, layer).unwrap();
        assert_eq!(stats, reference);
        assert!(
            next.pixels
                .iter()
                .zip(&previous.pixels)
                .filter(|(a, b)| a != b)
                .count()
                > 100
        );
        previous = next;
    }
}
#[test]
fn illumination_changes_the_surface_instead_of_emitting_like_the_star() {
    let mut sample = f(Vec2::new(0.25, 0.4));
    sample.normal = Vec3::new(-0.65, 0.8, 1.0).normalized();
    let lit = rocky::shade(&sample, Vec3::Z * 3.5, 4);
    sample.normal = -sample.normal;
    let dark = rocky::shade(&sample, Vec3::Z * 3.5, 4);
    assert!(dark.length() < lit.length() * 0.2);
    assert!(dark.length() > 0.0);
}
#[test]
fn render_is_deterministic_and_the_pattern_is_fixed_to_local_coordinates() {
    let (a, sa) = rocky::render(192, 128, 0.0, 4).unwrap();
    let (b, sb) = rocky::render(192, 128, 0.0, 4).unwrap();
    assert!(a.pixels == b.pixels);
    assert_eq!(sa, sb);
    let (turned, _) = rocky::render(192, 128, 0.7, 4).unwrap();
    assert!(
        a.pixels
            .iter()
            .zip(&turned.pixels)
            .filter(|(a, b)| a != b)
            .count()
            > 1000
    );
    let sample = f(Vec2::new(0.3, 0.45));
    let shift = Vec3::new(15.0, -3.0, 4.0);
    let moved = FragmentInput {
        world_position: sample.world_position + shift,
        ..sample
    };
    for layer in 1..=4 {
        assert_eq!(
            rocky::shade(&sample, Vec3::Z * 3.5, layer),
            rocky::shade(&moved, Vec3::Z * 3.5 + shift, layer)
        );
    }
}
#[test]
fn invalid_inputs_are_rejected_and_portrait_keeps_a_round_silhouette() {
    for (w, h) in [(0, 32), (32, 0), (usize::MAX, 2)] {
        assert!(rocky::render(w, h, 0.0, 4).is_err());
    }
    for angle in [f64::NAN, f64::INFINITY] {
        assert!(rocky::render(32, 32, angle, 4).is_err());
    }
    for layers in [0, 5, 255] {
        assert!(rocky::render(32, 32, 0.0, layers).is_err());
    }
    let (fb, _) = rocky::render(128, 192, 0.0, 4).unwrap();
    assert!(
        (0..fb.height).all(|y| fb.pixels[y * fb.width] == lab5::BACKGROUND
            && fb.pixels[(y + 1) * fb.width - 1] == lab5::BACKGROUND)
    );
    let points: Vec<_> = fb
        .pixels
        .iter()
        .enumerate()
        .filter(|(_, p)| **p != lab5::BACKGROUND)
        .map(|(i, _)| (i % fb.width, i / fb.width))
        .collect();
    let dx = points.iter().map(|p| p.0).max().unwrap() - points.iter().map(|p| p.0).min().unwrap();
    let dy = points.iter().map(|p| p.1).max().unwrap() - points.iter().map(|p| p.1).min().unwrap();
    assert!(dx.abs_diff(dy) <= 2);
}
