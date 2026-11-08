//! Reused procedural math from the accepted neutron shader, left unchanged there.
//! Fields evaluate coordinates directly; no image or precomputed texture is sampled.
use crate::math::{Vec2, Vec3};
use std::f64::consts::{PI, TAU};
pub fn direction(uv: Vec2) -> Vec3 {
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
pub fn noise(p: Vec3) -> f64 {
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
pub fn field(p: Vec3) -> f64 {
    (noise(p) * 0.57
        + noise(p * 2.03 + Vec3::splat(13.7)) * 0.29
        + noise(p * 4.11 + Vec3::splat(31.1)) * 0.14)
        .clamp(0.0, 1.0)
}
pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
