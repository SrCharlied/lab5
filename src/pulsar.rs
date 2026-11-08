//! Opt-in artistic exhibition, separate from the accepted coursework surface.
#[path = "pulsar_effects.rs"]
mod effects;
use crate::{
    assets::{obj::Model, primitives::uv_sphere},
    math::{Mat4, Transform},
    neutron_live::{LiveState, pulse_at},
    renderer::{
        framebuffer::Framebuffer, pipeline::draw_model, shader::to_rgb8, stats::RenderStats,
    },
    scene::camera::Camera,
};
pub struct PulsarRenderer {
    framebuffer: Framebuffer,
    linear: Vec<Vec3>,
    sphere: Model,
    camera: Camera,
    curves: Vec<Vec<Vec3>>,
    stages: u8,
}
impl PulsarRenderer {
    pub fn new(w: usize, h: usize, stages: u8) -> Result<Self, String> {
        if !(1..=4).contains(&stages) {
            return Err("Capas debe estar entre 1 y 4".into());
        }
        let framebuffer = Framebuffer::new(w, h, crate::BACKGROUND)?;
        let count = framebuffer.pixels.len();
        let camera = Camera::new(45_f64.to_radians(), w as f64 / h as f64, 0.1, 100.0)?;
        let mut curves = Vec::new();
        for (shell, l) in [1.6, 2.05, 2.5].into_iter().enumerate() {
            for azimuth in 0..4 {
                curves.push(magnetic_line(
                    l,
                    std::f64::consts::TAU * (azimuth as f64 / 4.0 + shell as f64 * 0.035),
                    128,
                )?);
            }
        }
        Ok(Self {
            framebuffer,
            linear: vec![Vec3::ZERO; count],
            sphere: uv_sphere(64, 128),
            camera,
            curves,
            stages,
        })
    }
    pub fn render(&mut self, state: &LiveState) -> Result<RenderStats, String> {
        let gain = pulse_at(state.time())?;
        let (yaw, pitch) = state.orbit();
        let d = 7.4 / self.camera.aspect.min(1.0) * state.distance_scale();
        let eye = Vec3::new(
            d * yaw.sin() * pitch.cos(),
            d * pitch.sin(),
            d * yaw.cos() * pitch.cos(),
        );
        let rotation = Mat4::rotation_y(state.angle());
        self.camera.position = eye;
        self.camera.target = Vec3::ZERO;
        self.framebuffer.clear();
        let decode = |v: u32| {
            let v = v as f64 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let background = Vec3::new(
            decode((crate::BACKGROUND >> 16) & 255),
            decode((crate::BACKGROUND >> 8) & 255),
            decode(crate::BACKGROUND & 255),
        );
        self.linear.fill(background);
        let w = self.framebuffer.width;
        let h = self.framebuffer.height;
        let transform = Transform {
            rotation: Vec3::new(0.0, state.angle(), 0.0),
            ..Transform::IDENTITY
        };
        // The rasterizer invokes this callback only after the real z-test passes.
        let stats = draw_model(
            &mut self.framebuffer,
            &self.sphere,
            &transform,
            &self.camera,
            true,
            |f| {
                let color = shade_surface(f, eye, gain, state.time());
                let index = f.screen_position.y as usize * w + f.screen_position.x as usize;
                self.linear[index] = color;
                color
            },
        );
        effects::apply(
            &mut self.linear,
            (w, h),
            &self.camera,
            rotation,
            &self.curves,
            gain,
            self.stages,
        );
        for (pixel, color) in self.framebuffer.pixels.iter_mut().zip(&self.linear) {
            // Compress emission before the existing sRGB encoder, retaining HDR detail.
            let compress = |v: f64| 1.0 - (-v).exp();
            *pixel = to_rgb8(Vec3::new(
                compress(color.x),
                compress(color.y),
                compress(color.z),
            ));
        }
        Ok(stats)
    }
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }
}
/// Analytic visibility against the unit star only; not general scene occlusion.
pub fn light_visible(eye: Vec3, point: Vec3) -> bool {
    effects::visible(eye, point)
}
/// Sample an illustrative dipole-shaped curve, in unit-star local space.
pub fn magnetic_line(l: f64, phi: f64, steps: usize) -> Result<Vec<crate::math::Vec3>, String> {
    if !l.is_finite()
        || !(1.1..=3.0).contains(&l)
        || !phi.is_finite()
        || !(16..=1024).contains(&steps)
    {
        return Err("Parametros de arco invalidos".into());
    }
    let axis = Vec3::new(0.48, 0.63, 0.61).normalized();
    let t = axis.cross(Vec3::Y).normalized();
    let b = axis.cross(t);
    let phi = phi.rem_euclid(std::f64::consts::TAU);
    let radial = t * phi.cos() + b * phi.sin();
    let start = (1.0 / l).sqrt().asin();
    Ok((0..=steps)
        .map(|i| {
            let theta = start + (std::f64::consts::PI - 2.0 * start) * i as f64 / steps as f64;
            let radius = l * theta.sin().powi(2);
            (axis * theta.cos() + radial * theta.sin()) * radius
        })
        .collect())
}
use crate::{
    math::Vec3,
    procedural::{direction, field, noise},
    renderer::shader::{Color, FragmentInput},
};
/// Finite unit-sphere fragments, gain from pulse_at, nonnegative finite time.
pub fn shade_surface(f: &FragmentInput, camera: Vec3, gain: f64, time: f64) -> Color {
    let p = direction(f.uv);
    let flow = Vec3::new(
        (time * 0.13).sin(),
        (time * 0.17).cos(),
        (time * 0.11).sin(),
    ) * 0.18;
    let warp = Vec3::new(
        field(p * 4.0 + Vec3::splat(4.7)),
        field(p * 4.0 + Vec3::splat(12.3)),
        field(p * 4.0 + Vec3::splat(24.1)),
    ) - Vec3::splat(0.5);
    let turbulent = field(p * 19.0 + warp * 3.0 + flow);
    let ridges = (1.0 - (2.0 * turbulent - 1.0).abs()).powi(7);
    let filaments = (1.0 - (2.0 * noise(p * 47.0 + warp * 4.0 + flow) - 1.0).abs()).powi(15);
    let facing = f
        .normal
        .normalized()
        .dot((camera - f.world_position).normalized())
        .max(0.0);
    let mut color = Vec3::new(0.008, 0.045, 0.14)
        + Vec3::new(0.035, 0.36, 0.90) * ridges
        + Vec3::new(0.16, 0.36, 0.50) * filaments * ridges;
    color = color * (0.55 + 0.45 * facing.sqrt());
    let pole = p.dot(Vec3::new(0.48, 0.63, 0.61).normalized()).abs();
    color += Vec3::new(1.2, 1.65, 1.8) * pole.powi(35) * gain * 1.4;
    color += Vec3::new(0.10, 0.65, 1.0) * (1.0 - facing).powi(4) * 1.7;
    color
}
