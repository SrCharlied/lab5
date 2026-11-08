use std::{io::BufReader, process::Command};

// Guards for validation added with the preview route; born green.
#[test]
fn invalid_preview_controls_do_not_write_a_png() {
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/invalid-neutron.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    for args in [
        vec!["--neutron", "--layers", "0"],
        vec!["--neutron", "--layers", "5"],
        vec!["--layers", "2"],
        vec!["--neutron", "--angle", "NaN"],
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
fn neutron_cli_exports_the_selected_shader_not_the_diagnostic() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = root.join("renders/tmp/neutron-cli.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--neutron",
            "--layers",
            "4",
            "--headless",
            "--width",
            "192",
            "--height",
            "128",
            "--output",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "neutron preview route must run: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(output).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    let (expected, _) = lab5::render_neutron(192, 128, 0.0, 4).unwrap();
    let rgb: Vec<u8> = expected
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert!(
        bytes[..info.buffer_size()] == rgb,
        "export must use the exact selected software buffer"
    );
}
