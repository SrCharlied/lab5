use lab5::{BACKGROUND, render_diagnostic};
#[test]
fn normal_shader_encodes_direction_without_a_material() {
    use lab5::{
        math::{Vec2, Vec3},
        renderer::shader::{FragmentInput, diagnostic},
    };
    for (normal, expected) in [
        (Vec3::X, Vec3::new(1.0, 0.5, 0.5)),
        (Vec3::Y, Vec3::new(0.5, 1.0, 0.5)),
        (Vec3::Z, Vec3::new(0.5, 0.5, 1.0)),
        (-Vec3::Z, Vec3::new(0.5, 0.5, 0.0)),
    ] {
        let fragment = FragmentInput {
            screen_position: Vec3::ZERO,
            world_position: Vec3::ZERO,
            normal: normal * 3.0,
            uv: Vec2::default(),
        };
        assert!(
            (diagnostic(&fragment) - expected).length() < 1e-12,
            "normal direction must drive RGB"
        );
    }
}

#[test]
fn silhouette_fits_wide_and_tall_frames() {
    for (width, height) in [(192, 128), (128, 192)] {
        let (fb, _) = render_diagnostic(width, height, 0.0).unwrap();
        assert!(
            (0..height).all(|y| fb.pixels[y * width] == BACKGROUND
                && fb.pixels[y * width + width - 1] == BACKGROUND),
            "sphere must not touch side edges at {width}x{height}"
        );
        assert!(fb.pixels[..width].iter().all(|&p| p == BACKGROUND));
        assert!(
            fb.pixels[(height - 1) * width..]
                .iter()
                .all(|&p| p == BACKGROUND)
        );
        let points: Vec<_> = fb
            .pixels
            .iter()
            .enumerate()
            .filter(|(_, p)| **p != BACKGROUND)
            .map(|(i, _)| (i % width, i / width))
            .collect();
        let dx =
            points.iter().map(|p| p.0).max().unwrap() - points.iter().map(|p| p.0).min().unwrap();
        let dy =
            points.iter().map(|p| p.1).max().unwrap() - points.iter().map(|p| p.1).min().unwrap();
        assert!(dx.abs_diff(dy) <= 2, "projected sphere should remain round");
        assert!(
            dx > width.min(height) / 2,
            "sphere must be large enough to inspect"
        );
    }
}

#[test]
fn sphere_has_a_filled_center_and_empty_corners() {
    let (fb, stats) = render_diagnostic(192, 128, 0.0).unwrap();
    assert_ne!(
        fb.pixels[64 * 192 + 96],
        BACKGROUND,
        "the sphere center must be rasterized"
    );
    assert_eq!(fb.pixels[0], BACKGROUND);
    assert_eq!(fb.pixels[fb.pixels.len() - 1], BACKGROUND);
    assert!(stats.triangles_rasterized > 0);
    assert!(stats.pixels_written > 1000);
}
