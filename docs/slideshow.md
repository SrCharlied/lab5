# Presentación única — slideshow

## Abrir

Desde la carpeta de `lab5`:

```sh
cargo run --release --locked
```

No es necesario ejecutar un comando por cuerpo. `--slideshow` es un alias explícito de esta presentación.

## Siete vistas

1. Púlsar principal, con giro y pulso.
2. Mundo rocoso de océano-ojo.
3. Gigante gaseoso turbulento.
4. Planeta volcánico.
5. Planeta desértico con su luna.
6. Planeta templado.
7. Estrella de neutrones base, de superficie estricta.

Al terminar vuelve a la primera vista. Se conservan los selectores individuales anteriores y sus controles: por ejemplo, `--pulsar` todavía ofrece su cámara orbital con flechas, mientras que las flechas del slideshow cambian de diapositiva.

## Controles

| Tecla | Acción en la presentación |
|---|---|
| Izquierda / derecha | Anterior / siguiente, con vuelta circular |
| Espacio | Pausa/reanuda tanto el avance automático como la animación del púlsar |
| R | Restablece la primera vista, su animación inicial y el avance automático |
| P | Guarda el framebuffer presentado, antes de aplicar el siguiente cambio |
| Esc | Sale |

El avance automático utiliza tiempo transcurrido, con ocho segundos por vista; no depende de contar cuadros. Un cambio manual reinicia la espera. Se puede seguir navegando mientras está pausado. La animación del púlsar avanza mientras su diapositiva está activa y sin pausa; al regresar conserva su estado, salvo que se pulse R. Los otros cuerpos no adquieren animación ni órbitas por formar parte del slideshow.

El título de la ventana muestra número, nombre y estado AUTO/PAUSA. No se dibujan rótulos encima de los cuerpos ni se mezclan imágenes con transiciones.

## Renderizado, no fotografías precargadas

La presentación reutiliza los shaders y los renderizadores aprobados. La luna conserva sus dos modelos reales y z-buffer compartido con el desértico. Las vistas estáticas solo se vuelven a generar al cambiar de cuerpo; el púlsar reutiliza su renderer durante la animación. No se cargan las capturas PNG para mostrar las diapositivas.

## Capturas y reproducción

Sin `--output`, la captura inicial usa `renders/captures/pulsar-main.png`; las demás usan `slide-02.png` a `slide-07.png` en esa carpeta ignorada. Una ruta `--output archivo.png` explícita se respeta al guardar cualquier diapositiva, por lo que capturas posteriores a esa misma ruta la sustituyen deliberadamente.

`--slide 1..7` selecciona la vista inicial; solo se acepta con la presentación. Puede usarse sin `--slideshow`, ya que ese es el modo por defecto. `--time` fija el tiempo inicial de la animación del púlsar, no adelanta el reloj de la presentación.

Ejemplo reproducible, sin ventana, del desértico con luna desde la presentación:

```sh
cargo run --release --locked -- --slideshow --slide 5 --headless --output renders/tmp/slide-luna.png
```

Las siete capturas de `renders/slideshow/` proceden de esta ruta real. Los tests comparan los cuadros con cada renderer individual y comprueban recorrido automático, navegación, pausa, reinicio, animación y rechazo de entradas inválidas sin alterar el cuadro anterior.

Las pruebas de ventana con cuadros limitados acreditan inicio y presentación en cada vista, no pulsaciones físicas de todas las teclas, fluidez percibida ni un benchmark. Esas comprobaciones de interacción corresponden al usuario.
