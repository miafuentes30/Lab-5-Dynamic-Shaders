# Estrella 

Representación de una estrella sobre una esfera usando rasterización en CPU (crate `pixels`) y shaders en Rust (`src/shaders`). Todo el color/emisión proviene de ruido procedural + tiempo.

## Demostración
Ingresa al Link:
https://www.canva.com/design/DAG5qPCilKg/TYnNPROSIUHAW3JTAOktWA/edit?utm_content=DAG5qPCilKg&utm_campaign=designshare&utm_medium=link2&utm_source=sharebutton


## Controles (runtime)
Tecla | Acción
------|-------------------------------------------------
W/A/S/D, Space, Shift | Mover cámara / subir / bajar
Flechas               | Rotar cámara
Q / E                 | Bajar / subir temperatura (`temp_norm`)
Z / X                 | Bajar / subir intensidad de flares
C / V                 | Bajar / subir escala de ruido
B / N                 | Bajar / subir velocidad de rotación
1 / 2 / 3             | Seleccionar Perlin / Simplex / Cellular
4                     | Toggle usar Cellular solo para flares
P                     | Screenshot (`screenshots/`)
H                     | Mostrar ayuda en consola
Esc                   | Salir

## Parámetros (`StarParams` en `uniforms.rs`)
`temp_norm` (0..1), `flare_intensity`, `noise_scale`, `rot_speed`, `noise_type` (Perlin/Simplex/Cellular), `use_cellular_flares`.

## Ruido
`noise_3d(p, kind)` elige Perlin/Simplex/Cellular. `fbm_3d_type(p, oct, lac, gain, scale, kind)` combina octavas. Tres capas FBM (baja/media/alta) con offsets temporales diferentes generan intensidad base. Ridge (potencia y abs) produce picos para flare.

## Emisión
`emission = (intensity^1.8 * 0.7 + flare * 0.9).min(2.5)`.
Flare controlado por `flare_intensity` y puede forzar Cellular.

## Distorsión Vertex
Radial scale = `1 + flare_ridge*0.08*flare_intensity + wave`. Wave seno pequeño. Mantiene esfera pero añade protuberancias suaves.

## Color
Gradiente estratificado por intensidad → mezcla hacia blanco cálido según `temp_norm`. Rim glow con función `rim()` y color cálido.

## Compilación
```bash
git clone https://github.com/miafuentes30/Lab-5-Dynamic-Shaders.git
cd Lab-5-Dynamic-Shaders/Lab5
cargo run --release
```


