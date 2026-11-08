use lab5::slideshow::{Input, LABELS, State};
#[test]
fn manual_navigation_wraps_and_restarts_the_dwell() {
    let mut s = State::new(0).unwrap();
    s.advance(
        0.0,
        Input {
            previous: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(s.index(), 6);
    s.advance(7.5, Input::default()).unwrap();
    s.advance(
        0.0,
        Input {
            next: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(s.index(), 0);
    s.advance(0.6, Input::default()).unwrap();
    assert_eq!(s.index(), 0, "manual change must reset dwell");
}
#[test]
fn pause_freezes_the_show_reset_returns_to_pulsar() {
    let mut s = State::new(3).unwrap();
    s.advance(
        0.0,
        Input {
            toggle_pause: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(s.paused());
    let frozen = s;
    s.advance(100.0, Input::default()).unwrap();
    assert_eq!(s, frozen);
    s.advance(
        0.0,
        Input {
            next: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(s.index(), 4);
    assert!(s.paused());
    s.advance(
        0.0,
        Input {
            reset: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(s, State::new(0).unwrap());
}
#[test]
fn every_slide_uses_its_existing_renderer_exactly() {
    use lab5::{
        extras::{self, Body},
        slideshow::Presentation,
    };
    for index in 0..7 {
        let view = Presentation::new(160, 120, 0.4, 0.0, 4, index).unwrap();
        if index == 0 {
            let mut live = lab5::neutron_live::LiveState::new(0.4).unwrap();
            live.advance(0.0, Default::default()).unwrap();
            let mut p = lab5::pulsar::PulsarRenderer::new(160, 120, 4).unwrap();
            p.render(&live).unwrap();
            assert!(view.frame().pixels == p.framebuffer().pixels);
            continue;
        }
        let expected = match index {
            1 => lab5::eyeball::render(160, 120, 0.4, 4).unwrap().0,
            2 => lab5::gas_v2::render(160, 120, 0.4, 4).unwrap().0,
            3 => {
                extras::render(160, 120, 0.4, 4, Body::Volcanic, false)
                    .unwrap()
                    .0
            }
            4 => {
                extras::render(160, 120, 0.4, 4, Body::Desert, true)
                    .unwrap()
                    .0
            }
            5 => {
                extras::render(160, 120, 0.4, 4, Body::Temperate, false)
                    .unwrap()
                    .0
            }
            _ => lab5::render_neutron(160, 120, 0.4, 4).unwrap().0,
        };
        assert!(
            view.frame().pixels == expected.pixels,
            "slide {index} does not use its real renderer"
        );
    }
}
#[test]
fn pulsar_animates_pause_freezes_pixels_and_reset_restores_first_frame() {
    use lab5::slideshow::Presentation;
    let mut p = Presentation::new(160, 120, 0.0, 0.0, 4, 0).unwrap();
    let first = p.frame().pixels.clone();
    p.advance(0.5, Input::default()).unwrap();
    assert!(
        p.frame().pixels != first,
        "pulsar animation missing in slideshow"
    );
    p.advance(
        0.0,
        Input {
            toggle_pause: true,
            ..Default::default()
        },
    )
    .unwrap();
    let paused = p.frame().pixels.clone();
    p.advance(2.0, Input::default()).unwrap();
    assert!(p.frame().pixels == paused);
    p.advance(
        0.0,
        Input {
            next: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(p.state().index(), 1);
    p.advance(
        0.0,
        Input {
            previous: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(p.frame().pixels == paused);
    p.advance(
        0.0,
        Input {
            reset: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(p.frame().pixels == first);
    assert!(!p.state().paused());
}
#[test]
fn state_and_renderer_reject_invalid_input_without_mutating_visible_state() {
    use lab5::slideshow::Presentation;
    assert!(State::new(7).is_err());
    assert!(State::new(usize::MAX).is_err());
    for index in [0, 4] {
        let mut p = Presentation::new(96, 72, 0.0, 0.0, 4, index).unwrap();
        for dt in [f64::NAN, f64::INFINITY, -0.1] {
            let before = p.state();
            let pixels = p.frame().pixels.clone();
            assert!(
                p.advance(
                    dt,
                    Input {
                        reset: true,
                        ..Default::default()
                    }
                )
                .is_err()
            );
            assert_eq!(p.state(), before);
            assert!(p.frame().pixels == pixels);
        }
    }
    for (w, h) in [(0, 64), (64, 0), (usize::MAX, 2)] {
        assert!(Presentation::new(w, h, 0.0, 0.0, 4, 0).is_err());
    }
    assert!(Presentation::new(32, 32, f64::NAN, 0.0, 4, 0).is_err());
    assert!(Presentation::new(32, 32, 0.0, f64::NAN, 4, 4).is_err());
    assert!(Presentation::new(32, 32, 0.0, 0.0, 0, 0).is_err());
    assert!(Presentation::new(32, 32, 0.0, 0.0, 4, 7).is_err());
    let mut state = State::new(0).unwrap();
    state.advance(f64::MAX, Input::default()).unwrap();
    assert!(state.index() < 7);
}
#[test]
fn automatic_renderer_cycle_visits_each_real_scene() {
    use lab5::slideshow::Presentation;
    let mut p = Presentation::new(96, 72, 0.0, 0.0, 4, 0).unwrap();
    for index in 1..=7 {
        p.advance(8.0, Input::default()).unwrap();
        assert_eq!(p.state().index(), index % 7);
        let expected = Presentation::new(96, 72, 0.0, 0.0, 4, index % 7).unwrap();
        assert!(
            p.frame().pixels == expected.frame().pixels,
            "automatic scene {index} stale"
        );
    }
}
#[test]
fn automatic_slides_use_elapsed_time_and_wrap() {
    let mut a = State::new(0).unwrap();
    a.advance(7.9, Input::default()).unwrap();
    assert_eq!(a.index(), 0);
    a.advance(0.2, Input::default()).unwrap();
    assert_eq!(a.index(), 1, "automatic advance missing");
    a.advance(48.0, Input::default()).unwrap();
    assert_eq!(a.index(), 0);
    let mut b = State::new(0).unwrap();
    b.advance(8.1, Input::default()).unwrap();
    assert_eq!(b.index(), 1);
    assert_eq!(LABELS.len(), 7);
}
