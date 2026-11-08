//! Artistic blue gas giant inspired by HD 189733 b, not an observed weather map.
pub fn render(
    w: usize,
    h: usize,
    angle: f64,
    layers: u8,
) -> Result<
    (
        crate::renderer::framebuffer::Framebuffer,
        crate::renderer::stats::RenderStats,
    ),
    String,
> {
    use crate::{
        assets::primitives::uv_sphere,
        math::Transform,
        renderer::{framebuffer::Framebuffer, pipeline::draw_model},
        scene::camera::Camera,
    };
    if !(1..=4).contains(&layers) {
        return Err("Capas debe estar entre 1 y 4".into());
    }
    if !angle.is_finite() {
        return Err("El angulo debe ser finito".into());
    }
    let mut fb = Framebuffer::new(w, h, crate::BACKGROUND)?;
    let sphere = uv_sphere(64, 128);
    let mut camera = Camera::new(45_f64.to_radians(), w as f64 / h as f64, 0.1, 100.0)?;
    camera.position = Vec3::new(0.0, 0.0, 3.5 / camera.aspect.min(1.0));
    camera.target = Vec3::ZERO;
    let transform = Transform {
        rotation: Vec3::new(0.0, angle, 0.0),
        ..Transform::IDENTITY
    };
    let stats = draw_model(&mut fb, &sphere, &transform, &camera, true, |f| {
        shade(f, camera.position, layers)
    });
    Ok((fb, stats))
}
use crate::{
    math::Vec3,
    procedural::{direction, field, smoothstep},
    renderer::shader::{Color, FragmentInput},
};
pub fn shade(f: &FragmentInput, camera: Vec3, layers: u8) -> Color {
    let p = direction(f.uv);
    let mut latitude = p.y;
    let shear = field(Vec3::new(p.x * 5.0, p.y * 8.0, p.z * 5.0) + Vec3::new(8.2, 3.4, 6.1));
    if layers >= 2 {
        latitude += (shear - 0.5) * 0.30 * (1.0 - p.y * p.y);
    }
    let band = 0.5 + 0.5 * (23.0 * latitude + 0.7 * (latitude * 9.0).sin()).sin();
    let mut color = Vec3::new(0.003, 0.012, 0.065)
        .lerp(Vec3::new(0.025, 0.24, 0.53), smoothstep(0.12, 0.90, band));
    if layers >= 2 {
        let fine = field(Vec3::new(p.x * 13.0, p.y * 24.0, p.z * 13.0) + Vec3::new(2.8, 13.7, 9.1));
        let jet = (0.5 + 0.5 * (67.0 * latitude + 7.0 * fine).sin()).powi(10);
        color = color.lerp(Vec3::new(0.045, 0.36, 0.57), jet * 0.40);
        color = color * (0.78 + 0.40 * fine);
    }
    if layers >= 3 {
        let axis = Vec3::new(-0.30, -0.22, 0.93).normalized();
        let t = Vec3::Y.cross(axis).normalized();
        let b = axis.cross(t);
        let x = p.dot(t) / 0.42;
        let y = p.dot(b) / 0.20;
        let radius = (x * x + y * y).sqrt();
        let mask = (1.0 - smoothstep(0.82, 1.08, radius)) * smoothstep(0.84, 0.92, p.dot(axis));
        let phi = y.atan2(x);
        let curl = 0.5
            + 0.5 * (2.0 * phi - 12.0 * radius + 3.0 * field(p * 19.0 + Vec3::splat(8.7))).sin();
        let structure = smoothstep(0.06, 0.23, radius);
        let spiral = Vec3::new(0.006, 0.025, 0.12).lerp(Vec3::new(0.11, 0.50, 0.68), curl.powi(3));
        let eye = Vec3::new(0.004, 0.012, 0.045);
        let storm = eye.lerp(spiral, structure);
        color = color.lerp(storm, mask);
    }
    if layers >= 4 {
        let haze = smoothstep(
            0.53,
            0.76,
            field(Vec3::new(p.x * 6.0, p.y * 18.0, p.z * 6.0) + Vec3::splat(19.4)),
        );
        color = color.lerp(Vec3::new(0.22, 0.40, 0.65), haze * 0.24);
        let facing = f
            .normal
            .normalized()
            .dot((camera - f.world_position).normalized())
            .max(0.0);
        color += Vec3::new(0.01, 0.06, 0.18) * (1.0 - facing).powi(4) * 0.4;
    }
    let diffuse = f
        .normal
        .normalized()
        .dot(Vec3::new(-0.65, 0.8, 1.0).normalized())
        .max(0.0);
    color * (0.075 + 0.925 * diffuse)
}
