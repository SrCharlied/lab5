# Verificación de la entrega

## Reproducir

Con Rust/Cargo instalados y Python 3.9 o superior:

```sh
python scripts/verify_delivery.py
```

El primer build puede descargar dependencias; `Cargo.lock` fija sus versiones. Con caché disponible se puede añadir `--offline`. Para comprobar también inicio y presentación de ventanas, añadir `--window` en una sesión gráfica.

El script no depende de respaldos, conversaciones, registros locales, imágenes de referencia privadas ni de UMBRAL. Los logs y renders de comparación se escriben bajo `target/delivery-verification/`; las pruebas también generan PNG temporales bajo `renders/tmp/`. Ambas carpetas están ignoradas por Git. No sobrescribe las imágenes de entrega.

## Comprobaciones

- Integridad de los archivos de código, pruebas, manifiestos Cargo y verificador mediante `docs/source-manifest.json`.
- `cargo fmt --check`.
- `cargo test --locked`: **114 pruebas** en la entrega verificada.
- `cargo clippy --locked --all-targets -- -D warnings`.
- `cargo build --release --locked`.
- Regeneración y comparación SHA-256 de los tres cuerpos, sus cuatro capas, el diagnóstico y la imagen de exhibición del púlsar.
- Integridad del video incluido; reproducir su codificación no es necesario para verificar o ejecutar el laboratorio.

Los argumentos y hashes exactos de las imágenes están en `docs/render-oracles.json`. El resultado de cada ejecución queda en `target/delivery-verification/results.json` junto con los logs individuales. Un fallo detiene la verificación; no se interpreta un log parcial como aprobación.

## Alcance de la suite

Las pruebas cubren framebuffer, profundidad, rasterización, orientación y encuadre, selección real de shaders, capas, costura y polos, campos locales, repetibilidad, entradas inválidas y equivalencia entre PNG decodificado y framebuffer. La estrella animada también tiene pruebas de reloj, pausa, reinicio, cámara y visibilidad del pulso en píxeles codificados.

Una presentación de tres cuadros acredita inicio y salida de la ventana, **no una prueba física de todas las teclas ni un benchmark de fluidez**. El video muestreado del púlsar tampoco mide los FPS de la aplicación.

## Integridad y plataforma

Los hashes de captura se comprobaron en **Windows x64**. Distintas bibliotecas matemáticas, plataformas o toolchains podrían producir diferencias de redondeo; no se promete identidad binaria universal entre plataformas. Las referencias no se regeneran silenciosamente para ocultar diferencias.

Los atributos Git preservan los bytes del código Rust revisado y de los medios, incluyendo algunos archivos heredados con finales de línea CRLF. El repositorio incluye todo lo necesario para compilar; no tiene dependencias Cargo de ruta hacia otros proyectos.

Los registros históricos de desarrollo y copias de respaldo no forman parte del repositorio público. Esta verificación pública es autosuficiente y comprueba el código y las imágenes efectivamente incluidos.
