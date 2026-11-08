//! Guards for already accepted/copied behavior; not feature RED/GREEN claims.
use lab5::{
    BACKGROUND,
    math::{Vec2, Vec3, Vec4},
    render_diagnostic,
    renderer::{
        clipping::{VertexOut, clip_near},
        framebuffer::Framebuffer,
        rasterizer::{RasterVertex, triangle_shaded},
        shader::to_rgb8,
    },
};
#[test]
fn same_scene_produces_identical_pixels() {
    let (a, sa) = render_diagnostic(192, 128, 0.7).unwrap();
    let (b, sb) = render_diagnostic(192, 128, 0.7).unwrap();
    assert_eq!(a.pixels, b.pixels);
    assert_eq!(sa, sb);
}
#[test]
fn invalid_sizes_and_angles_are_rejected() {
    for (w, h) in [(0, 128), (128, 0), (usize::MAX, 2)] {
        assert!(render_diagnostic(w, h, 0.0).is_err());
    }
    assert!(render_diagnostic(128, 128, f64::NAN).is_err());
    assert!(render_diagnostic(128, 128, f64::INFINITY).is_err());
}
#[test]
fn normal_image_has_correct_orientation_and_front_face() {
    let (fb, _) = render_diagnostic(192, 128, 0.0).unwrap();
    let at = |x: usize, y: usize| fb.pixels[y * 192 + x];
    let red = |p: u32| (p >> 16) & 255;
    let green = |p: u32| (p >> 8) & 255;
    assert!(red(at(70, 64)) < red(at(122, 64)));
    assert!(green(at(96, 40)) > green(at(96, 88)));
    assert!((at(96, 64) & 255) > 250, "visible center must face +Z");
    assert_ne!(at(96, 64), BACKGROUND);
}
#[test]
fn near_plane_clipping_keeps_only_valid_vertices() {
    let v = |z| VertexOut {
        clip_position: Vec4::new(0.0, 0.0, z, 1.0),
        world_position: Vec3::ZERO,
        world_normal: Vec3::Z,
        uv: Vec2::default(),
    };
    let clipped = clip_near([v(-2.0), v(0.0), v(0.5)]);
    assert_eq!(clipped.len(), 4);
    assert!(
        clipped
            .iter()
            .all(|v| v.clip_position.z >= -v.clip_position.w - 1e-12)
    );
    assert!(clip_near([v(-2.0), v(-3.0), v(-4.0)]).is_empty());
}
#[test]
fn shaded_triangles_keep_winding_depth_and_rgb() {
    let v = |x, y, z| RasterVertex {
        x,
        y,
        z,
        inverse_w: 1.0,
        world_position: Vec3::ZERO,
        normal: Vec3::Z,
        uv: Vec2::default(),
    };
    let tri = [v(1.0, 1.0, 0.2), v(7.0, 1.0, 0.2), v(1.0, 7.0, 0.2)];
    let mut a = Framebuffer::new(9, 9, 0).unwrap();
    triangle_shaded(&mut a, tri, &mut |_| Vec3::X);
    assert_eq!(a.pixels[2 * 9 + 2], 0xff0000);
    assert_eq!(a.pixels[8 * 9 + 8], 0);
    let expected = a.pixels.clone();
    a.clear();
    triangle_shaded(&mut a, [tri[2], tri[1], tri[0]], &mut |_| Vec3::X);
    assert_eq!(a.pixels, expected);
    let back = tri.map(|v| RasterVertex { z: 0.8, ..v });
    let (evaluated, written) = triangle_shaded(&mut a, back, &mut |_| Vec3::Y);
    assert_eq!((evaluated, written), (0, 0));
    assert_eq!(a.pixels, expected);
    a.clear();
    triangle_shaded(&mut a, back, &mut |_| Vec3::Z);
    assert_eq!(a.pixels[2 * 9 + 2], 0x0000ff);
    assert_eq!(to_rgb8(Vec3::new(f64::NAN, -1.0, 2.0)), 0x0000ff);
}
#[test]
fn invalid_cli_options_fail_without_creating_images() {
    use std::process::Command;
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/invalid-guard.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    for args in [
        vec!["--width", "0"],
        vec!["--height", "0"],
        vec!["--angle", "NaN"],
        vec!["--frames", "0"],
        vec!["--unknown"],
        vec!["--width"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
            .args(["--headless", "--output"])
            .arg(&output)
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("Error:"));
        assert!(!output.exists());
    }
}
