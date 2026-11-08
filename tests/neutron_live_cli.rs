use std::{io::BufReader, process::Command};
#[test]
fn invalid_live_options_do_not_write_an_image() {
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/invalid-live.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    for args in [
        vec!["--neutron-live", "--time", "NaN"],
        vec!["--neutron-live", "--time", "-1"],
        vec!["--neutron-live", "--neutron"],
        vec!["--diagnostic", "--time", "1"],
        vec!["--neutron-live", "--layers", "0"],
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
fn live_cli_exports_the_requested_time_from_the_real_viewer() {
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/live-cli.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
            "--neutron-live",
            "--time",
            "1.8",
            "--angle",
            "-0.65",
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
        "live route must execute: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(output).unwrap()))
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    reader.finish().unwrap();
    let mut state = lab5::neutron_live::LiveState::new(-0.65).unwrap();
    state
        .advance(1.8, lab5::neutron_live::Controls::default())
        .unwrap();
    let mut viewer = lab5::neutron_live::LiveRenderer::new(160, 120, 4).unwrap();
    viewer.render(&state).unwrap();
    let rgb: Vec<u8> = viewer
        .framebuffer()
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert!(
        rgb == bytes[..info.buffer_size()],
        "CLI must export the same buffer used by the live viewer"
    );
}
