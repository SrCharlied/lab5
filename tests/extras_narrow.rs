use lab5::{
    BACKGROUND,
    extras::{self, Body},
};

#[test]
fn extreme_portrait_keeps_bonus_planet_and_optional_moon_visible() {
    for (w, h) in [(48, 1080), (24, 1080)] {
        for (body, moon) in [
            (Body::Desert, false),
            (Body::Desert, true),
            (Body::Volcanic, false),
            (Body::Temperate, false),
        ] {
            let (fb, stats) = extras::render(w, h, 0.0, 4, body, moon).unwrap();
            let foreground = fb.pixels.iter().filter(|&&p| p != BACKGROUND).count();
            assert!(
                foreground > 20,
                "{body:?} {w}x{h} moon={moon}: empty or subpixel frame ({foreground})"
            );
            assert!(stats.pixels_written > 20);
            assert_eq!(stats.objects_submitted, if moon { 2 } else { 1 });
            for (i, &pixel) in fb.pixels.iter().enumerate() {
                if pixel != BACKGROUND {
                    let (x, y) = (i % w, i / w);
                    assert!(x > 0 && x < w - 1 && y > 0 && y < h - 1);
                }
            }
            if moon {
                let (alone, _) = extras::render(w, h, 0.0, 4, body, false).unwrap();
                assert!(
                    fb.pixels.iter().zip(&alone.pixels).any(|(a, b)| a != b),
                    "moon disappeared"
                );
            }
        }
    }
}
