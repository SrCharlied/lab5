//! Opt-in artistic bonus planets. No external assets.
use crate::{
    math::Vec3,
    renderer::{
        framebuffer::Framebuffer,
        shader::{Color, FragmentInput},
        stats::RenderStats,
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Volcanic,
    Desert,
    Temperate,
}
// Surface identities are evaluated from local UV directions, not world position.
pub fn shade(body: Body, f: &FragmentInput, layers: u8) -> Color {
    use crate::procedural::{direction, field, smoothstep};
    let p = direction(f.uv);
    let light = 0.065
        + 0.935
            * f.normal
                .normalized()
                .dot(Vec3::new(-0.65, 0.8, 1.0).normalized())
                .max(0.0);
    if body == Body::Desert {
        let terrain = field(p * 3.5 + Vec3::new(18.2, 4.6, 7.9));
        let mut color = Vec3::new(0.20, 0.085, 0.025).lerp(Vec3::new(0.62, 0.34, 0.10), terrain);
        if layers >= 2 {
            let basin = 1.0 - smoothstep(0.30, 0.45, terrain);
            color = color.lerp(Vec3::new(0.075, 0.035, 0.018), basin * 0.75);
        }
        if layers >= 3 {
            let strata = (p.y * 32.0 + field(p * 4.2 + Vec3::splat(12.4)) * 15.0).sin() * 0.5 + 0.5;
            color = color.lerp(
                Vec3::new(0.72, 0.40, 0.15),
                smoothstep(0.35, 0.90, strata) * 0.26,
            );
        }
        if layers >= 4 {
            let dunes = (p.x * 78.0 + p.z * 23.0 + field(p * 7.0) * 15.0).sin() * 0.5 + 0.5;
            let grain = field(p * 65.0 + Vec3::splat(8.1));
            color = color * (0.78 + 0.14 * dunes + 0.16 * grain);
        }
        return color * light;
    }
    if body == Body::Temperate {
        let terrain = field(p * 3.3 + Vec3::new(2.7, 13.1, 7.4));
        let land = smoothstep(0.48, 0.51, terrain);
        let mut color = Vec3::new(0.004, 0.038, 0.068).lerp(
            Vec3::new(0.012, 0.16, 0.19),
            smoothstep(0.32, 0.49, terrain),
        );
        if layers >= 2 {
            let dry =
                smoothstep(0.40, 0.68, field(p * 7.0 + Vec3::splat(14.3))) * (1.0 - p.y.abs());
            let vegetation =
                Vec3::new(0.035, 0.16, 0.033).lerp(Vec3::new(0.23, 0.24, 0.065), dry * 0.65);
            color = color.lerp(vegetation, land);
        }
        if layers >= 3 {
            let ice = smoothstep(0.81, 0.96, p.y.abs() + 0.035 * (field(p * 18.0) - 0.5));
            color = color.lerp(Vec3::new(0.62, 0.72, 0.73), ice);
        }
        if layers >= 4 {
            let warp = field(p * 4.8 + Vec3::splat(18.5));
            let cloud = smoothstep(0.55, 0.72, field(p * 8.0 + Vec3::splat(warp * 1.4)));
            color = color.lerp(Vec3::new(0.75, 0.82, 0.80), cloud * 0.68);
        }
        return color * light;
    }
    let grain = field(p * 34.0 + Vec3::splat(7.2));
    let mut rock = Vec3::new(0.008, 0.004, 0.003).lerp(Vec3::new(0.048, 0.021, 0.012), grain);
    if layers >= 2 {
        let scars = smoothstep(0.40, 0.66, field(p * 6.0 + Vec3::splat(12.3)));
        rock = rock.lerp(Vec3::new(0.003, 0.002, 0.002), scars * 0.65);
    }
    let mut emission = Vec3::ZERO;
    if layers >= 3 {
        let warp = Vec3::new(
            field(p * 3.2),
            field(p * 3.2 + Vec3::splat(9.1)),
            field(p * 3.2 + Vec3::splat(23.4)),
        ) - Vec3::splat(0.5);
        let q = p * 4.5 + warp * 2.0;
        let d = (field(q + Vec3::splat(4.8)) - 0.49).abs();
        let lava = 1.0 - smoothstep(0.009, 0.041, d);
        let heat = field(p * 12.0 + Vec3::splat(31.2));
        emission =
            Vec3::new(0.95, 0.045, 0.002).lerp(Vec3::new(2.0, 0.65, 0.035), heat * lava) * lava;
        if layers >= 4 {
            let cooled = 0.40 + 0.60 * smoothstep(0.27, 0.65, field(p * 23.0 + Vec3::splat(41.5)));
            emission = emission * cooled;
            rock = rock * (0.72 + 0.45 * grain);
        }
    }
    rock * light + emission
}
pub fn moon_shade(f: &FragmentInput) -> Color {
    use crate::procedural::{direction, field, smoothstep};
    let p = direction(f.uv);
    let mut value = 0.15 + 0.14 * field(p * 6.0 + Vec3::splat(8.4));
    for axis in [
        Vec3::new(-0.4, 0.2, 0.9),
        Vec3::new(0.4, -0.3, 0.86),
        Vec3::new(0.15, 0.63, 0.75),
        Vec3::new(-0.8, -0.4, 0.3),
        Vec3::new(0.75, 0.4, 0.5),
        Vec3::new(0.4, 0.1, -0.9),
        Vec3::new(-0.5, 0.7, -0.6),
        Vec3::new(-0.1, -0.8, -0.4),
    ] {
        let d = 1.0 - p.dot(axis.normalized());
        let bowl = 1.0 - smoothstep(0.004, 0.022, d);
        let rim = 1.0 - smoothstep(0.004, 0.012, (d - 0.024).abs());
        value *= 1.0 - 0.55 * bowl;
        value += rim * 0.08;
    }
    let light = 0.065
        + 0.935
            * f.normal
                .normalized()
                .dot(Vec3::new(-0.65, 0.8, 1.0).normalized())
                .max(0.0);
    Vec3::new(value, value * 0.95, value * 0.88) * light
}
pub fn render(
    w: usize,
    h: usize,
    angle: f64,
    layers: u8,
    body: Body,
    moon: bool,
) -> Result<(Framebuffer, RenderStats), String> {
    use crate::{
        assets::primitives::uv_sphere, math::Transform, renderer::pipeline::draw_model,
        scene::camera::Camera,
    };
    if !(1..=4).contains(&layers) {
        return Err("Capas debe estar entre 1 y 4".into());
    }
    if !angle.is_finite() {
        return Err("El angulo debe ser finito".into());
    }
    if moon && body != Body::Desert {
        return Err("La luna requiere el planeta desertico".into());
    }
    let mut fb = Framebuffer::new(w, h, crate::BACKGROUND)?;
    let mut camera = Camera::new(45_f64.to_radians(), w as f64 / h as f64, 0.1, 100.0)?;
    let distance = if body == Body::Desert { 4.8 } else { 3.5 };
    camera.position = Vec3::new(0.0, 0.0, distance / camera.aspect.min(1.0));
    // Keep both bodies within the far plane even for extreme portrait framing.
    let far = camera.position.z + 2.0;
    if far > camera.far {
        camera.set_lens(camera.fov_y, camera.aspect, 0.1, far)?;
    }
    camera.target = Vec3::ZERO;
    let tr = Transform {
        rotation: Vec3::new(0.0, angle, 0.0),
        ..Transform::IDENTITY
    };
    let mut stats = draw_model(&mut fb, &uv_sphere(64, 128), &tr, &camera, true, |f| {
        shade(body, f, layers)
    });
    if moon {
        let satellite = Transform {
            position: Vec3::new(1.45, 0.72, 0.35),
            scale: Vec3::splat(0.22),
            rotation: Vec3::new(0.0, angle, 0.0),
        };
        stats += draw_model(
            &mut fb,
            &uv_sphere(32, 64),
            &satellite,
            &camera,
            true,
            moon_shade,
        );
    }
    Ok((fb, stats))
}
