mod live_app;
mod pulsar_app;
use lab5::{render_diagnostic, render_neutron, renderer::image::save_png};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::path::PathBuf;

const HELP: &str = "Lab 5 — cuerpos celestes por software\n\
Uso: cargo run --release -- [--neutron | --neutron-live | --rocky | --eyeball | --pulsar | --gas | --gas-v2] [--time S] [--layers 1..4] [--headless] [--output ruta.png] [--width N] [--height N] [--angle RAD] [--frames N]\n\
Ventana: Esc salir; P guardar PNG.\n\
--frames N cierra la ventana despues de N cuadros (N > 0).\n\
--angle fija la rotacion Y inicial en radianes; --neutron es estatico, --neutron-live gira.\n\
--neutron, --rocky o --eyeball activa una vista estatica; --layers muestra capas acumulativas (4 por defecto).\n\
--eyeball selecciona el mundo de hielo con oceano-ojo; --rocky conserva la candidata cobriza.\n\
--gas selecciona la candidata estatica del gigante azul HD 189733 b; --layers muestra sus cuatro capas.\n\
--gas-v2 selecciona la variante turbulenta estatica; --gas conserva las bandas V1.\n\
--neutron-live activa giro, pulsacion y camara; --time S fija su tiempo inicial.\n\
--pulsar es una exhibicion separada: superficie, arcos, haces y corona; --time fija su tiempo y --layers sus etapas acumulativas.\n\
LIVE: Flechas orbitar; W/S acercar/alejar; Espacio pausa; R reinicia; P PNG; Esc salir.\n";

fn run() -> Result<(), String> {
    let mut width = 960_usize;
    let mut height = 720_usize;
    let mut angle = 0.0_f64;
    let mut explicit_angle = false;
    let mut live = false;
    let mut time = 0.0_f64;
    let mut explicit_time = false;
    let mut headless = false;
    let mut neutron = false;
    let mut rocky = false;
    let mut eyeball = false;
    let mut pulsar = false;
    let mut gas = false;
    let mut gas_v2 = false;
    let mut layers = 4_u8;
    let mut explicit_layers = false;
    let mut explicit_output = false;
    let mut frames = None;
    let mut output = PathBuf::from("renders/diagnostic.png");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            print!("{HELP}");
            return Ok(());
        }
        if arg == "--headless" {
            headless = true;
            continue;
        }
        if arg == "--neutron" {
            neutron = true;
            continue;
        }
        if arg == "--neutron-live" {
            live = true;
            continue;
        }
        if arg == "--rocky" {
            rocky = true;
            continue;
        }
        if arg == "--eyeball" {
            eyeball = true;
            continue;
        }
        if arg == "--pulsar" {
            pulsar = true;
            continue;
        }
        if arg == "--gas" {
            gas = true;
            continue;
        }
        if arg == "--gas-v2" {
            gas_v2 = true;
            continue;
        }
        if ![
            "--width", "--height", "--angle", "--frames", "--output", "--layers", "--time",
        ]
        .contains(&arg.as_str())
        {
            return Err(format!("Opcion desconocida: {arg}"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("Falta valor para {arg}"))?;
        match arg.as_str() {
            "--width" => width = value.parse().map_err(|_| "Ancho invalido")?,
            "--height" => height = value.parse().map_err(|_| "Alto invalido")?,
            "--angle" => {
                angle = value.parse().map_err(|_| "Angulo invalido")?;
                explicit_angle = true;
            }
            "--time" => {
                time = value.parse().map_err(|_| "Tiempo invalido")?;
                explicit_time = true;
            }
            "--frames" => {
                let count = value
                    .parse::<usize>()
                    .map_err(|_| "Numero de cuadros invalido")?;
                if count == 0 {
                    return Err("--frames debe ser mayor que cero".into());
                }
                frames = Some(count);
            }
            "--output" => {
                output = PathBuf::from(value);
                explicit_output = true;
            }
            "--layers" => {
                layers = value.parse().map_err(|_| "Capas invalidas")?;
                explicit_layers = true;
            }
            _ => unreachable!(),
        }
    }
    if u8::from(live)
        + u8::from(neutron)
        + u8::from(rocky)
        + u8::from(eyeball)
        + u8::from(pulsar)
        + u8::from(gas)
        + u8::from(gas_v2)
        > 1
    {
        return Err(
            "Escoge un solo modo: --neutron, --neutron-live, --rocky, --eyeball, --pulsar, --gas o --gas-v2"
                .into(),
        );
    }
    if explicit_time && !live && !pulsar {
        return Err("--time requiere --neutron-live".into());
    }
    if explicit_layers && !neutron && !live && !rocky && !eyeball && !pulsar && !gas && !gas_v2 {
        return Err("--layers requiere un cuerpo celeste".into());
    }
    if neutron && !explicit_output {
        output = PathBuf::from("renders/neutron-v1.png");
    }
    if live && !explicit_output {
        output = PathBuf::from("renders/neutron-live.png");
    }
    if rocky && !explicit_output {
        output = PathBuf::from("renders/rocky-v1.png");
    }
    if eyeball && !explicit_output {
        output = PathBuf::from("renders/eyeball-v1.png");
    }
    if pulsar && !explicit_output {
        output = PathBuf::from("renders/pulsar-v1.png");
    }
    if gas && !explicit_output {
        output = PathBuf::from("renders/gas-v1.png");
    }
    if gas_v2 && !explicit_output {
        output = PathBuf::from("renders/gas-v2.png");
    }
    if output.extension().and_then(|s| s.to_str()) != Some("png") {
        return Err("--output debe terminar en .png".into());
    }
    if pulsar {
        return pulsar_app::run(live_app::Options {
            width,
            height,
            angle,
            time,
            layers,
            headless,
            output,
            frames,
        });
    }
    if live {
        return live_app::run(live_app::Options {
            width,
            height,
            angle: if explicit_angle { angle } else { -0.65 },
            time,
            layers,
            headless,
            output,
            frames,
        });
    }
    let (framebuffer, stats) = if gas_v2 {
        lab5::gas_v2::render(width, height, angle, layers)?
    } else if gas {
        lab5::gas::render(width, height, angle, layers)?
    } else if eyeball {
        lab5::eyeball::render(width, height, angle, layers)?
    } else if rocky {
        lab5::rocky::render(width, height, angle, layers)?
    } else if neutron {
        render_neutron(width, height, angle, layers)?
    } else {
        render_diagnostic(width, height, angle)?
    };
    println!(
        "{}x{} | {} triangulos rasterizados | {} pixeles escritos",
        width, height, stats.triangles_rasterized, stats.pixels_written
    );
    if headless {
        save_png(&output, &framebuffer)?;
        println!("PNG: {}", output.display());
        return Ok(());
    }
    let mut window = Window::new(
        if gas_v2 {
            "Lab 5 | gigante azul V2 turbulento (candidata) | P captura | Esc salir"
        } else if gas {
            "Lab 5 | gigante azul HD 189733 b (candidata artistica) | P captura | Esc salir"
        } else if eyeball {
            "Lab 5 | LHS 1140 b: oceano-ojo (candidata artistica) | P captura | Esc salir"
        } else if rocky {
            "Lab 5 | rocoso LHS 1140 b (interpretacion candidata) | P captura | Esc salir"
        } else if neutron {
            "Lab 5 | estrella de neutrones v1 (candidata) | P captura | Esc salir"
        } else {
            "Lab 5 | diagnostico de normales | P captura | Esc salir"
        },
        width,
        height,
        WindowOptions::default(),
    )
    .map_err(|e| format!("No se pudo abrir la ventana: {e}"))?;
    window.set_target_fps(60);
    let mut shown = 0_usize;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&framebuffer.pixels, width, height)
            .map_err(|e| e.to_string())?;
        shown += 1;
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            save_png(&output, &framebuffer)?;
            println!("PNG: {}", output.display());
        }
        if frames.is_some_and(|limit| shown >= limit) {
            break;
        }
    }
    println!("Ventana cerrada: {shown} cuadros presentados");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
