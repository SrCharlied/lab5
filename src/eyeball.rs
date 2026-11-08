//! Artistic ice world with one ocean-eye; no textures or physical climate claims.
use crate::{
    math::{Vec2, Vec3},
    procedural::{direction, field, noise, smoothstep},
    renderer::shader::{Color, FragmentInput},
};
struct Eye {
    rho: f64,
    phi: f64,
    radius: f64,
    mask: f64,
}
pub fn render(
    width: usize,
    height: usize,
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
    let mut fb = Framebuffer::new(width, height, crate::BACKGROUND)?;
    let sphere = uv_sphere(64, 128);
    let mut camera = Camera::new(
        45_f64.to_radians(),
        width as f64 / height as f64,
        0.1,
        100.0,
    )?;
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
fn eye(p: Vec3) -> Eye {
    let axis = Vec3::new(-0.37, 0.24, 0.90).normalized();
    let tangent = Vec3::Y.cross(axis).normalized();
    let bitangent = axis.cross(tangent);
    let rho = p.dot(axis).clamp(-1.0, 1.0).acos();
    let phi = p.dot(bitangent).atan2(p.dot(tangent));
    let warp = field(p * 7.0 + Vec3::splat(6.8));
    let radius = 0.49
        + 0.028 * (5.0 * phi + warp * 3.0).sin()
        + 0.016 * (2.0 * field(p * 13.0 + Vec3::splat(2.1)) - 1.0)
        + 0.009 * (13.0 * phi + warp * 4.0).sin();
    let mask = 1.0 - smoothstep(radius - 0.018, radius + 0.018, rho);
    Eye {
        rho,
        phi,
        radius,
        mask,
    }
}
/// A single perturbed spherical cap, never an absolute-dot antipodal pair.
pub fn ocean_mask(uv: Vec2) -> f64 {
    eye(direction(uv)).mask
}
pub fn shade(f: &FragmentInput, _camera: Vec3, layers: u8) -> Color {
    let p = direction(f.uv);
    let e = eye(p);
    let broad = field(p * 6.0 + Vec3::new(3.1, 7.4, 2.6));
    let grain = field(p * 43.0 + Vec3::splat(17.2));
    let mut color = Vec3::new(0.28, 0.25, 0.18).lerp(Vec3::new(0.63, 0.57, 0.43), broad)
        * (0.88 + 0.12 * grain);
    if layers >= 2 {
        let distance = e.rho - e.radius;
        let envelope =
            (1.0 - smoothstep(0.10, 0.62, distance)) * smoothstep(-0.035, 0.04, distance);
        let warp = field(p * 21.0 + Vec3::splat(9.2));
        let main =
            (0.5 + 0.5 * (28.0 * e.phi + 2.0 * (e.rho * 9.0).sin() + warp * 4.0).cos()).powi(22);
        let branch = (0.5 + 0.5 * (47.0 * e.phi + 19.0 * e.rho + warp * 5.0).cos()).powi(35);
        let fine = (1.0 - (2.0 * noise(p * 29.0 + Vec3::splat(4.9)) - 1.0).abs()).powi(24);
        let crack = ((main * 0.68 + branch * 0.30) * envelope + fine * 0.10).clamp(0.0, 0.85);
        color = color.lerp(Vec3::new(0.025, 0.05, 0.065), crack);
        let margin = 1.0 - smoothstep(0.02, 0.095, distance.abs());
        color = color.lerp(Vec3::new(0.09, 0.17, 0.19), margin * 0.24);
    }
    if layers >= 3 {
        let r = e.rho / e.radius;
        let water_detail = field(p * 18.0 + Vec3::new(4.3, 9.1, 2.8));
        let mut sea =
            Vec3::new(0.001, 0.008, 0.025).lerp(Vec3::new(0.006, 0.055, 0.12), water_detail);
        sea = sea.lerp(
            Vec3::new(0.025, 0.15, 0.22),
            smoothstep(0.78, 1.0, r) * 0.65,
        );
        let stream = (0.5 + 0.5 * (3.0 * e.phi + 14.0 * (r + 0.12).ln() + grain * 4.0).sin())
            .powi(7)
            * smoothstep(0.02, 0.15, r);
        sea += Vec3::new(0.002, 0.009, 0.016) * stream;
        color = color.lerp(sea, e.mask);
    }
    if layers >= 4 {
        let r = e.rho / e.radius;
        let phase = 3.0 * e.phi
            + 12.0 * (r + 0.12).ln()
            + 4.8 * (field(p * 20.0 + Vec3::splat(8.4)) - 0.5)
            + 2.0 * (field(p * 85.0 + Vec3::splat(2.7)) - 0.5);
        let arms = smoothstep(0.52, 0.96, 0.5 + 0.5 * phase.cos());
        let envelope = 1.0 - smoothstep(0.40, 0.95, r);
        let wisps = smoothstep(0.26, 0.72, field(p * 65.0 + Vec3::new(7.1, 3.4, 9.2)));
        let core = 1.0 - smoothstep(0.06, 0.28, r);
        let cloud = (core + (1.0 - core) * arms * envelope * wisps * 0.40) * e.mask;
        let frost = 0.86 + 0.14 * field(p * 85.0 + Vec3::splat(9.8));
        color = color.lerp(Vec3::new(0.82, 0.86, 0.83) * frost, cloud);
    }
    let diffuse = f
        .normal
        .normalized()
        .dot(Vec3::new(-0.75, 0.60, 0.75).normalized())
        .max(0.0);
    color * (0.035 + 0.965 * diffuse)
}
