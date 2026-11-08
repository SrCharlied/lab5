use std::{io::BufReader, path::Path, process::Command};
#[test]
fn v2_cli_exports_the_selected_real_framebuffer() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/gas-v2-cli.png");
    let r = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--gas-v2",
            "--headless",
            "--angle",
            "0.4",
            "--layers",
            "3",
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
        r.status.success(),
        "V2 CLI missing: {}",
        String::from_utf8_lossy(&r.stderr)
    );
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(output).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    assert_eq!((info.width, info.height), (160, 120));
    assert_eq!(info.color_type, png::ColorType::Rgb);
    let (fb, _) = lab5::gas_v2::render(160, 120, 0.4, 3).unwrap();
    let rgb: Vec<_> = fb
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert!(rgb == bytes[..info.buffer_size()]);
}
#[test]
fn v2_cli_rejects_conflicts_and_invalid_arguments_before_writing() {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/invalid-gas-v2.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    for args in [
        vec!["--gas-v2", "--gas"],
        vec!["--gas-v2", "--neutron"],
        vec!["--gas-v2", "--eyeball"],
        vec!["--gas-v2", "--pulsar"],
        vec!["--gas-v2", "--rocky"],
        vec!["--gas-v2", "--neutron-live"],
        vec!["--gas-v2", "--layers", "0"],
        vec!["--gas-v2", "--angle", "NaN"],
        vec!["--gas-v2", "--time", "1"],
    ] {
        let r = Command::new(env!("CARGO_BIN_EXE_lab5"))
            .args(["--headless", "--output"])
            .arg(&output)
            .args(args)
            .output()
            .unwrap();
        assert!(!r.status.success());
        assert!(!output.exists());
        let error = String::from_utf8_lossy(&r.stderr);
        assert!(
            !error.contains("Opcion desconocida"),
            "missing mode is not valid rejection evidence: {error}"
        );
    }
}
#[test]
fn v2_default_capture_keeps_the_banded_version_separate() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/gas-v2-default");
    std::fs::create_dir_all(&dir).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .current_dir(&dir)
        .args(["--gas-v2", "--headless", "--width", "64", "--height", "48"])
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(dir.join("renders/gas-v2.png").exists());
    assert!(!dir.join("renders/gas-v1.png").exists());
}
