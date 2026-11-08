pub mod extras;
pub mod eyeball;
pub mod gas;
pub mod gas_v2;
pub mod neutron;
pub mod neutron_live;
pub mod neutron_viewer;
pub mod procedural;
pub mod pulsar;
pub mod rocky;
pub use neutron::render as render_neutron;
pub mod assets;
pub mod math;
pub mod renderer;
pub mod scene;
use renderer::{framebuffer::Framebuffer, stats::RenderStats};
pub const BACKGROUND: u32 = 0x050811;
/// One sphere shaded per fragment. No game assets or material system.
pub fn render_diagnostic(
    width: usize,
    height: usize,
    angle: f64,
) -> Result<(Framebuffer, RenderStats), String> {
    use math::{Transform, Vec3};
    use scene::camera::Camera;
    if !angle.is_finite() {
        return Err("El angulo debe ser finito".into());
    }
    let mut framebuffer = Framebuffer::new(width, height, BACKGROUND)?;
    let sphere = assets::primitives::uv_sphere(64, 128);
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
    let stats = renderer::pipeline::draw_model(
        &mut framebuffer,
        &sphere,
        &transform,
        &camera,
        true,
        renderer::shader::diagnostic,
    );
    Ok((framebuffer, stats))
}
