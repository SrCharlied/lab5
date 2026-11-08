use std::{io::BufReader, path::Path, process::Command};
#[test]
fn gas_cli_exports_the_exact_software_framebuffer() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/gas-cli.png");
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--gas",
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
        "gas CLI missing: {}",
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
    let (fb, _) = lab5::gas::render(160, 120, 0.4, 3).unwrap();
    let rgb: Vec<_> = fb
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert!(rgb == bytes[..info.buffer_size()]);
}
#[test]
fn gas_rejects_conflicts_and_invalid_controls_for_the_intended_reason() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/invalid-gas.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    for (args, message) in [
        (vec!["--gas", "--neutron"], "Escoge un solo modo"),
        (vec!["--gas", "--neutron-live"], "Escoge un solo modo"),
        (vec!["--gas", "--rocky"], "Escoge un solo modo"),
        (vec!["--gas", "--eyeball"], "Escoge un solo modo"),
        (vec!["--gas", "--pulsar"], "Escoge un solo modo"),
        (
            vec!["--gas", "--time", "1"],
            "--time requiere --neutron-live",
        ),
        (
            vec!["--gas", "--layers", "0"],
            "Capas debe estar entre 1 y 4",
        ),
        (
            vec!["--gas", "--layers", "5"],
            "Capas debe estar entre 1 y 4",
        ),
        (vec!["--gas", "--angle", "NaN"], "El angulo debe ser finito"),
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
            "wrong rejection: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
#[test]
fn gas_default_output_is_separate_from_accepted_bodies() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/gas-default");
    std::fs::create_dir_all(&dir).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .current_dir(&dir)
        .args(["--gas", "--headless", "--width", "64", "--height", "48"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(dir.join("renders/gas-v1.png").exists());
    for name in ["eyeball-v1.png", "neutron-v1.png", "pulsar-v1.png"] {
        assert!(!dir.join("renders").join(name).exists());
    }
}
