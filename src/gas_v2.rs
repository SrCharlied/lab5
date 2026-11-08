//! Turbulent artistic variant; original gas shader stays unchanged.

fn mix_clouds(base: Color, cloud: Color) -> Color {
    base.lerp(cloud, 0.85)
}
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
fn advect(mut p: Vec3) -> Vec3 {
    let centers = [
        Vec3::new(-0.42, 0.34, 0.85),
        Vec3::new(0.46, -0.28, 0.86),
        Vec3::new(0.45, 0.56, -0.66),
        Vec3::new(-0.43, -0.44, -0.78),
        Vec3::new(0.95, 0.12, 0.25),
        Vec3::new(-0.94, -0.1, -0.32),
        Vec3::new(0.1, 0.97, 0.18),
        Vec3::new(-0.1, -0.97, 0.20),
    ];
    for (i, axis) in centers.into_iter().enumerate() {
        let axis = axis.normalized();
        let weight = p.dot(axis).max(0.0).powi(18);
        let angle = weight * if i % 2 == 0 { 3.8 } else { -3.4 };
        let (s, c) = angle.sin_cos();
        p = p * c + axis.cross(p) * s + axis * (axis.dot(p) * (1.0 - c));
    }
    let q = Vec3::new(p.x * 2.6, p.y * 3.8, p.z * 2.6);
    let w = Vec3::new(
        field(q + Vec3::splat(3.1)),
        field(q + Vec3::splat(11.7)),
        field(q + Vec3::splat(27.4)),
    ) - Vec3::splat(0.5);
    let r = q + w * 1.7;
    let w2 = Vec3::new(
        field(r + Vec3::splat(8.3)),
        field(r + Vec3::splat(19.1)),
        field(r + Vec3::splat(37.8)),
    ) - Vec3::splat(0.5);
    r + w2 * 1.2
}
pub fn shade(f: &FragmentInput, camera: Vec3, layers: u8) -> Color {
    let p = direction(f.uv);
    let mass = field(p * 3.2 + Vec3::new(3.7, 8.2, 11.9));
    let mut color = Vec3::new(0.003, 0.009, 0.035)
        .lerp(Vec3::new(0.032, 0.095, 0.24), smoothstep(0.20, 0.80, mass));
    if layers >= 2 {
        let q = advect(p);
        let cloud = field(q * 2.1 + Vec3::new(7.2, 4.1, 9.8));
        let turbulent = Vec3::new(0.004, 0.012, 0.042)
            .lerp(Vec3::new(0.10, 0.20, 0.38), smoothstep(0.24, 0.78, cloud));
        color = mix_clouds(color, turbulent);
        if layers >= 3 {
            let stream = field(q * 2.4 + Vec3::new(12.4, 7.8, 2.3));
            let crest = 1.0 - smoothstep(0.010, 0.075, (stream - 0.52).abs());
            let breakup = 0.35 + 0.65 * field(q * 8.5 + Vec3::splat(17.2));
            color = color.lerp(Vec3::new(0.65, 0.80, 1.0), crest * breakup * 0.80);
        }
        if layers >= 4 {
            let haze = 0.035 + 0.065 * field(q * 1.3 + Vec3::splat(24.7));
            color = color.lerp(Vec3::new(0.16, 0.24, 0.38), haze);
        }
    }
    if layers >= 4 {
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

#[cfg(test)]
mod layer_tests {
    #[test]
    fn coarse_base_remains_a_contribution_after_the_turbulent_layer() {
        let cloud = crate::math::Vec3::new(0.1, 0.2, 0.38);
        let a = super::mix_clouds(crate::math::Vec3::ZERO, cloud);
        let b = super::mix_clouds(crate::math::Vec3::splat(0.1), cloud);
        assert!(
            (a - b).length() > 0.01,
            "turbulent layer discards the coarse base"
        );
    }
}
