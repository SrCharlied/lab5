use lab5::{
    math::{Vec2, Vec3},
    neutron,
    renderer::shader::FragmentInput,
};
fn fragment(p: Vec3) -> FragmentInput {
    let p = p.normalized();
    FragmentInput {
        screen_position: Vec3::ZERO,
        world_position: p,
        normal: p,
        uv: Vec2::new(
            p.z.atan2(p.x).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU,
            p.y.clamp(-1.0, 1.0).acos() / std::f64::consts::PI,
        ),
    }
}
#[test]
fn hot_region_has_structure_not_a_perfect_specular_oval() {
    let axis = Vec3::new(0.48, 0.63, 0.61).normalized();
    let a = axis.cross(Vec3::Y).normalized();
    let b = axis.cross(a);
    let p = (axis * 0.99 + a * 0.14).normalized();
    let q = (axis * 0.99 + b * 0.14).normalized();
    let gain = |v| {
        let f = fragment(v);
        neutron::shade(&f, v * 3.5, 2) - neutron::shade(&f, v * 3.5, 1)
    };
    assert!(
        (gain(p) - gain(q)).length() > 0.008,
        "equal magnetic latitude should still have fine heat structure"
    );
}

#[test]
fn preview_rejects_layer_counts_outside_one_to_four() {
    for layers in [0, 5, 255] {
        assert!(
            lab5::render_neutron(32, 32, 0.0, layers).is_err(),
            "reject layer count {layers}"
        );
    }
}

#[test]
fn real_rasterizer_uses_the_star_shader_and_preserves_geometry() {
    let (diagnostic, reference) = lab5::render_diagnostic(192, 128, 0.0).unwrap();
    let (star, stats) = lab5::render_neutron(192, 128, 0.0, 4).unwrap();
    assert!(
        star.pixels != diagnostic.pixels,
        "new route must execute the neutron shader"
    );
    assert_eq!(
        stats, reference,
        "same sphere, camera, clipping and depth workload"
    );
    let (again, again_stats) = lab5::render_neutron(192, 128, 0.0, 4).unwrap();
    assert_eq!(again.pixels, star.pixels);
    assert_eq!(again_stats, stats);
    assert_eq!(star.pixels[0], lab5::BACKGROUND);
}

#[test]
fn fourth_layer_lifts_the_limb_not_the_facing_center() {
    let mut f = fragment(Vec3::Z);
    let camera = Vec3::Z * 3.5;
    let center = neutron::shade(&f, camera, 4) - neutron::shade(&f, camera, 3);
    f.normal = Vec3::X;
    let limb = neutron::shade(&f, camera, 4) - neutron::shade(&f, camera, 3);
    assert!(
        limb.z > 0.07,
        "limb layer must visibly separate the silhouette"
    );
    assert!(center.length() < 1e-12, "rim must not whiten the face");
}

#[test]
fn filaments_add_nonuniform_detail_without_replacing_the_hot_pole() {
    let mut gains = Vec::new();
    for j in 1..18 {
        for i in 0..36 {
            let uv = Vec2::new(i as f64 / 36.0, j as f64 / 18.0);
            let f = FragmentInput {
                uv,
                ..fragment(Vec3::Z)
            };
            let delta = neutron::shade(&f, Vec3::Z * 3.5, 3) - neutron::shade(&f, Vec3::Z * 3.5, 2);
            assert!(delta.is_finite());
            gains.push(delta.length());
        }
    }
    let max = gains.iter().copied().fold(0.0_f64, f64::max);
    let min = gains.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        max > 0.025 && max - min > 0.02,
        "curved detail must actually affect color"
    );
    assert!(max < 0.3, "filaments must remain secondary");
    let f = fragment(Vec3::new(0.48, 0.63, 0.61));
    let delta = neutron::shade(&f, Vec3::Z * 3.5, 3) - neutron::shade(&f, Vec3::Z * 3.5, 2);
    assert!(
        delta.length() < 0.001,
        "do not paint filaments over the hot core"
    );
}

#[test]
fn hot_spots_are_localized_at_two_opposite_magnetic_poles() {
    let axis = Vec3::new(0.48, 0.63, 0.61).normalized();
    let equator = axis.cross(Vec3::Y).normalized();
    for p in [axis, -axis] {
        let f = fragment(p);
        let camera = p * 3.5;
        let delta = neutron::shade(&f, camera, 2) - neutron::shade(&f, camera, 1);
        assert!(delta.x > 0.9, "hot pole must have a nearly white core");
    }
    let f = fragment(equator);
    let delta = neutron::shade(&f, equator * 3.5, 2) - neutron::shade(&f, equator * 3.5, 1);
    assert!(
        delta.length() < 0.001,
        "poles must not become a uniform white tint"
    );
}

#[test]
fn base_layer_is_blue_emissive_and_spatially_varied() {
    let camera = Vec3::new(0.0, 0.0, 3.5);
    let a = neutron::shade(&fragment(Vec3::Z), camera, 1);
    let b = neutron::shade(&fragment(Vec3::new(0.6, 0.2, 0.77)), camera, 1);
    assert!(
        a.z > 0.04 && a.z > a.x * 2.0,
        "surface needs blue emission before hot spots"
    );
    assert!((a - b).length() > 0.001, "surface must not be a flat tint");
    assert!(a.is_finite() && b.is_finite());
}
