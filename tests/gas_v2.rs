use lab5::{
    math::{Vec2, Vec3},
    procedural::direction,
    renderer::shader::FragmentInput,
};
#[test]
fn fourth_layer_adds_soft_haze_without_erasing_filament_contrast() {
    let mut changed = 0;
    for j in 1..32 {
        for i in 0..64 {
            let sample = f(Vec2::new(i as f64 / 64.0, j as f64 / 32.0));
            let a = lab5::gas_v2::shade(&sample, Vec3::Z * 3.5, 3);
            let b = lab5::gas_v2::shade(&sample, Vec3::Z * 3.5, 4);
            assert!(b.is_finite());
            let difference = (a - b).length();
            assert!(difference < 0.20, "haze hides the structure");
            if difference > 0.003 {
                changed += 1;
            }
        }
    }
    assert!(changed > 200, "soft haze is missing: {changed}");
}
#[test]
fn real_v2_pipeline_changes_pattern_but_not_camera_mesh_or_geometry_statistics() {
    let (old, reference) = lab5::gas::render(192, 128, 0.0, 4).unwrap();
    let (new, actual) = lab5::gas_v2::render(192, 128, 0.0, 4).unwrap();
    assert_eq!(reference, actual);
    assert!(
        old.pixels != new.pixels,
        "V2 route still calls the banded shader"
    );
    for (a, b) in old.pixels.iter().zip(&new.pixels) {
        assert_eq!(*a == lab5::BACKGROUND, *b == lab5::BACKGROUND);
    }
}
#[test]
fn third_layer_adds_pale_filaments_globally_not_an_isolated_spiral() {
    let mut bins = [0usize; 8];
    let mut clear = 0;
    for j in 1..48 {
        for i in 0..96 {
            let uv = Vec2::new(i as f64 / 96.0, j as f64 / 48.0);
            let p = direction(uv);
            let sample = f(uv);
            let a = lab5::gas_v2::shade(&sample, Vec3::Z * 3.5, 2);
            let b = lab5::gas_v2::shade(&sample, Vec3::Z * 3.5, 3);
            if (a - b).length() > 0.08 {
                let bin = usize::from(p.x > 0.0)
                    + 2 * usize::from(p.y > 0.0)
                    + 4 * usize::from(p.z > 0.0);
                bins[bin] += 1;
                assert!(b.x > a.x);
            }
            if (a - b).length() < 0.01 {
                clear += 1;
            }
        }
    }
    assert!(
        bins.iter().all(|n| *n > 10) && clear > 500,
        "filaments missing or cover everything: bins={bins:?} clear={clear}"
    );
}
#[test]
fn second_layer_changes_cloud_structure_across_all_eight_spatial_octants() {
    let mut bins = [0usize; 8];
    for j in 1..48 {
        for i in 0..96 {
            let uv = Vec2::new(i as f64 / 96.0, j as f64 / 48.0);
            let p = direction(uv);
            let sample = f(uv);
            let a = lab5::gas_v2::shade(&sample, Vec3::Z * 3.5, 1);
            let b = lab5::gas_v2::shade(&sample, Vec3::Z * 3.5, 2);
            assert!(b.is_finite());
            if (a - b).length() > 0.02 {
                let bin = usize::from(p.x > 0.0)
                    + 2 * usize::from(p.y > 0.0)
                    + 4 * usize::from(p.z > 0.0);
                bins[bin] += 1;
            }
        }
    }
    assert!(
        bins.iter().all(|n| *n > 30),
        "distributed turbulence missing: {bins:?}"
    );
}
fn f(uv: Vec2) -> FragmentInput {
    let p = direction(uv);
    FragmentInput {
        uv,
        normal: Vec3::new(-0.65, 0.8, 1.0).normalized(),
        world_position: p,
        screen_position: Vec3::ZERO,
    }
}
#[test]
fn base_has_longitudinal_cloud_masses_instead_of_regular_latitude_stripes() {
    for v in [0.27, 0.43, 0.61] {
        let values: Vec<_> = (0..96)
            .map(|i| lab5::gas_v2::shade(&f(Vec2::new(i as f64 / 96.0, v)), Vec3::Z * 3.5, 1))
            .collect();
        assert!(values.iter().all(|c| c.is_finite() && c.z > c.x * 2.0));
        let low = values.iter().map(|c| c.z).fold(f64::INFINITY, f64::min);
        let high = values.iter().map(|c| c.z).fold(0.0_f64, f64::max);
        assert!(
            high - low > 0.025,
            "regular zonal bands remain at v={v}: range={}",
            high - low
        );
    }
}
