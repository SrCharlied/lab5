use std::{io::BufReader, process::Command};
#[test]
fn rocky_rejects_conflicting_modes_invalid_layers_and_time_without_writing() {
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/invalid-rocky.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    for args in [
        vec!["--rocky", "--neutron"],
        vec!["--rocky", "--neutron-live"],
        vec!["--rocky", "--layers", "0"],
        vec!["--rocky", "--time", "1"],
        vec!["--rocky", "--angle", "NaN"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
            .args(["--headless", "--output"])
            .arg(&output)
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(!output.exists());
    }
}

#[test]
fn rocky_cli_exports_the_actual_selected_shader() {
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/rocky-cli.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--rocky",
            "--headless",
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
        "rocky route must execute: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(output).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    let (fb, _) = lab5::rocky::render(160, 120, 0.0, 4).unwrap();
    let rgb: Vec<u8> = fb
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert!(
        rgb == bytes[..info.buffer_size()],
        "PNG must match the actual software framebuffer"
    );
}
