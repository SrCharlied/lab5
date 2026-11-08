//! Visible pulse response is measured AFTER the actual sRGB8 encoder, at a fixed pose.
use lab5::{
    assets::primitives::uv_sphere,
    math::{Transform, Vec3},
    neutron,
    neutron_live::pulse_at,
    renderer::{framebuffer::Framebuffer, image::save_png, pipeline::draw_model},
    scene::camera::Camera,
};
fn fixed_pose(gain: f64) -> Framebuffer {
    let mut fb = Framebuffer::new(640, 480, lab5::BACKGROUND).unwrap();
    let sphere = uv_sphere(64, 128);
    let mut camera = Camera::new(45_f64.to_radians(), 640.0 / 480.0, 0.1, 100.0).unwrap();
    camera.position = Vec3::new(0.0, 0.0, 3.5);
    camera.target = Vec3::ZERO;
    let transform = Transform {
        rotation: Vec3::new(0.0, -0.65, 0.0),
        ..Transform::IDENTITY
    };
    draw_model(&mut fb, &sphere, &transform, &camera, true, |f| {
        neutron::shade_pulsed(f, camera.position, 4, gain)
    });
    fb
}
#[test]
fn pulse_has_clear_display_contrast_at_a_fixed_pose() {
    let peak = fixed_pose(pulse_at(0.0).unwrap());
    let low = fixed_pose(pulse_at(1.8).unwrap());
    let red = |p: u32| ((p >> 16) & 255) as f64;
    let white = |p: u32| ((p >> 16) & 255) > 240 && ((p >> 8) & 255) > 240 && (p & 255) > 240;
    let roi: Vec<_> = peak
        .pixels
        .iter()
        .zip(&low.pixels)
        .filter(|(a, _)| white(**a))
        .collect();
    assert!(roi.len() > 100, "probe must include a real hot region");
    let peak_mean = roi.iter().map(|(a, _)| red(**a)).sum::<f64>() / roi.len() as f64;
    let low_mean = roi.iter().map(|(_, b)| red(**b)).sum::<f64>() / roi.len() as f64;
    let saturated_at_low = roi.iter().filter(|(_, b)| white(**b)).count();
    let remaining_white = saturated_at_low as f64 / roi.len() as f64;
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("renders/tmp/pulse-visibility");
    save_png(&root.join("peak.png"), &peak).unwrap();
    save_png(&root.join("minimum.png"), &low).unwrap();
    println!(
        "PULSE_VISIBILITY gain_min={} hot_pixels={} mean_red_peak={peak_mean:.3} mean_red_min={low_mean:.3} red_drop={:.3} remaining_white_ratio={remaining_white:.5}",
        pulse_at(1.8).unwrap(),
        roi.len(),
        peak_mean - low_mean
    );
    assert!(
        peak_mean - low_mean >= 50.0,
        "pulse must survive clipping and sRGB encoding with a clear display-space change"
    );
    assert!(
        remaining_white <= 0.35,
        "the white core must visibly contract instead of staying saturated"
    );
    assert!(
        low.pixels
            .iter()
            .zip(&peak.pixels)
            .filter(|(_, p)| **p != lab5::BACKGROUND)
            .all(|(p, _)| *p != lab5::BACKGROUND),
        "do not extinguish the sphere"
    );
}
