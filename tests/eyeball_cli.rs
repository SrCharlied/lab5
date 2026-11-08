use std::{io::BufReader, path::Path, process::Command};
#[test]
fn eyeball_cli_exports_the_exact_framebuffer_for_layer_angle_and_size() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/eyeball-cli.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--eyeball",
            "--headless",
            "--layers",
            "3",
            "--angle",
            "0.4",
            "--width",
            "160",
            "--height",
            "120",
            "--output",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "eye CLI failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(output).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    assert_eq!((info.width, info.height), (160, 120));
    assert_eq!(info.color_type, png::ColorType::Rgb);
    let (fb, _) = lab5::eyeball::render(160, 120, 0.4, 3).unwrap();
    let rgb: Vec<u8> = fb
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert!(
        rgb == bytes[..info.buffer_size()],
        "CLI PNG must equal selected software framebuffer"
    );
}
#[test]
fn eyeball_rejects_conflicting_modes_and_invalid_controls_before_writing() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/invalid-eyeball.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    for (args, message) in [
        (vec!["--eyeball", "--rocky"], "Escoge un solo modo"),
        (vec!["--eyeball", "--neutron"], "Escoge un solo modo"),
        (vec!["--eyeball", "--neutron-live"], "Escoge un solo modo"),
        (
            vec!["--eyeball", "--time", "1"],
            "--time requiere --neutron-live",
        ),
        (
            vec!["--eyeball", "--layers", "0"],
            "Capas debe estar entre 1 y 4",
        ),
        (
            vec!["--eyeball", "--layers", "5"],
            "Capas debe estar entre 1 y 4",
        ),
        (
            vec!["--eyeball", "--angle", "NaN"],
            "El angulo debe ser finito",
        ),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
            .args(["--headless", "--output"])
            .arg(&output)
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(!output.exists());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(message),
            "must reject for intended reason, not an unknown mode: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
#[test]
fn eyeball_has_its_own_default_output_without_replacing_previous_candidates() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/eyeball-default");
    std::fs::create_dir_all(&dir).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .current_dir(&dir)
        .args(["--eyeball", "--headless", "--width", "48", "--height", "48"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(dir.join("renders/eyeball-v1.png").is_file());
    assert!(!dir.join("renders/rocky-v1.png").exists());
    assert!(!dir.join("renders/neutron-v1.png").exists());
}
