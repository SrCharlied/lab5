use lab5::{
    eyeball,
    math::{Vec2, Vec3},
    procedural::direction,
    renderer::shader::FragmentInput,
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
fn first_layer_is_an_illuminated_varied_beige_ice_shell() {
    let a = eyeball::shade(&f(Vec2::new(0.18, 0.45)), Vec3::Z * 3.5, 1);
    let b = eyeball::shade(&f(Vec2::new(0.26, 0.65)), Vec3::Z * 3.5, 1);
    assert!(
        a.x > 0.10 && a.x > a.z,
        "exterior must be pale warm ice, not blue ocean or black"
    );
    assert!(
        (a - b).length() > 0.005,
        "shell must have procedural variation and illumination"
    );
}

#[test]
fn second_layer_adds_visible_fractures_without_replacing_the_shell() {
    let mut changed = 0;
    let mut maximum = 0.0_f64;
    for y in 1..48 {
        for x in 0..96 {
            let f = f(Vec2::new(x as f64 / 96.0, y as f64 / 48.0));
            if f.normal.z <= 0.0 {
                continue;
            }
            let a = eyeball::shade(&f, Vec3::Z * 3.5, 1);
            let b = eyeball::shade(&f, Vec3::Z * 3.5, 2);
            let delta = (a - b).length();
            maximum = maximum.max(delta);
            if delta > 0.01 {
                changed += 1;
            }
        }
    }
    assert!(
        changed > 16 && maximum > 0.03,
        "fracture contribution missing: {changed} samples, max {maximum}"
    );
}

#[test]
fn third_layer_has_one_bounded_front_ocean_and_no_antipodal_copy() {
    let axis = Vec3::new(-0.37, 0.24, 0.90).normalized();
    let mut interior = 0;
    let mut changed = 0;
    for y in 1..64 {
        for x in 0..128 {
            let uv = Vec2::new(x as f64 / 128.0, y as f64 / 64.0);
            let f = f(uv);
            let rho = f.normal.dot(axis).clamp(-1.0, 1.0).acos();
            let mask = eyeball::ocean_mask(uv);
            assert!(mask.is_finite() && (0.0..=1.0).contains(&mask));
            if rho < 0.35 {
                assert!(mask > 0.99);
                interior += 1;
            }
            if rho > 0.65 {
                assert_eq!(mask, 0.0, "ocean must not leak onto the rest of the sphere");
            }
            let a = eyeball::shade(&f, Vec3::Z * 3.5, 2);
            let b = eyeball::shade(&f, Vec3::Z * 3.5, 3);
            if rho > 0.65 {
                assert_eq!(a, b);
            }
            if (a - b).length() > 0.02 {
                changed += 1;
            }
        }
    }
    assert!(interior > 10 && changed > 50);
}

#[test]
fn fourth_layer_is_a_bright_structured_cloud_only_inside_the_ocean() {
    let axis = Vec3::new(-0.37, 0.24, 0.90).normalized();
    let uv = Vec2::new(
        axis.z.atan2(axis.x).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU,
        axis.y.acos() / std::f64::consts::PI,
    );
    let core = eyeball::shade(&f(uv), Vec3::Z * 3.5, 4);
    assert!(
        core.x > 0.45 && (core.x - core.z).abs() < 0.08,
        "central cloud must read white: {core:?}"
    );
    let mut changed = 0;
    let mut clear_water = 0;
    for y in 1..64 {
        for x in 0..128 {
            let uv = Vec2::new(x as f64 / 128.0, y as f64 / 64.0);
            let f = f(uv);
            let a = eyeball::shade(&f, Vec3::Z * 3.5, 3);
            let b = eyeball::shade(&f, Vec3::Z * 3.5, 4);
            let mask = eyeball::ocean_mask(uv);
            if mask == 0.0 {
                assert_eq!(a, b, "clouds must not cover the frozen exterior");
            }
            if (a - b).length() > 0.04 {
                changed += 1;
            }
            if mask > 0.99 && (a - b).length() < 0.01 {
                clear_water += 1;
            }
        }
    }
    assert!(
        changed > 30 && clear_water > 10,
        "need structured cloud and exposed ocean"
    );
}

#[test]
fn render_selects_the_eye_shader_on_the_actual_sphere() {
    let (base, base_stats) = eyeball::render(160, 120, 0.0, 1).unwrap();
    let (full, full_stats) = eyeball::render(160, 120, 0.0, 4).unwrap();
    let (diagnostic, _) = lab5::render_diagnostic(160, 120, 0.0).unwrap();
    assert!(
        full.pixels != diagnostic.pixels,
        "eye route still calls diagnostic"
    );
    assert!(
        base.pixels != full.pixels,
        "real framebuffer ignores layers"
    );
    assert_eq!(base_stats.pixels_written, full_stats.pixels_written);
    assert_eq!(
        base_stats.triangles_rasterized,
        full_stats.triangles_rasterized
    );
    assert!(full_stats.pixels_written > 1000);
}

#[test]
fn outer_cloud_arms_remain_translucent_instead_of_opaque_white_ribbons() {
    let axis = Vec3::new(-0.37, 0.24, 0.90).normalized();
    let mut maximum_lift = 0.0_f64;
    let mut samples = 0;
    for y in 1..100 {
        for x in 0..200 {
            let mut sample = f(Vec2::new(x as f64 / 200.0, y as f64 / 100.0));
            let angle = sample.normal.dot(axis).clamp(-1.0, 1.0).acos();
            if !(0.18..0.40).contains(&angle) {
                continue;
            }
            // Isolate color mixing from the varying diffuse factor.
            sample.normal = Vec3::new(-0.75, 0.60, 0.75).normalized();
            let water = eyeball::shade(&sample, Vec3::Z * 3.5, 3);
            let cloud = eyeball::shade(&sample, Vec3::Z * 3.5, 4);
            maximum_lift = maximum_lift.max(cloud.x - water.x);
            samples += 1;
        }
    }
    println!("outer-cloud samples={samples}, maximum red lift={maximum_lift}");
    assert!(samples > 100);
    assert!(
        maximum_lift < 0.34,
        "outer arms overpower the dark ocean: {maximum_lift}"
    );
    assert!(maximum_lift > 0.05, "outer spiral must still be visible");
}
