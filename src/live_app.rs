//! Window adapter: pure controls/state and rendering live in the library.
use lab5::{
    neutron_live::{Controls, LiveRenderer, LiveState},
    renderer::image::save_png,
};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::{path::PathBuf, time::Instant};

pub struct Options {
    pub width: usize,
    pub height: usize,
    pub angle: f64,
    pub time: f64,
    pub layers: u8,
    pub headless: bool,
    pub output: PathBuf,
    pub frames: Option<usize>,
}
pub fn run(options: Options) -> Result<(), String> {
    let mut state = LiveState::new(options.angle)?;
    state.advance(options.time, Controls::default())?;
    let mut viewer = LiveRenderer::new(options.width, options.height, options.layers)?;
    let stats = viewer.render(&state)?;
    println!(
        "LIVE {}x{} | t={:.3}s | {} pixeles escritos",
        options.width,
        options.height,
        state.time(),
        stats.pixels_written
    );
    if options.headless {
        save_png(&options.output, viewer.framebuffer())?;
        println!("PNG: {}", options.output.display());
        return Ok(());
    }
    let mut window = Window::new(
        "Lab 5 | estrella en vivo",
        options.width,
        options.height,
        WindowOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    window.set_target_fps(60);
    println!(
        "Flechas: orbitar | W/S: acercar/alejar | Espacio: pausa | R: reiniciar | P: PNG | Esc: salir"
    );
    let mut shown = 0_usize;
    let mut last = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.set_title(&format!(
            "Lab 5 | estrella en vivo | {} | t={:.1}s | Flechas/W/S camara | Espacio pausa",
            if state.paused() { "PAUSA" } else { "GIRO" },
            state.time()
        ));
        window
            .update_with_buffer(&viewer.framebuffer().pixels, options.width, options.height)
            .map_err(|e| e.to_string())?;
        shown += 1;
        // Capture exactly the buffer just presented, before changing simulation/camera.
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            save_png(&options.output, viewer.framebuffer())?;
            println!("PNG: {}", options.output.display());
        }
        if options.frames.is_some_and(|limit| shown >= limit) {
            break;
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;
        let axis = |positive: Key, negative: Key| {
            f64::from(u8::from(window.is_key_down(positive)))
                - f64::from(u8::from(window.is_key_down(negative)))
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
        "Ventana LIVE cerrada: {shown} cuadros presentados | t={:.6}s",
        state.time()
    );
    Ok(())
}
