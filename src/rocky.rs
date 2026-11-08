//! LHS 1140 b inspired artistic rocky world; not an observed surface reconstruction.
use crate::{
    math::Vec3,
    procedural::{direction, field, noise, smoothstep},
    renderer::shader::{Color, FragmentInput},
};
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
fn terrain(p: Vec3) -> (f64, f64) {
    let warp = Vec3::new(
        field(p * 1.7 + Vec3::new(4.2, 1.3, 8.1)),
        field(p * 1.7 + Vec3::new(9.4, 6.7, 2.1)),
        field(p * 1.7 + Vec3::new(2.3, 8.2, 7.4)),
    ) - Vec3::splat(0.5);
    let q = p + warp * 0.45;
    let height = 0.78 * field(q * 2.4 + Vec3::new(7.1, 2.8, 4.6))
        + 0.22 * field(q * 5.8 + Vec3::new(16.3, 8.2, 3.1));
    (height, field(p * 21.0 + Vec3::new(3.4, 7.3, 1.8)))
}
pub fn shade(f: &FragmentInput, _camera: Vec3, layers: u8) -> Color {
    let p = direction(f.uv);
    let (height, detail) = terrain(p);
    let ridges = (1.0 - (2.0 * noise(p * 13.0 + Vec3::splat(5.2)) - 1.0).abs()).powi(3);
    let rock = Vec3::new(0.025, 0.020, 0.016)
        .lerp(Vec3::new(0.28, 0.10, 0.035), smoothstep(0.32, 0.65, height));
    let mut color = rock.lerp(
        Vec3::new(0.52, 0.40, 0.25),
        ridges * smoothstep(0.52, 0.68, height) * 0.45,
    ) * (0.72 + 0.45 * detail);
    if layers >= 2 {
        let land = smoothstep(0.47 - 0.012, 0.47 + 0.012, height);
        let shallows = smoothstep(0.32, 0.47, height);
        let sea = Vec3::new(0.004, 0.018, 0.026).lerp(Vec3::new(0.018, 0.17, 0.15), shallows);
        color = sea.lerp(color, land);
    }
    if layers >= 3 {
        let latitude = p.y.abs() + (noise(p * 8.0 + Vec3::splat(12.1)) - 0.5) * 0.12;
        let ice = smoothstep(0.79, 0.87, latitude);
        let fissure = (1.0 - (2.0 * noise(p * 31.0 + Vec3::splat(8.3)) - 1.0).abs()).powi(18);
        let frozen = Vec3::new(0.70, 0.80, 0.78).lerp(Vec3::new(0.06, 0.20, 0.25), fissure * 0.28)
            * (0.86 + 0.14 * detail);
        color = color.lerp(frozen, ice);
    }
    if layers >= 4 {
        let shear = Vec3::new(p.x * 7.0 + (p.y * 7.0).sin() * 1.1, p.y * 12.0, p.z * 7.0);
        let cloud = smoothstep(0.57, 0.76, field(shear + Vec3::new(19.2, 7.4, 3.5)));
        let wisps = 0.45 + 0.55 * field(shear * 2.8 + Vec3::splat(12.7));
        color = color.lerp(Vec3::new(0.78, 0.82, 0.80), cloud * wisps * 0.42);
    }
    let diffuse = f
        .normal
        .normalized()
        .dot(Vec3::new(-0.65, 0.80, 1.0).normalized())
        .max(0.0);
    color * (0.08 + 0.92 * diffuse)
}
