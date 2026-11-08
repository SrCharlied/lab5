//! Guards added after the live features: born green, not feature REDs.
use lab5::neutron_live::{Controls, LiveRenderer, LiveState, pulse_at};
#[test]
fn invalid_clock_or_controls_leave_every_state_field_unchanged() {
    let mut state = LiveState::new(-0.65).unwrap();
    state.advance(0.2, Controls::default()).unwrap();
    for dt in [-1.0, f64::NAN, f64::INFINITY] {
        let before = state;
        assert!(state.advance(dt, Controls::default()).is_err());
        assert_eq!(state, before);
    }
    for input in [
        Controls {
            zoom: f64::NAN,
            ..Controls::default()
        },
        Controls {
            orbit_y: 2.0,
            ..Controls::default()
        },
        Controls {
            orbit_x: f64::INFINITY,
            ..Controls::default()
        },
    ] {
        let before = state;
        assert!(state.advance(0.1, input).is_err());
        assert_eq!(state, before);
    }
    assert!(LiveState::new(f64::NAN).is_err());
}
#[test]
fn pulse_is_periodic_finite_and_bounded_for_sampled_times() {
    for i in 0..100 {
        let t = i as f64 * 0.037;
        let a = pulse_at(t).unwrap();
        assert!((0.20..=1.0).contains(&a));
        assert!((a - pulse_at(t + 3.6).unwrap()).abs() < 1e-12);
    }
    assert!(pulse_at(f64::MAX).unwrap().is_finite());
    assert!(pulse_at(-0.1).is_err());
}
#[test]
fn camera_reset_restores_the_approved_render_not_just_the_clock() {
    let mut state = LiveState::new(-0.65).unwrap();
    let mut viewer = LiveRenderer::new(192, 128, 4).unwrap();
    viewer.render(&state).unwrap();
    let reference = viewer.framebuffer().pixels.clone();
    state
        .advance(
            0.7,
            Controls {
                orbit_x: 1.0,
                orbit_y: -1.0,
                zoom: 1.0,
                ..Controls::default()
            },
        )
        .unwrap();
    viewer.render(&state).unwrap();
    assert!(reference != viewer.framebuffer().pixels);
    state
        .advance(
            0.0,
            Controls {
                reset: true,
                ..Controls::default()
            },
        )
        .unwrap();
    viewer.render(&state).unwrap();
    assert!(reference == viewer.framebuffer().pixels);
    assert!(!state.paused());
}
#[test]
fn paused_camera_sweep_keeps_the_sphere_visible_without_edge_clipping() {
    for (w, h) in [(192, 128), (128, 192)] {
        let mut viewer = LiveRenderer::new(w, h, 4).unwrap();
        let mut s = LiveState::new(-0.65).unwrap();
        s.advance(
            0.0,
            Controls {
                toggle_pause: true,
                ..Controls::default()
            },
        )
        .unwrap();
        for _ in 0..20 {
            s.advance(
                0.2,
                Controls {
                    orbit_x: 1.0,
                    orbit_y: 1.0,
                    zoom: 1.0,
                    ..Controls::default()
                },
            )
            .unwrap();
            viewer.render(&s).unwrap();
            let fb = viewer.framebuffer();
            assert_ne!(fb.pixels[h / 2 * w + w / 2], lab5::BACKGROUND);
            assert!((0..h).all(|y| fb.pixels[y * w] == lab5::BACKGROUND
                && fb.pixels[(y + 1) * w - 1] == lab5::BACKGROUND));
        }
        assert_eq!(s.time(), 0.0);
    }
}
#[test]
fn layer_one_does_not_breathe_when_geometry_and_camera_are_fixed() {
    use lab5::{
        math::{Vec2, Vec3},
        neutron,
        renderer::shader::FragmentInput,
    };
    let f = FragmentInput {
        screen_position: Vec3::ZERO,
        world_position: Vec3::Z,
        normal: Vec3::Z,
        uv: Vec2::new(0.31, 0.27),
    };
    assert_eq!(
        neutron::shade_pulsed(&f, Vec3::Z * 3.5, 1, pulse_at(0.0).unwrap()),
        neutron::shade_pulsed(&f, Vec3::Z * 3.5, 1, pulse_at(1.8).unwrap())
    );
}
