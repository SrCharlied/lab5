//! Preservation/robustness guards born green; not additional feature REDs.
use lab5::{
    math::{Vec2, Vec3},
    neutron,
    renderer::shader::FragmentInput,
};
fn sample(uv: Vec2, layers: u8) -> Vec3 {
    let f = FragmentInput {
        screen_position: Vec3::ZERO,
        world_position: Vec3::Z,
        normal: Vec3::Z,
        uv,
    };
    neutron::shade(&f, Vec3::Z * 3.5, layers)
}
#[test]
fn longitude_seam_and_geometric_poles_are_continuous() {
    for layers in 1..=4 {
        for j in 0..=24 {
            let v = j as f64 / 24.0;
            assert!(
                (sample(Vec2::new(0.0, v), layers) - sample(Vec2::new(1.0, v), layers)).length()
                    < 1e-10
            );
        }
        for v in [0.0, 1.0] {
            let reference = sample(Vec2::new(0.0, v), layers);
            for i in 0..=24 {
                assert!(
                    (sample(Vec2::new(i as f64 / 24.0, v), layers) - reference).length() < 1e-10
                );
            }
        }
    }
}
#[test]
fn sampled_colors_are_finite_nonnegative_and_repeatable() {
    for layers in 1..=4 {
        for j in 0..=30 {
            for i in 0..=60 {
                let uv = Vec2::new(i as f64 / 60.0, j as f64 / 30.0);
                let c = sample(uv, layers);
                assert!(c.is_finite());
                assert!(c.x >= 0.0 && c.y >= 0.0 && c.z >= 0.0);
                assert!(c.x < 4.0 && c.y < 4.0 && c.z < 4.0);
                assert_eq!(c, sample(uv, layers));
            }
        }
    }
}
#[test]
fn translating_body_and_camera_together_does_not_swim_the_pattern() {
    let f = FragmentInput {
        screen_position: Vec3::ZERO,
        world_position: Vec3::new(0.2, 0.4, 0.8),
        normal: Vec3::Z,
        uv: Vec2::new(0.31, 0.27),
    };
    let camera = Vec3::Z * 3.5;
    let shift = Vec3::new(13.0, -7.0, 4.0);
    let shifted = FragmentInput {
        world_position: f.world_position + shift,
        ..f
    };
    for layers in 1..=4 {
        assert!(
            (neutron::shade(&f, camera, layers) - neutron::shade(&shifted, camera + shift, layers))
                .length()
                < 1e-12
        );
    }
}
#[test]
fn accumulated_layers_change_color_without_changing_geometry_work() {
    let (mut previous, reference) = lab5::render_neutron(192, 128, 0.0, 1).unwrap();
    for layer in 2..=4 {
        let (next, stats) = lab5::render_neutron(192, 128, 0.0, layer).unwrap();
        assert_eq!(stats, reference);
        let changed = next
            .pixels
            .iter()
            .zip(&previous.pixels)
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            changed > 100,
            "layer {layer} must contribute to the rasterized image"
        );
        previous = next;
    }
}
#[test]
fn star_pattern_changes_when_the_sphere_rotates() {
    let (a, _) = lab5::render_neutron(192, 128, 0.0, 4).unwrap();
    let (b, _) = lab5::render_neutron(192, 128, 0.7, 4).unwrap();
    let changed = a
        .pixels
        .iter()
        .zip(&b.pixels)
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 1000);
}
#[test]
fn star_rejects_invalid_size_or_angle_and_fits_portrait() {
    for (w, h) in [(0, 32), (32, 0), (usize::MAX, 2)] {
        assert!(lab5::render_neutron(w, h, 0.0, 4).is_err());
    }
    for angle in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(lab5::render_neutron(32, 32, angle, 4).is_err());
    }
    let (fb, _) = lab5::render_neutron(128, 192, 0.0, 4).unwrap();
    assert!(
        (0..fb.height).all(|y| fb.pixels[y * fb.width] == lab5::BACKGROUND
            && fb.pixels[(y + 1) * fb.width - 1] == lab5::BACKGROUND)
    );
}
#[test]
fn approved_diagnostic_fixture_remains_pixel_identical() {
    use std::io::BufReader;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/diagnostic.png");
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(path).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    let (fb, _) = lab5::render_diagnostic(info.width as usize, info.height as usize, 0.0).unwrap();
    let expected: Vec<u8> = fb
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert!(
        expected == bytes[..info.buffer_size()],
        "accepted diagnostic must not change"
    );
}
