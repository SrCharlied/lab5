use lab5::{
    assets::primitives::uv_sphere,
    extras::{self, Body},
    math::{Transform, Vec3},
    renderer::{framebuffer::Framebuffer, pipeline::draw_model},
    scene::camera::Camera,
};
use std::{io::BufReader, path::Path, process::Command};
#[test]
fn volcanic_cli_exports_its_real_shader_not_the_diagnostic() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/extras-volcanic-cli.png");
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--volcanic",
            "--headless",
            "--width",
            "160",
            "--height",
            "120",
            "--angle",
            "0.4",
            "--output",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let (actual, stats) = extras::render(160, 120, 0.4, 4, Body::Volcanic, false).unwrap();
    let mut expected = Framebuffer::new(160, 120, lab5::BACKGROUND).unwrap();
    let mut camera = Camera::new(45_f64.to_radians(), 160.0 / 120.0, 0.1, 100.0).unwrap();
    camera.position = Vec3::new(0.0, 0.0, 3.5);
    camera.target = Vec3::ZERO;
    let tr = Transform {
        rotation: Vec3::new(0.0, 0.4, 0.0),
        ..Transform::IDENTITY
    };
    let reference = draw_model(
        &mut expected,
        &uv_sphere(64, 128),
        &tr,
        &camera,
        true,
        |f| extras::shade(Body::Volcanic, f, 4),
    );
    assert_eq!(stats, reference);
    assert!(
        actual.pixels == expected.pixels,
        "render must execute selected shader"
    );
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(output).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    assert_eq!((info.width, info.height), (160, 120));
    let rgb: Vec<_> = actual
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert_eq!(rgb, bytes[..info.buffer_size()]);
}
fn export_bonus(flag: &str, body: Body, moon: bool) {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "renders/tmp/extras-{}-{moon}.png",
        flag.trim_start_matches('-')
    ));
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_lab5"));
    cmd.args([
        flag,
        "--headless",
        "--width",
        "160",
        "--height",
        "120",
        "--angle",
        "0.4",
        "--output",
    ])
    .arg(&output);
    if moon {
        cmd.arg("--moon");
    }
    let r = cmd.output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let (fb, stats) = extras::render(160, 120, 0.4, 4, body, moon).unwrap();
    assert_eq!(stats.objects_submitted, if moon { 2 } else { 1 });
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(output).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    let rgb: Vec<_> = fb
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert_eq!(rgb, bytes[..info.buffer_size()]);
}
#[test]
fn desert_cli_exports_selected_framebuffer() {
    export_bonus("--desert", Body::Desert, false);
}
#[test]
fn temperate_cli_exports_selected_framebuffer() {
    export_bonus("--temperate", Body::Temperate, false);
}
#[test]
fn desert_moon_cli_exports_both_models() {
    export_bonus("--desert", Body::Desert, true);
}
#[test]
fn bonus_conflicts_and_invalid_inputs_reject_before_writing() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/extras-invalid");
    std::fs::create_dir_all(&dir).unwrap();
    for (i, args) in [
        vec!["--volcanic", "--desert"],
        vec!["--temperate", "--gas-v2"],
        vec!["--desert", "--layers", "0"],
        vec!["--volcanic", "--time", "1"],
        vec!["--moon"],
        vec!["--volcanic", "--moon"],
        vec!["--temperate", "--moon"],
        vec!["--desert", "--angle", "NaN"],
        vec!["--desert", "--width", "0"],
    ]
    .into_iter()
    .enumerate()
    {
        let out = dir.join(format!("{i}.png"));
        std::fs::write(&out, b"sentinel").unwrap();
        let r = Command::new(env!("CARGO_BIN_EXE_lab5"))
            .args(["--headless", "--output"])
            .arg(&out)
            .args(args)
            .output()
            .unwrap();
        assert!(!r.status.success());
        assert_eq!(std::fs::read(out).unwrap(), b"sentinel");
        assert!(
            !String::from_utf8_lossy(&r.stderr).contains("Opcion desconocida"),
            "known option must reach real validation"
        );
    }
}
