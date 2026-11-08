use crate::math::{Vec2, Vec3};
use std::sync::OnceLock;
pub type Color = Vec3;
#[derive(Clone, Copy, Debug)]
pub struct FragmentInput {
    /// Pixel (x, y) y profundidad NDC `[-1, 1]`.
    pub screen_position: Vec3,
    pub world_position: Vec3,
    /// Interpolada con correccion de perspectiva; el shader la renormaliza.
    pub normal: Vec3,
    pub uv: Vec2,
}

/// Maps normalized world normals from [-1,1] to linear RGB [0,1].
/// Diagnostic only: not one of the final celestial shaders.
pub fn diagnostic(fragment: &FragmentInput) -> Color {
    fragment.normal.normalized() * 0.5 + Vec3::splat(0.5)
}
/// Lineal `0..=1` a sRGB de 8 bits empaquetado `0xRRGGBB`, con tabla para no llamar `powf`.
pub fn to_rgb8(color: Color) -> u32 {
    static TABLE: OnceLock<Vec<u8>> = OnceLock::new();
    const STEPS: usize = 4096;
    let table = TABLE.get_or_init(|| {
        (0..=STEPS)
            .map(|i| {
                let l = i as f64 / STEPS as f64;
                let s = if l <= 0.003_130_8 {
                    l * 12.92
                } else {
                    1.055 * l.powf(1.0 / 2.4) - 0.055
                };
                (s * 255.0 + 0.5) as u8
            })
            .collect()
    });
    let ch = |v: f64| {
        let v = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
        table[(v * STEPS as f64 + 0.5) as usize] as u32
    };
    (ch(color.x) << 16) | (ch(color.y) << 8) | ch(color.z)
}
