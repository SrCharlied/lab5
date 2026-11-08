use std::{io::BufReader, path::Path, process::Command};
#[test]
fn pulsar_cli_exports_the_actual_exhibition_frame_at_the_requested_time() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = root.join("renders/tmp/pulsar-cli.png");
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--pulsar",
            "--headless",
            "--time",
            "1.8",
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
        "pulsar CLI missing: {}",
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
    let mut state = lab5::neutron_live::LiveState::new(0.4).unwrap();
    state.advance(1.8, Default::default()).unwrap();
    let mut viewer = lab5::pulsar::PulsarRenderer::new(160, 120, 4).unwrap();
    viewer.render(&state).unwrap();
    let rgb: Vec<_> = viewer
        .framebuffer()
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert!(rgb == bytes[..info.buffer_size()]);
}
#[test]
fn pulsar_conflicts_and_invalid_inputs_fail_for_the_right_reason_without_writing() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/invalid-pulsar.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    for (args, message) in [
        (vec!["--pulsar", "--neutron"], "Escoge un solo modo"),
        (vec!["--pulsar", "--neutron-live"], "Escoge un solo modo"),
        (vec!["--pulsar", "--rocky"], "Escoge un solo modo"),
        (vec!["--pulsar", "--eyeball"], "Escoge un solo modo"),
        (
            vec!["--pulsar", "--layers", "0"],
            "Capas debe estar entre 1 y 4",
        ),
        (vec!["--pulsar", "--time", "NaN"], "invalido"),
        (vec!["--pulsar", "--time", "-1"], "invalido"),
        (vec!["--pulsar", "--angle", "NaN"], "angulo debe ser finito"),
        (vec!["--pulsar", "--width", "0"], "Dimensiones"),
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
            "wrong failure reason: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
#[test]
fn pulsar_default_capture_does_not_replace_approved_outputs() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/pulsar-default");
    std::fs::create_dir_all(&dir).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .current_dir(&dir)
        .args(["--pulsar", "--headless", "--width", "64", "--height", "48"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(dir.join("renders/pulsar-v1.png").exists());
    for name in ["neutron-v1.png", "neutron-live.png", "eyeball-v1.png"] {
        assert!(!dir.join("renders").join(name).exists());
    }
}
