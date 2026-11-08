use std::{io::BufReader, path::Path, process::Command};
fn export(name: &str, args: &[&str]) -> Vec<u8> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("renders/tmp/primary-{name}.png"));
    let r = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--headless",
            "--width",
            "160",
            "--height",
            "120",
            "--output",
        ])
        .arg(&path)
        .args(args)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(path).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    bytes.truncate(info.buffer_size());
    bytes
}
#[test]
fn primary_without_body_flag_is_exactly_the_existing_pulsar() {
    assert_eq!(export("default", &[]), export("explicit", &["--pulsar"]));
}
#[test]
fn primary_preserves_time_angle_and_layers_of_the_pulsar() {
    let args = ["--time", "1.8", "--angle", "0.4", "--layers", "3"];
    let mut explicit = vec!["--pulsar"];
    explicit.extend_from_slice(&args);
    assert_eq!(
        export("time-default", &args),
        export("time-explicit", &explicit)
    );
}
#[test]
fn explicit_diagnostic_preserves_old_framebuffer_and_rejects_mode_conflicts() {
    let rgb = export("diagnostic", &["--diagnostic"]);
    let (fb, _) = lab5::render_diagnostic(160, 120, 0.0).unwrap();
    let expected: Vec<_> = fb
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert_eq!(rgb, expected);
    for other in ["--pulsar", "--neutron", "--desert", "--gas-v2"] {
        let r = Command::new(env!("CARGO_BIN_EXE_lab5"))
            .args(["--diagnostic", other, "--headless"])
            .output()
            .unwrap();
        assert!(!r.status.success());
        assert!(String::from_utf8_lossy(&r.stderr).contains("Escoge un solo modo"));
    }
}
#[test]
fn primary_default_capture_keeps_reference_images_separate() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("renders/tmp/primary-default-capture")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&dir).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .current_dir(&dir)
        .args(["--headless", "--width", "64", "--height", "48"])
        .output()
        .unwrap();
    assert!(r.status.success());
    assert!(dir.join("renders/captures/pulsar-main.png").is_file());
    assert!(!dir.join("renders/diagnostic.png").exists());
    assert!(!dir.join("renders/pulsar-v1.png").exists());
}
