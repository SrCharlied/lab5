use std::{io::BufReader, process::Command};
#[test]
fn headless_cli_exports_the_actual_framebuffer_as_rgb_png() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = root.join("renders/tmp/cli-test.png");
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    let result = Command::new(env!("CARGO_BIN_EXE_lab5"))
        .args([
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
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        output.is_file(),
        "headless invocation must create the requested PNG"
    );
    let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(output).unwrap()))
        .read_info()
        .unwrap();
    let mut data = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut data).unwrap();
    assert_eq!((info.width, info.height), (160, 120));
    assert_eq!(info.color_type, png::ColorType::Rgb);
    let (fb, _) = lab5::render_diagnostic(160, 120, 0.0).unwrap();
    let expected: Vec<u8> = fb
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    assert_eq!(
        &data[..info.buffer_size()],
        expected.as_slice(),
        "PNG must match the software buffer exactly"
    );
}
