//! Artistic neutron-star candidate, not an astrophysical simulation.
use crate::{
    math::{Vec2, Vec3},
    renderer::shader::{Color, FragmentInput},
};
use std::f64::consts::{PI, TAU};

/// Static candidate using the accepted sphere/camera and the real CPU pipeline.
/// Layers are cumulative (1..=4); no external assets or postprocessing.
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
    let mut framebuffer = Framebuffer::new(width, height, crate::BACKGROUND)?;
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
    let stats = draw_model(&mut framebuffer, &sphere, &transform, &camera, true, |f| {
        shade(f, camera.position, layers)
    });
    Ok((framebuffer, stats))
}

// Direction fixed to the procedural sphere's local frame, not the world.
fn direction(uv: Vec2) -> Vec3 {
    let (st, ct) = (uv.y * PI).sin_cos();
    let (sp, cp) = (uv.x * TAU).sin_cos();
    Vec3::new(st * cp, ct, st * sp)
}
fn hash(x: u32, y: u32, z: u32) -> f64 {
    let mut h =
        x.wrapping_mul(0x8da6b343) ^ y.wrapping_mul(0xd8163841) ^ z.wrapping_mul(0xcb1ab31f);
    h = (h ^ (h >> 16)).wrapping_mul(0x7feb352d);
    h = (h ^ (h >> 15)).wrapping_mul(0x846ca68b);
    h ^= h >> 16;
    (h >> 8) as f64 / 16_777_215.0
}
fn noise(p: Vec3) -> f64 {
    let cell = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
    let fade = |t: f64| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let f = [
        fade(p.x - p.x.floor()),
        fade(p.y - p.y.floor()),
        fade(p.z - p.z.floor()),
    ];
    let mut sum = 0.0;
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                let weight =
                    |axis: usize, index: i32| if index == 0 { 1.0 - f[axis] } else { f[axis] };
                sum += hash(
                    cell[0].wrapping_add(x) as u32,
                    cell[1].wrapping_add(y) as u32,
                    cell[2].wrapping_add(z) as u32,
                ) * weight(0, x)
                    * weight(1, y)
                    * weight(2, z);
            }
        }
    }
    sum
}
fn field(p: Vec3) -> f64 {
    (noise(p) * 0.57
        + noise(p * 2.03 + Vec3::splat(13.7)) * 0.29
        + noise(p * 4.11 + Vec3::splat(31.1)) * 0.14)
        .clamp(0.0, 1.0)
}
/// Inputs must be finite fragment attributes from the unit UV sphere and 1..=4 layers.
/// Returns linear emission color; this is art direction, not a thermal spectrum.
pub fn shade(fragment: &FragmentInput, camera: Vec3, layers: u8) -> Color {
    let p = direction(fragment.uv);
    let broad = field(p * 2.4 + Vec3::new(4.1, 9.7, 2.3));
    let view = (camera - fragment.world_position).normalized();
    let facing = fragment.normal.normalized().dot(view).max(0.0);
    // 1. Coarse, local-space surface variation and artistic limb darkening.
    let mut color = Vec3::new(0.001, 0.005, 0.025).lerp(Vec3::new(0.018, 0.080, 0.24), broad)
        * (0.48 + 0.52 * facing.sqrt());
    // 2. Antipodal warm-white cores with cyan skirts and fine heat variation.
    if layers >= 2 {
        let axis = Vec3::new(0.48, 0.63, 0.61).normalized();
        let pole = p.dot(axis).abs().clamp(0.0, 1.0);
        color += Vec3::new(0.015, 0.40, 1.0) * pole.powi(8) * 0.85;
        let granulation = 0.64 + 0.72 * noise(p * 45.0 + Vec3::splat(19.2));
        color += Vec3::new(1.0, 0.98, 0.96) * pole.powi(32) * 1.5 * granulation;
    }
    // 3. Broken curved strands in magnetic coordinates, suppressed at the poles.
    // Integer longitude frequency is periodic; the polar weight removes its singularity.
    if layers >= 3 {
        let axis = Vec3::new(0.48, 0.63, 0.61).normalized();
        let tangent = axis.cross(Vec3::Y).normalized();
        let bitangent = axis.cross(tangent);
        let pole = p.dot(axis).clamp(-1.0, 1.0);
        let theta = pole.acos();
        let phi = p.dot(bitangent).atan2(p.dot(tangent));
        let phase = 10.0 * phi + 5.0 * (theta * 2.0).sin() + 6.0 * broad;
        let strand = (0.5 + 0.5 * phase.sin()).powi(28);
        let broken = (noise(p * 7.5 + Vec3::splat(7.1)) * 2.5 - 0.6).clamp(0.0, 1.0);
        let strength = strand * broken * (1.0 - pole * pole);
        color += Vec3::new(0.02, 0.25, 0.75) * (strength * 0.32);
    }
    // 4. View-dependent inner rim. No pixels outside the sphere are generated.
    if layers >= 4 {
        color += Vec3::new(0.025, 0.20, 0.40) * ((1.0 - facing).powi(5) * 0.65);
    }
    color
}

/// Live wrapper; the accepted shade implementation above remains unchanged.
/// `gain` must be finite and in [0.20, 1.0], as returned by neutron_live::pulse_at.
pub fn shade_pulsed(fragment: &FragmentInput, camera: Vec3, layers: u8, gain: f64) -> Color {
    let base = shade(fragment, camera, layers);
    if gain == 1.0 || layers < 2 {
        return base;
    }
    let p = direction(fragment.uv);
    let axis = Vec3::new(0.48, 0.63, 0.61).normalized();
    let pole = p.dot(axis).abs().clamp(0.0, 1.0);
    let skirt = Vec3::new(0.015, 0.40, 1.0) * pole.powi(8) * 0.85;
    let granulation = 0.64 + 0.72 * noise(p * 45.0 + Vec3::splat(19.2));
    let core = Vec3::new(1.0, 0.98, 0.96) * pole.powi(32) * 1.5 * granulation;
    base + (skirt + core) * (gain - 1.0)
}
