//! Screen-space additive emission from projected procedural 3D curves.
//! This is not volumetric transport or a general transparent-geometry renderer.
use crate::{
    math::{Mat4, Vec3},
    scene::camera::Camera,
};

pub(super) fn visible(eye: Vec3, p: Vec3) -> bool {
    if !eye.is_finite() || !p.is_finite() {
        return false;
    }
    let delta = p - eye;
    let distance = delta.length();
    let e2 = eye.dot(eye);
    if !distance.is_finite() || distance <= 0.0 || !e2.is_finite() || e2 <= 1.0 {
        return false;
    }
    let dir = delta / distance;
    let along = -eye.dot(dir);
    let discriminant = along * along - (e2 - 1.0);
    if discriminant <= 0.0 {
        return true;
    }
    let first = along - discriminant.sqrt();
    !(first > 0.0 && first < distance - 1e-5)
}
fn project(camera: &Camera, w: usize, h: usize, p: Vec3) -> Option<Vec3> {
    let q = camera.view_projection().transform(p.extend(1.0));
    if !q.w.is_finite() || q.w <= 0.0 {
        return None;
    }
    let n = q.xyz() / q.w;
    n.is_finite().then_some(Vec3::new(
        (n.x + 1.0) * w as f64 * 0.5,
        (1.0 - n.y) * h as f64 * 0.5,
        n.z,
    ))
}
fn spot(buffer: &mut [Vec3], size: (usize, usize), center: Vec3, sigma: f64, color: Vec3) {
    let (w, h) = size;
    let radius = (sigma * 4.0).ceil();
    let x0 = (center.x - radius).floor().max(0.0) as usize;
    let x1 = (center.x + radius).ceil().min(w as f64 - 1.0) as isize;
    let y0 = (center.y - radius).floor().max(0.0) as usize;
    let y1 = (center.y + radius).ceil().min(h as f64 - 1.0) as isize;
    if x1 < 0 || y1 < 0 {
        return;
    }
    for y in y0..=y1 as usize {
        for x in x0..=x1 as usize {
            let d2 = (x as f64 + 0.5 - center.x).powi(2) + (y as f64 + 0.5 - center.y).powi(2);
            let a = (-d2 / (2.0 * sigma * sigma)).exp();
            buffer[y * w + x] += color * a;
        }
    }
}
struct Painter<'a> {
    buffer: &'a mut [Vec3],
    size: (usize, usize),
    camera: &'a Camera,
}
impl Painter<'_> {
    fn segment(&mut self, a: Vec3, b: Vec3, color: Vec3, width: f64) {
        let (w, h) = self.size;
        let (Some(pa), Some(pb)) = (project(self.camera, w, h, a), project(self.camera, w, h, b))
        else {
            return;
        };
        let steps = ((pb.x - pa.x).abs().max((pb.y - pa.y).abs()).ceil() as usize).clamp(1, 1024);
        let scale = (w.min(h) as f64 / 720.0).max(0.35);
        let sigma = (width * scale).max(0.45);
        for i in 0..steps {
            let t = i as f64 / steps as f64;
            let p = a.lerp(b, t);
            if !visible(self.camera.position, p) {
                continue;
            }
            let Some(center) = project(self.camera, w, h, p) else {
                continue;
            };
            spot(self.buffer, self.size, center, sigma * 3.5, color * 0.035);
            spot(self.buffer, self.size, center, sigma, color);
        }
    }
}
pub(super) fn apply(
    buffer: &mut [Vec3],
    size: (usize, usize),
    camera: &Camera,
    rotation: Mat4,
    curves: &[Vec<Vec3>],
    gain: f64,
    stages: u8,
) {
    let transform = |p: Vec3| rotation.transform_dir(p);
    if stages >= 2 {
        let mut painter = Painter {
            buffer,
            size,
            camera,
        };
        for (index, curve) in curves.iter().enumerate() {
            let brightness = 0.105 + 0.035 * (index % 3) as f64;
            for pair in curve.windows(2) {
                painter.segment(
                    transform(pair[0]),
                    transform(pair[1]),
                    Vec3::new(0.12, 0.57, 0.95) * brightness,
                    0.8,
                );
            }
        }
    }
    if stages >= 3 {
        let axis = Vec3::new(0.48, 0.63, 0.61).normalized();
        let tangent = axis.cross(Vec3::Y).normalized();
        let bitangent = axis.cross(tangent);
        let mut painter = Painter {
            buffer,
            size,
            camera,
        };
        for sign in [-1.0, 1.0] {
            let origin = axis * sign * 1.005;
            for step in 0..36 {
                let t0 = step as f64 / 36.0;
                let t1 = (step + 1) as f64 / 36.0;
                let a = axis * sign * (1.005 + 3.2 * t0);
                let b = axis * sign * (1.005 + 3.2 * t1);
                let fade = (1.0 - t0).powf(0.65) * (0.5 + 0.5 * gain);
                painter.segment(
                    transform(a),
                    transform(b),
                    Vec3::new(0.08, 0.55, 1.0) * fade * 0.10,
                    2.0 + t0 * 4.5,
                );
                painter.segment(
                    transform(a),
                    transform(b),
                    Vec3::new(0.5, 0.9, 1.2) * fade * 0.13,
                    0.6,
                );
            }
            for ray in 0..8 {
                let phi = std::f64::consts::TAU * ray as f64 / 8.0;
                let radial = tangent * phi.cos() + bitangent * phi.sin();
                for step in 0..20 {
                    let t0 = step as f64 / 20.0;
                    let t1 = (step + 1) as f64 / 20.0;
                    let a = origin + axis * sign * (3.2 * t0) + radial * (0.22 * t0);
                    let b = origin + axis * sign * (3.2 * t1) + radial * (0.22 * t1);
                    painter.segment(
                        transform(a),
                        transform(b),
                        Vec3::new(0.1, 0.5, 0.8) * (1.0 - t0) * (0.5 + 0.5 * gain) * 0.035,
                        0.55,
                    );
                }
            }
        }
    }
    if stages >= 4 {
        let (w, h) = size;
        let d = camera.position.length();
        let radius = h as f64 / (2.0 * (camera.fov_y * 0.5).tan() * (d * d - 1.0).sqrt());
        for y in 0..h {
            for x in 0..w {
                let r = ((x as f64 + 0.5 - w as f64 * 0.5).powi(2)
                    + (y as f64 + 0.5 - h as f64 * 0.5).powi(2))
                .sqrt()
                    / radius;
                // Exterior-only analytic corona; not a convolution bloom.
                if r >= 1.0 {
                    let rim = (-((r - 1.0) / 0.105).powi(2)).exp() * 0.65;
                    let haze = (-(r - 1.0) / 0.42).exp() * 0.035;
                    buffer[y * w + x] += Vec3::new(0.06, 0.58, 1.0) * (rim + haze);
                }
            }
        }
    }
}
