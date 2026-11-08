use std::{
    io::BufReader,
    path::{Path, PathBuf},
    process::Command,
};
fn directory() -> PathBuf {
    let d = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("renders/tmp/slideshow-cli")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&d).unwrap();
    d
}
#[test]
fn all_seven_slides_export_the_exact_selected_framebuffer() {
    let dir = directory();
    for index in 0..7 {
        let out = dir.join(format!("{index}.png"));
        let r = Command::new(env!("CARGO_BIN_EXE_lab5"))
            .args(["--slideshow", "--slide"])
            .arg((index + 1).to_string())
            .args([
                "--headless",
                "--width",
                "160",
                "--height",
                "120",
                "--angle",
                "0.4",
                "--output",
            ])
            .arg(&out)
            .output()
            .unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        assert!(String::from_utf8_lossy(&r.stdout).contains(&format!("SLIDESHOW {}/7", index + 1)));
        let p = lab5::slideshow::Presentation::new(160, 120, 0.4, 0.0, 4, index).unwrap();
        let expected: Vec<_> = p
            .frame()
            .pixels
            .iter()
            .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
            .collect();
        let mut reader = png::Decoder::new(BufReader::new(std::fs::File::open(out).unwrap()))
            .read_info()
            .unwrap();
        let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut bytes).unwrap();
        reader.finish().unwrap();
        assert_eq!((info.width, info.height), (160, 120));
        assert!(
            expected == bytes[..info.buffer_size()],
            "slide {index} PNG differs"
        );
    }
}
#[test]
fn slideshow_invalid_inputs_leave_sentinel_untouched() {
    let dir = directory();
    for (i, args) in [
        vec!["--slide", "0"],
        vec!["--slide", "8"],
        vec!["--slideshow", "--pulsar"],
        vec!["--desert", "--slide", "2"],
        vec!["--slideshow", "--time", "NaN"],
        vec!["--slideshow", "--width", "0"],
        vec!["--slideshow", "--layers", "0"],
        vec!["--slideshow", "--moon"],
    ]
    .into_iter()
    .enumerate()
    {
        let out = dir.join(format!("invalid-{i}.png"));
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
            "mode must reach real validation"
        );
    }
}
