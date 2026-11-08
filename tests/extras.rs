use lab5::{
    extras::{self, Body},
    math::{Vec2, Vec3},
    procedural::direction,
    renderer::shader::FragmentInput,
};
fn sample(u: f64, v: f64) -> FragmentInput {
    let uv = Vec2::new(u, v);
    let p = direction(uv);
    FragmentInput {
        uv,
        normal: p,
        world_position: p,
        screen_position: Vec3::ZERO,
    }
}
#[test]
fn moon_is_a_separate_rasterized_sphere_and_keeps_the_planet_pixels() {
    let (without, one) = extras::render(320, 240, 0.4, 4, Body::Desert, false).unwrap();
    let (with, two) = extras::render(320, 240, 0.4, 4, Body::Desert, true).unwrap();
    assert_eq!(two.objects_submitted, 2, "moon requires its own model draw");
    assert!(two.vertices_processed > one.vertices_processed);
    let mut moon_pixels = 0;
    for (a, b) in without.pixels.iter().zip(&with.pixels) {
        if *a != lab5::BACKGROUND {
            assert_eq!(a, b, "moon must not paint over this separated planet");
        } else if *b != lab5::BACKGROUND {
            moon_pixels += 1;
        }
    }
    assert!(
        moon_pixels > 80 && moon_pixels < 1500,
        "small visible moon missing: {moon_pixels}"
    );
}
#[test]
fn temperate_has_separate_ocean_land_and_ice_regions() {
    let light = Vec3::new(-0.65, 0.8, 1.0).normalized();
    let (mut sea, mut land, mut ice) = (0, 0, 0);
    for j in 1..32 {
        for i in 0..64 {
            let mut f = sample(i as f64 / 64.0, j as f64 / 32.0);
            f.normal = light;
            let c = extras::shade(Body::Temperate, &f, 2);
            if c.z > c.y * 1.05 && c.y > c.x * 2.0 {
                sea += 1;
            }
            if c.y > c.x * 1.3 && c.y > c.z * 1.3 {
                land += 1;
            }
            let full = extras::shade(Body::Temperate, &f, 3);
            if full.x > 0.3 && full.y > 0.3 && full.z > 0.3 {
                ice += 1;
            }
        }
    }
    assert!(sea > 200, "oceans missing: {sea}");
    assert!(land > 150, "continents missing: {land}");
    assert!(ice > 50, "polar ice missing: {ice}");
}
#[test]
fn desert_has_warm_nonemissive_terrain_with_spatial_variation() {
    let light = Vec3::new(-0.65, 0.8, 1.0).normalized();
    let mut warm = 0;
    let mut lo = f64::INFINITY;
    let mut hi = 0.0_f64;
    for j in 1..24 {
        for i in 0..48 {
            let mut f = sample(i as f64 / 48.0, j as f64 / 24.0);
            f.normal = light;
            let c = extras::shade(Body::Desert, &f, 4);
            f.normal = -light;
            let night = extras::shade(Body::Desert, &f, 4);
            assert!(
                night.length() < c.length() * 0.15,
                "desert must not emit lava"
            );
            if c.x > c.y * 1.1 && c.y > c.z * 1.2 {
                warm += 1;
            }
            lo = lo.min(c.x);
            hi = hi.max(c.x);
        }
    }
    assert!(warm > 900, "warm sand missing: {warm}");
    assert!(hi - lo > 0.08, "terrain is a flat tint");
}
#[test]
fn volcanic_has_dark_crust_and_localized_red_emission_on_the_night_side() {
    let mut hot = 0;
    let mut crust = 0;
    let light = Vec3::new(-0.65, 0.8, 1.0).normalized();
    for j in 1..32 {
        for i in 0..64 {
            let mut f = sample(i as f64 / 64.0, j as f64 / 32.0);
            f.normal = -light;
            let c = extras::shade(Body::Volcanic, &f, 4);
            if c.x > 0.3 && c.x > c.y * 2.0 {
                hot += 1;
            }
            if c.length() < 0.06 {
                crust += 1;
            }
        }
    }
    assert!(hot > 40 && hot < 1000, "localized lava missing: {hot}");
    assert!(crust > 500, "dark basalt missing: {crust}");
}
