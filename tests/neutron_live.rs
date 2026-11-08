use lab5::neutron_live::{Controls, LiveState};
#[test]
fn live_renderer_preserves_startup_and_reacts_to_clock_and_camera() {
    use lab5::neutron_live::LiveRenderer;
    let mut state = LiveState::new(-0.65).unwrap();
    let mut viewer = LiveRenderer::new(192, 128, 4).unwrap();
    let stats = viewer.render(&state).unwrap();
    let first = viewer.framebuffer().pixels.clone();
    let (reference, expected) = lab5::render_neutron(192, 128, -0.65, 4).unwrap();
    assert!(
        first == reference.pixels,
        "live t=0 must retain the approved appearance exactly"
    );
    assert_eq!(stats, expected);
    let ptr = viewer.framebuffer().pixels.as_ptr();
    state.advance(1.0, Controls::default()).unwrap();
    viewer.render(&state).unwrap();
    assert!(
        first != viewer.framebuffer().pixels,
        "elapsed time must change the actual rendered image"
    );
    assert_eq!(
        viewer.framebuffer().pixels.as_ptr(),
        ptr,
        "reuse the framebuffer between frames"
    );
    state
        .advance(
            0.0,
            Controls {
                toggle_pause: true,
                ..Controls::default()
            },
        )
        .unwrap();
    let paused = viewer.framebuffer().pixels.clone();
    state.advance(1.0, Controls::default()).unwrap();
    viewer.render(&state).unwrap();
    assert!(
        paused == viewer.framebuffer().pixels,
        "pause must freeze the actual image"
    );
    state
        .advance(
            0.3,
            Controls {
                orbit_x: 1.0,
                ..Controls::default()
            },
        )
        .unwrap();
    viewer.render(&state).unwrap();
    assert!(
        paused != viewer.framebuffer().pixels,
        "orbit must affect the real camera even when paused"
    );
}

#[test]
fn pulse_modulates_only_the_hot_regions_and_has_an_exact_approved_peak() {
    use lab5::{
        math::{Vec2, Vec3},
        neutron,
        neutron_live::pulse_at,
        renderer::shader::FragmentInput,
    };
    assert_eq!(pulse_at(0.0).unwrap(), 1.0);
    let low = pulse_at(3.6 * 0.5).unwrap();
    assert!(
        (low - 0.20).abs() < 1e-12,
        "pulse must dim smoothly without a blackout"
    );
    assert!(pulse_at(f64::NAN).is_err());
    let axis = Vec3::new(0.48, 0.63, 0.61).normalized();
    let f = FragmentInput {
        screen_position: Vec3::ZERO,
        world_position: axis,
        normal: axis,
        uv: Vec2::new(
            axis.z.atan2(axis.x).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU,
            axis.y.acos() / std::f64::consts::PI,
        ),
    };
    let camera = axis * 3.5;
    let peak = neutron::shade(&f, camera, 4);
    assert_eq!(neutron::shade_pulsed(&f, camera, 4, 1.0), peak);
    let dimmed = neutron::shade_pulsed(&f, camera, 4, low);
    assert!(
        dimmed.x < peak.x * 0.4 && dimmed.x > peak.x * 0.1,
        "hot core must visibly pulse"
    );
    assert_eq!(
        neutron::shade_pulsed(&f, camera, 1, low),
        neutron::shade(&f, camera, 1),
        "base layer must not pulse"
    );
}

#[test]
fn camera_orbits_and_zooms_even_while_simulation_is_paused() {
    let mut s = LiveState::new(-0.65).unwrap();
    s.advance(
        0.0,
        Controls {
            toggle_pause: true,
            ..Controls::default()
        },
    )
    .unwrap();
    s.advance(
        0.5,
        Controls {
            orbit_x: 1.0,
            orbit_y: 1.0,
            zoom: 1.0,
            ..Controls::default()
        },
    )
    .unwrap();
    assert!(
        s.orbit().0 > 0.0 && s.orbit().1 > 0.0,
        "camera controls must change orbit angles"
    );
    assert!(
        s.distance_scale() < 1.0,
        "positive zoom must approach the sphere"
    );
    assert_eq!(s.time(), 0.0, "moving camera must not resume rotation");
    s.advance(
        100.0,
        Controls {
            orbit_y: 1.0,
            zoom: 1.0,
            ..Controls::default()
        },
    )
    .unwrap();
    assert!(s.orbit().1 <= 1.2 && s.distance_scale() >= 0.85);
    let before = s;
    assert!(
        s.advance(
            0.1,
            Controls {
                orbit_x: f64::NAN,
                ..Controls::default()
            }
        )
        .is_err()
    );
    assert_eq!(
        s, before,
        "invalid inputs must not partially mutate the viewer"
    );
}

#[test]
fn pause_freezes_the_clock_and_reset_restores_the_initial_state() {
    let mut s = LiveState::new(-0.65).unwrap();
    s.advance(0.5, Controls::default()).unwrap();
    let frozen = s.time();
    s.advance(
        0.5,
        Controls {
            toggle_pause: true,
            ..Controls::default()
        },
    )
    .unwrap();
    assert_eq!(
        s.time(),
        frozen,
        "pause must freeze the simulation clock immediately"
    );
    s.advance(0.2, Controls::default()).unwrap();
    assert_eq!(s.time(), frozen);
    s.advance(
        0.1,
        Controls {
            toggle_pause: true,
            ..Controls::default()
        },
    )
    .unwrap();
    assert!((s.time() - (frozen + 0.1)).abs() < 1e-12);
    s.advance(
        0.2,
        Controls {
            reset: true,
            ..Controls::default()
        },
    )
    .unwrap();
    assert_eq!(s, LiveState::new(-0.65).unwrap());
}

#[test]
fn spin_uses_elapsed_seconds_not_frame_count() {
    let mut a = LiveState::new(-0.65).unwrap();
    let mut b = a;
    a.advance(0.5, Controls::default()).unwrap();
    for _ in 0..5 {
        b.advance(0.1, Controls::default()).unwrap();
    }
    assert!(
        (a.time() - 0.5).abs() < 1e-12,
        "simulation time must advance"
    );
    assert!(
        (a.angle() - (-0.65 + 0.42 * 0.5)).abs() < 1e-12,
        "model must rotate"
    );
    assert!(
        (a.angle() - b.angle()).abs() < 1e-12,
        "same elapsed time must not depend on frame count"
    );
}
