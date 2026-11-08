use lab5::{
    math::{Vec2, Vec3},
    procedural::direction,
    renderer::shader::{FragmentInput, to_rgb8},
};
#[test]
fn exhibition_stages_add_light_outside_the_actual_rasterized_sphere() {
    let state = lab5::neutron_live::LiveState::new(0.0).unwrap();
    let mut previous = lab5::pulsar::PulsarRenderer::new(320, 240, 1).unwrap();
    let stats = previous.render(&state).unwrap();
    let mut previous = previous.framebuffer().pixels.clone();
    for stage in 2..=4 {
        let mut viewer = lab5::pulsar::PulsarRenderer::new(320, 240, stage).unwrap();
        assert_eq!(viewer.render(&state).unwrap(), stats);
        let next = &viewer.framebuffer().pixels;
        let external = next
            .iter()
            .zip(&previous)
            .filter(|(a, b)| **a != **b && **b == lab5::BACKGROUND)
            .count();
        println!("stage={stage} newly lit external pixels={external}");
        assert!(
            external > 50,
            "stage {stage} missing its external light contribution"
        );
        previous = next.clone();
    }
}
#[test]
fn magnetic_curves_attach_to_the_sphere_and_reach_the_requested_apex() {
    for longitude in [0.0, 1.3, 5.2] {
        let line = lab5::pulsar::magnetic_line(2.35, longitude, 128).unwrap();
        assert_eq!(line.len(), 129);
        assert!((line[0].length() - 1.0).abs() < 1e-10);
        assert!((line[128].length() - 1.0).abs() < 1e-10);
        assert!(
            line.iter()
                .all(|p| p.is_finite() && p.length() >= 1.0 - 1e-10)
        );
        let maximum = line.iter().map(|p| p.length()).fold(0.0_f64, f64::max);
        assert!(
            (maximum - 2.35).abs() < 1e-10,
            "magnetic loops are missing: max radius {maximum}"
        );
        assert!((line[0] - line[128]).length() > 0.8);
    }
    for l in [f64::NAN, f64::INFINITY, 0.5, 100.0] {
        assert!(lab5::pulsar::magnetic_line(l, 0.0, 128).is_err());
    }
    for phi in [f64::NAN, f64::INFINITY] {
        assert!(lab5::pulsar::magnetic_line(2.0, phi, 128).is_err());
    }
    for count in [0, 1, usize::MAX] {
        assert!(lab5::pulsar::magnetic_line(2.0, 0.0, count).is_err());
    }
}
#[test]
fn exhibition_surface_has_broad_cyan_turbulence_without_whiting_out_the_disk() {
    let mut blue = Vec::new();
    let mut encoded = Vec::new();
    for y in 1..32 {
        for x in 0..64 {
            let uv = Vec2::new(x as f64 / 64.0, y as f64 / 32.0);
            let p = direction(uv);
            if p.z < 0.2 {
                continue;
            }
            let f = FragmentInput {
                uv,
                normal: p,
                world_position: p,
                screen_position: Vec3::ZERO,
            };
            let c = lab5::pulsar::shade_surface(&f, Vec3::Z * 7.4, 1.0, 0.0);
            assert!(c.is_finite() && c.x >= 0.0 && c.y >= 0.0 && c.z >= 0.0);
            blue.push(c.z);
            encoded.push(to_rgb8(c));
        }
    }
    blue.sort_by(f64::total_cmp);
    let median = blue[blue.len() / 2];
    println!("Median surface blue={median}");
    assert!(
        median > 0.30,
        "surface still reads as the old dark globe: {median}"
    );
    let white = encoded
        .iter()
        .filter(|p| ((**p >> 16) & 255) > 245 && ((**p >> 8) & 255) > 245)
        .count();
    assert!(
        white < encoded.len() / 3,
        "preserve visible surface detail, not a white disk"
    );
}
