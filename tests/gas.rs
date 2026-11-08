use lab5::{
    math::{Vec2, Vec3},
    procedural::direction,
    renderer::shader::FragmentInput,
};
#[test]
fn fourth_layer_adds_haze_and_a_cold_inner_limb() {
    let mut changed = 0;
    for j in 1..32 {
        for i in 0..64 {
            let f = sample(Vec2::new(i as f64 / 64.0, j as f64 / 32.0));
            let a = lab5::gas::shade(&f, Vec3::Z * 3.5, 3);
            let b = lab5::gas::shade(&f, Vec3::Z * 3.5, 4);
            assert!(b.is_finite());
            if (a - b).length() > 0.005 {
                changed += 1;
            }
        }
    }
    assert!(changed > 200, "haze contribution missing: {changed}");
}
#[test]
fn real_renderer_selects_gas_and_keeps_all_geometry_statistics() {
    let (diagnostic, reference) = lab5::render_diagnostic(192, 128, 0.0).unwrap();
    let (full, stats) = lab5::gas::render(192, 128, 0.0, 4).unwrap();
    assert!(
        full.pixels != diagnostic.pixels,
        "gas route still calls diagnostic"
    );
    assert_eq!(stats, reference);
    let (clear, other) = lab5::gas::render(192, 128, 0.0, 3).unwrap();
    assert_eq!(stats, other);
    assert!(
        full.pixels
            .iter()
            .zip(&clear.pixels)
            .filter(|(a, b)| a != b)
            .count()
            > 100
    );
}
#[test]
fn third_layer_adds_one_localized_storm_without_covering_the_planet() {
    let axis = Vec3::new(-0.30, -0.22, 0.93).normalized();
    let mut changed = 0;
    let mut maximum = 0.0_f64;
    for j in 1..64 {
        for i in 0..128 {
            let f = sample(Vec2::new(i as f64 / 128.0, j as f64 / 64.0));
            let a = lab5::gas::shade(&f, Vec3::Z * 3.5, 2);
            let b = lab5::gas::shade(&f, Vec3::Z * 3.5, 3);
            let delta = (a - b).length();
            maximum = maximum.max(delta);
            if f.normal.dot(axis) < 0.84 {
                assert_eq!(a, b, "storm escaped its bounded region");
            }
            if delta > 0.02 {
                changed += 1;
            }
        }
    }
    assert!(
        changed > 30 && changed < 500 && maximum > 0.1,
        "storm missing or unbounded: changed={changed}, max={maximum}"
    );
}
#[test]
fn second_layer_deforms_bands_into_longitude_dependent_currents() {
    let mut changed = 0;
    let mut sheared = 0;
    for j in 4..28 {
        for i in 0..48 {
            let mut a = sample(Vec2::new(i as f64 / 48.0, j as f64 / 32.0));
            let mut b = sample(Vec2::new((i as f64 + 7.0) / 48.0, j as f64 / 32.0));
            a.normal = Vec3::new(-0.65, 0.8, 1.0).normalized();
            b.normal = a.normal;
            let base = lab5::gas::shade(&a, Vec3::Z * 3.5, 1);
            let current = lab5::gas::shade(&a, Vec3::Z * 3.5, 2);
            let other = lab5::gas::shade(&b, Vec3::Z * 3.5, 2);
            if (base - current).length() > 0.02 {
                changed += 1;
            }
            if (current - other).length() > 0.02 {
                sheared += 1;
            }
        }
    }
    assert!(
        changed > 200 && sheared > 200,
        "currents missing: changed={changed}, longitude response={sheared}"
    );
}
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
fn first_layer_has_blue_latitude_bands_instead_of_a_flat_ball() {
    let mut values = Vec::new();
    for j in 8..25 {
        let c = lab5::gas::shade(&sample(Vec2::new(0.25, j as f64 / 32.0)), Vec3::Z * 3.5, 1);
        assert!(
            c.is_finite() && c.z > c.x * 2.0 && c.y > c.x,
            "expected a blue gas palette"
        );
        values.push(c.z);
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(0.0, f64::max);
    assert!(
        max - min > 0.12,
        "latitude bands missing: range {}",
        max - min
    );
}
