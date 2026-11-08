use lab5::{
    math::{Vec2, Vec3},
    renderer::shader::FragmentInput,
    rocky,
};
fn fragment(uv: Vec2) -> FragmentInput {
    let p = lab5::procedural::direction(uv);
    FragmentInput {
        screen_position: Vec3::ZERO,
        world_position: p,
        normal: p,
        uv,
    }
}
#[test]
fn real_pipeline_renders_the_rocky_shader_instead_of_the_diagnostic() {
    let (old, old_stats) = lab5::render_diagnostic(192, 128, 0.0).unwrap();
    let (rock, stats) = rocky::render(192, 128, 0.0, 4).unwrap();
    assert!(
        rock.pixels != old.pixels,
        "dispatch must execute the new shader"
    );
    assert_eq!(
        stats, old_stats,
        "same sphere/camera pipeline must preserve geometry workload"
    );
    assert_ne!(rock.pixels[64 * 192 + 96], lab5::BACKGROUND);
    assert_eq!(rock.pixels[0], lab5::BACKGROUND);
}

#[test]
fn fourth_layer_adds_sparse_clouds_without_hiding_all_the_terrain() {
    let mut changed = 0;
    let mut clear = 0;
    let mut total = 0;
    for j in 1..24 {
        for i in 0..48 {
            let f = fragment(Vec2::new(i as f64 / 48.0, j as f64 / 24.0));
            let a = rocky::shade(&f, Vec3::Z * 3.5, 3);
            let b = rocky::shade(&f, Vec3::Z * 3.5, 4);
            if (a - b).length() > 0.01 {
                changed += 1;
            } else {
                clear += 1;
            }
            assert!(b.is_finite());
            total += 1;
        }
    }
    println!("cloud pixels={changed} clear={clear} total={total}");
    assert!(changed > total / 30, "clouds must affect the actual color");
    assert!(
        clear > total / 3,
        "leave large clear regions for the rocky identity"
    );
}

#[test]
fn third_layer_adds_polar_ice_without_whitening_the_equator() {
    let camera = Vec3::Z * 3.5;
    let pole = fragment(Vec2::new(0.25, 0.02));
    let a = rocky::shade(&pole, camera, 2);
    let b = rocky::shade(&pole, camera, 3);
    assert!(
        (a - b).length() > 0.15,
        "polar ice must be an actual color layer"
    );
    assert!(b.z > a.z + 0.1, "ice should lift the cold channels");
    let equator = fragment(Vec2::new(0.25, 0.5));
    assert_eq!(
        rocky::shade(&equator, camera, 2),
        rocky::shade(&equator, camera, 3),
        "ice must not wash out equatorial geology"
    );
}

#[test]
fn second_layer_creates_teal_basins_while_preserving_exposed_rock() {
    let mut changed = 0;
    let mut rock = 0;
    let mut teal = 0;
    let mut total = 0;
    for j in 3..21 {
        for i in 0..48 {
            let f = fragment(Vec2::new(i as f64 / 48.0, j as f64 / 24.0));
            let a = rocky::shade(&f, Vec3::Z * 3.5, 1);
            let b = rocky::shade(&f, Vec3::Z * 3.5, 2);
            if (a - b).length() > 0.01 {
                changed += 1;
                if b.y > b.x * 1.2 && b.z > b.x {
                    teal += 1;
                }
            } else {
                rock += 1;
            }
            total += 1;
        }
    }
    println!("basins changed={changed} preserved_rock={rock} teal={teal} total={total}");
    assert!(
        changed > total / 8,
        "basins must visibly affect the surface"
    );
    assert!(
        rock > total / 10,
        "this must remain legible as a rocky planet"
    );
    assert!(
        teal > changed / 2,
        "the water must have a distinct cool palette"
    );
}

#[test]
fn first_layer_is_varied_warm_geology_not_a_flat_or_emissive_surface() {
    let camera = Vec3::Z * 3.5;
    let a = rocky::shade(&fragment(Vec2::new(0.25, 0.5)), camera, 1);
    let b = rocky::shade(&fragment(Vec2::new(0.30, 0.35)), camera, 1);
    assert!(
        a.x > 0.02 && a.x > a.z * 1.2,
        "rock should have a warm mineral identity"
    );
    assert!(
        (a - b).length() > 0.005,
        "procedural geology must affect color"
    );
    assert!(a.is_finite() && b.is_finite());
}
