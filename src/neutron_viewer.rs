//! Cached geometry/framebuffer for the opt-in live viewer.
use crate::{
    assets::{obj::Model, primitives::uv_sphere},
    math::{Transform, Vec3},
    neutron_live::{LiveState, pulse_at},
    renderer::{framebuffer::Framebuffer, pipeline::draw_model, stats::RenderStats},
    scene::camera::Camera,
};
pub struct LiveRenderer {
    framebuffer: Framebuffer,
    sphere: Model,
    camera: Camera,
    layers: u8,
}
impl LiveRenderer {
    pub fn new(width: usize, height: usize, layers: u8) -> Result<Self, String> {
        if !(1..=4).contains(&layers) {
            return Err("Capas debe estar entre 1 y 4".into());
        }
        let framebuffer = Framebuffer::new(width, height, crate::BACKGROUND)?;
        let camera = Camera::new(
            45_f64.to_radians(),
            width as f64 / height as f64,
            0.1,
            100.0,
        )?;
        Ok(Self {
            framebuffer,
            sphere: uv_sphere(64, 128),
            camera,
            layers,
        })
    }
    pub fn render(&mut self, state: &LiveState) -> Result<RenderStats, String> {
        let gain = pulse_at(state.time())?;
        let (yaw, pitch) = state.orbit();
        let d = 3.5 / self.camera.aspect.min(1.0) * state.distance_scale();
        let eye = Vec3::new(
            d * yaw.sin() * pitch.cos(),
            d * pitch.sin(),
            d * yaw.cos() * pitch.cos(),
        );
        let transform = Transform {
            rotation: Vec3::new(0.0, state.angle(), 0.0),
            ..Transform::IDENTITY
        };
        if !eye.is_finite() || !transform.rotation.is_finite() {
            return Err("Pose de visor invalida".into());
        }
        self.camera.position = eye;
        self.camera.target = Vec3::ZERO;
        self.framebuffer.clear();
        Ok(draw_model(
            &mut self.framebuffer,
            &self.sphere,
            &transform,
            &self.camera,
            true,
            |f| crate::neutron::shade_pulsed(f, eye, self.layers, gain),
        ))
    }
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }
}
