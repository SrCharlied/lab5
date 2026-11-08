//! Separate exhibition adapter; the approved live_app remains unchanged.
use crate::live_app::Options;
use lab5::{
    neutron_live::{Controls, LiveState},
    pulsar::PulsarRenderer,
    renderer::image::save_png,
};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Instant;
pub fn run(options: Options) -> Result<(), String> {
    let mut state = LiveState::new(options.angle)?;
    state.advance(options.time, Controls::default())?;
    let mut viewer = PulsarRenderer::new(options.width, options.height, options.layers)?;
    let start = Instant::now();
    let stats = viewer.render(&state)?;
    println!(
        "PULSAR {}x{} | t={:.3}s | esfera: {} pixeles escritos | frame CPU {:.1}ms",
        options.width,
        options.height,
        state.time(),
        stats.pixels_written,
        start.elapsed().as_secs_f64() * 1000.0
    );
    if options.headless {
        save_png(&options.output, viewer.framebuffer())?;
        println!("PNG: {}", options.output.display());
        return Ok(());
    }
    let mut window = Window::new(
        "Lab 5 | PULSAR (exhibicion)",
        options.width,
        options.height,
        WindowOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    window.set_target_fps(30);
    println!(
        "EXHIBICION: Flechas orbitar | W/S zoom | Espacio pausa | R reiniciar | P PNG | Esc salir"
    );
    let mut shown = 0_usize;
    let mut last = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.set_title(&format!(
            "Lab 5 | PULSAR | {} | t={:.1}s | Flechas/W/S camara | Espacio pausa",
            if state.paused() { "PAUSA" } else { "GIRO" },
            state.time()
        ));
        window
            .update_with_buffer(&viewer.framebuffer().pixels, options.width, options.height)
            .map_err(|e| e.to_string())?;
        shown += 1;
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            save_png(&options.output, viewer.framebuffer())?;
            println!("PNG: {}", options.output.display());
        }
        if options.frames.is_some_and(|n| shown >= n) {
            break;
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;
        let axis = |a, b| {
            f64::from(u8::from(window.is_key_down(a))) - f64::from(u8::from(window.is_key_down(b)))
        };
        let input = Controls {
            orbit_x: axis(Key::Right, Key::Left),
            orbit_y: axis(Key::Up, Key::Down),
            zoom: axis(Key::W, Key::S),
            toggle_pause: window.is_key_pressed(Key::Space, KeyRepeat::No),
            reset: window.is_key_pressed(Key::R, KeyRepeat::No),
        };
        let previous = state;
        state.advance(dt, input)?;
        if state != previous {
            viewer.render(&state)?;
        }
    }
    println!(
        "Ventana PULSAR cerrada: {shown} cuadros presentados | t={:.6}s",
        state.time()
    );
    Ok(())
}
