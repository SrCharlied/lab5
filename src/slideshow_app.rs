//! Single-window presentation adapter. Legacy individual viewers remain available.
use crate::live_app::Options;
use lab5::{
    renderer::image::save_png,
    slideshow::{Input, LABELS, Presentation},
};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::{path::PathBuf, time::Instant};
fn capture(options: &Options, custom: bool, show: &Presentation) -> Result<(), String> {
    let index = show.state().index();
    let path = if custom {
        options.output.clone()
    } else if index == 0 {
        PathBuf::from("renders/captures/pulsar-main.png")
    } else {
        PathBuf::from(format!("renders/captures/slide-{:02}.png", index + 1))
    };
    save_png(&path, show.frame())?;
    println!("PNG: {}", path.display());
    Ok(())
}
pub fn run(options: Options, index: usize, custom_output: bool) -> Result<(), String> {
    let mut show = Presentation::new(
        options.width,
        options.height,
        options.angle,
        options.time,
        options.layers,
        index,
    )?;
    println!(
        "SLIDESHOW {}/{} | {}",
        index + 1,
        LABELS.len(),
        LABELS[index]
    );
    if options.headless {
        return capture(&options, custom_output, &show);
    }
    let mut window = Window::new(
        "Lab 5 | slideshow",
        options.width,
        options.height,
        WindowOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    window.set_target_fps(30);
    println!("Flechas: anterior/siguiente | Espacio: pausa | R: inicio | P: PNG | Esc: salir");
    let mut shown = 0_usize;
    let mut last = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let state = show.state();
        window.set_title(&format!(
            "Lab 5 | {}/{} {} | {} | Izq/Der: cambiar | Espacio: pausa | P: PNG",
            state.index() + 1,
            LABELS.len(),
            LABELS[state.index()],
            if state.paused() { "PAUSA" } else { "AUTO 8s" }
        ));
        window
            .update_with_buffer(&show.frame().pixels, options.width, options.height)
            .map_err(|e| e.to_string())?;
        shown += 1;
        // Capture exactly the presented frame before applying the next input/update.
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            capture(&options, custom_output, &show)?;
        }
        if options.frames.is_some_and(|limit| shown >= limit) {
            break;
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;
        let input = Input {
            next: window.is_key_pressed(Key::Right, KeyRepeat::No),
            previous: window.is_key_pressed(Key::Left, KeyRepeat::No),
            toggle_pause: window.is_key_pressed(Key::Space, KeyRepeat::No),
            reset: window.is_key_pressed(Key::R, KeyRepeat::No),
        };
        show.advance(dt, input)?;
    }
    println!("Ventana slideshow cerrada: {shown} cuadros presentados");
    Ok(())
}
