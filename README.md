#  Estrella con Shaders Procedurales

Este proyecto implementa un tipo de estrella utilizando shaders dinámicos y diferentes técnicas de ruido procedural en Rust con WGPU. La implementación simula una estrella realista con actividad solar, turbulencias, llamaradas y una corona brillante.

## Demostración

![Estrella](Lab5/assets/star_animation.gif)

## Implementación de Criterios del Laboratorio

### 1. Creatividad Visual y Realism
- **Simulación realista** de actividad solar con múltiples capas de turbulencia
- **Corona solar dinámica** con degradado de colores (amarillo → naranja → rojo)
- **Efectos atmosféricos** basados en scattering de Rayleigh
- **Manchas solares** (zonas oscuras) que aparecen y desaparecen
- **Pulsaciones** que simulan la actividad variable de una estrella real
- **Gradientes de color** que van desde el blanco caliente hasta el rojo profundo

### Complejidad del Shader 

El proyecto implementa **múltiples funciones de ruido** combinadas de forma compleja:

#### Tipos de Ruido Implementados:

**1. Value Noise 3D (`noise3`)**
```wgsl
fn noise3(p: vec3<f32>) -> f32
```
- Interpola valores en una cuadrícula 3D usando smoothstep
- Usado como base para turbulencia general
- **Efecto**: Crea patrones suaves y continuos de variación

**2. Ridge Noise (`ridgenoise`)**
```wgsl
fn ridgenoise(p: vec3<f32>) -> f32 {
    return 1.0 - abs(noise3(p) * 2.0 - 1.0);
}
```
- Crea patrones de crestas y valles pronunciados
- **Efecto**: Simula llamaradas solares y protuberancias

**3. Cellular/Voronoi Noise (`voronoi`)**
```wgsl
fn voronoi(p: vec3<f32>) -> f32
```
- Calcula distancias a puntos aleatorios en celdas 3D
- **Efecto**: Crea patrones celulares que simulan plasma y granulación solar

**4. Fractional Brownian Motion (`fbm` y `ridge_fbm`)**
```wgsl
fn fbm(p: vec3<f32>, octaves: u32) -> f32
fn ridge_fbm(p: vec3<f32>, octaves: u32) -> f32
```
- Combina múltiples octavas de ruido con diferentes frecuencias
- **Efecto**: Agrega detalles multi-escala desde grandes estructuras hasta detalles finos

### Animación Temporal Continua

```rust
// En main.rs
let t = start.elapsed().as_secs_f32();
let cycle = (t * 0.25).fract();  // Ciclo 0..1 para pulsaciones
```

```wgsl
// En el shader
let t = params.time * params.speed;
// Múltiples velocidades de animación
ridge_fbm(p * freq + vec3<f32>(t, t*0.7, -t*0.4), octaves);
fbm(p * freq * 2.0 + vec3<f32>(t*0.3, -t*0.5, t*0.2), octaves);
voronoi(p * freq * 3.0 + vec3<f32>(-t*0.2, t*0.4, t*0.3));
```

**Características:**
- Animación continua y cíclica
- Diferentes velocidades para cada capa de ruido
- Pulsaciones suaves con funciones sinusoidales
- Movimiento orbital de cámara

### Ruido Procedural con Parámetros Ajustables (20 pts)

**Parámetros configurables en `Params`:**

```rust
pub struct Params {
    pub time: f32,           // Tiempo de animación
    pub freq: f32,           // Frecuencia del ruido (4.0)
    pub amp: f32,            // Amplitud de distorsión (0.35)
    pub speed: f32,          // Velocidad de animación (0.5)
    pub octaves: u32,        // Número de octavas FBM (6)
    pub seed: u32,           // Semilla para variación (1337)
    pub temp_kelvin: f32,    // Temperatura en Kelvin (6000.0)
    pub cycle_t: f32,        // Ciclo de pulsación (0..1)
}
```

Todos los parámetros pueden modificarse en tiempo real para ajustar la apariencia.

### Emisión Variable - Luminosidad y Picos de Energía 

**Sistema de intensidad dinámica:**

```wgsl
let core_intensity = 3.5 + 2.0 * pulse;           // Núcleo base pulsante
let flare_intensity = 3.0 * solar_flares;         // Llamaradas solares
let plasma_intensity = 2.5 * plasma_effect * pulse_var;  // Efectos de plasma
let grain_intensity = 1.5 * granulation;          // Granulación de superficie

let total_intensity = core_intensity + flare_intensity + 
                     plasma_intensity + grain_intensity;
```

**Efectos de emisión:**
- Pulsaciones continuas que simulan el ciclo de actividad solar
- Picos de energía localizados (llamaradas) basados en ridge noise
- Variación de intensidad entre 0.0 y 12.0
- Corona más brillante durante los picos de pulsación

### Distorsión Visual en Vertex Shader 

**Desplazamiento radial de vértices:**

```wgsl
@vertex
fn vs_main(in: VSIn) -> VSOut {
    // Múltiples capas de distorsión
    let base_turb = ridge_fbm(...);      // Turbulencia base
    let detail_turb = fbm(...);          // Detalles finos
    let cells = voronoi(...);            // Patrón celular
    
    // Combinación de distorsiones
    let base_distortion = pow(max(0.0, base_turb), 2.0) * 0.8;
    let detail_distortion = (detail_turb - 0.5) * 0.4;
    let cell_distortion = (cells - 0.5) * 0.3;
    
    let total_distortion = params.amp * (base_distortion + 
                                        detail_distortion + 
                                        cell_distortion);
    
    // Desplazar vértices radialmente
    let pos_ws = (1.0 + total_distortion) * in.pos;
}
```

**Resultado:** La superficie de la esfera se deforma dinámicamente, creando protuberancias y depresiones que simulan la actividad solar.

### Control de Color por Intensidad/Temperatura 

**Sistema de gradiente multi-zona:**

```wgsl
// Paleta de colores diferenciada por temperatura
let white_hot = vec3<f32>(1.0, 1.0, 0.9);      // Centro más caliente
let yellow_hot = vec3<f32>(1.0, 0.9, 0.2);     // Zona amarilla
let orange_medium = vec3<f32>(1.0, 0.5, 0.0);  // Zona naranja
let red_cool = vec3<f32>(0.9, 0.2, 0.0);       // Zona roja
let dark_red = vec3<f32>(0.6, 0.1, 0.0);       // Manchas oscuras

// Mezcla basada en intensidad de llamaradas
base_color = mix(base_color, red_cool, cool_zones);
base_color = mix(base_color, orange_medium, medium_zones);
base_color = mix(base_color, yellow_hot, hot_zones);
```

**Conversión Temperatura-Color:**
```wgsl
fn kelvin_to_rgb(temp: f32) -> vec3<f32>
```
- Implementa la conversión precisa de temperatura Kelvin a RGB
- 6000K simula el color del Sol (amarillo-naranja)
- Rango: 1000K (rojo) hasta 40000K (azul)

**Corona con degradado:**
```wgsl
let inner_corona = vec3<f32>(1.0, 0.8, 0.1);   // Amarillo brillante
let mid_corona = vec3<f32>(1.0, 0.4, 0.0);     // Naranja
let outer_corona = vec3<f32>(0.9, 0.1, 0.0);   // Rojo
```

### Documentación  

Este README incluye:
- Descripción de cada tipo de ruido utilizado
- Explicación de cómo el ruido afecta color e intensidad
- Ejemplos de código comentados
- Diagramas de cómo se combinan los efectos
- Lista completa de parámetros ajustables

## Cómo Funciona el Sistema de Ruido

### Flujo de Procesamiento:

```
1. VERTEX SHADER
   ├─ Ridge FBM → Turbulencia base (protuberancias grandes)
   ├─ Value FBM → Detalles finos
   ├─ Voronoi → Patrón celular
   └─ → Desplazamiento radial de vértices

2. FRAGMENT SHADER
   ├─ Ridge FBM → Llamaradas solares (zonas brillantes)
   ├─ Value FBM → Granulación de superficie
   ├─ Voronoi → Efectos de plasma
   │
   ├─ Intensidad total → Suma de todos los efectos
   ├─ Color por zona → Gradiente blanco→amarillo→naranja→rojo
   ├─ Corona → Glow en los bordes con degradado
   └─ Atmósfera → Scattering basado en ángulo de visión
```

### Efecto de Cada Ruido en Color/Intensidad:

| Tipo de Ruido | Dónde se Usa | Efecto en Color | Efecto en Intensidad |
|---------------|--------------|-----------------|---------------------|
| **Ridge FBM** | Llamaradas | Zonas blancas/amarillas | +3.0 en picos |
| **Value FBM** | Granulación | Variación naranja/rojo | +1.5 sutil |
| **Voronoi** | Plasma | Patrón celular amarillo | +2.5 en celdas |
| **Pulsación** | Global | Todos los colores | +2.0 cíclico |

### Combinación de Efectos:

```
Color Final = 
    (Color Base según Intensidad) +
    (Corona según Distancia) +
    (Atmósfera según Ángulo) -
    (Manchas Solares Oscuras)
```

## Características Visuales 

### Colores Vibrantes:
- **Blanco-Amarillo**: Centro super caliente (>85% intensidad)
- **Amarillo Brillante**: Zonas de alta actividad (70-85%)
- **Naranja Intenso**: Zonas de actividad media (40-70%)
- **Rojo Profundo**: Zonas más frías (<40%)
- **Rojo Oscuro**: Manchas solares

### Efectos Dinámicos:
1. **Llamaradas Solares**: Aparecen como líneas brillantes amarillas
2. **Granulación**: Patrón de celdas naranjas en la superficie
3. **Plasma**: Estructuras celulares que fluyen
4. **Corona**: Halo brillante con degradado amarillo→naranja→rojo
5. **Pulsaciones**: Ciclo de 4 segundos de brillo variable
6. **Manchas**: Zonas oscuras rojizas que se mueven

## Tecnologías Utilizadas

- **Rust** - Lenguaje de programación
- **WGPU** - API de gráficos moderno (WebGPU)
- **WGSL** - WebGPU Shading Language
- **Winit** - Manejo de ventanas
- **glam** - Matemáticas de gráficos

## Estructura del Proyecto

```
Lab5/
├── src/
│   ├── main.rs          # Loop principal y configuración
│   ├── renderer.rs      # Inicialización de WGPU y render
│   ├── mesh.rs          # Generación de esfera UV
│   ├── uniforms.rs      # Buffers uniformes
│   └── params.rs        # Estructuras de parámetros
├── shaders/
│   └── star.wgsl        # Shader principal con todos los efectos
├── assets/
│   └── star_animation.gif
└── README.md
```

##  Controles

- La cámara orbita automáticamente alrededor de la estrella
- La animación es continua y no requiere interacción
- Cerrar ventana para salir

## Compilación y Ejecución

```bash
# Clonar el repositorio
git clone https://github.com/miafuentes30/Lab-5-Dynamic-Shaders.git
cd Lab5

# Compilar y ejecutar
cargo run --release
```

## Parámetros Configurables

Para modificar la apariencia, edita los valores en `main.rs`:

```rust
let params = Params {
    freq: 4.0,           // ↑ más detalles, ↓ menos detalles
    amp: 0.35,           // ↑ más distorsión, ↓ más esférico
    speed: 0.5,          // ↑ más rápido, ↓ más lento
    octaves: 6,          // ↑ más detallado, ↓ más suave
    temp_kelvin: 6000.0, // ↑ más azul/blanco, ↓ más rojo
};
```

## Detalles Técnicos 

### Post-Procesamiento:

1. **Tone Mapping**: Reinhard modificado para HDR
2. **Saturación**: Factor 2.5× para colores vibrantes
3. **Gamma Correction**: γ = 0.7 para mayor brillo
4. **Color Grading**: Boost de canales RGB selectivo

### Optimizaciones:

- Uso de `smoothstep` para transiciones suaves
- Cache de cálculos de ruido reutilizables
- Loop optimizado para FBM con early exit
- Vertex shader eficiente con distorsión radial
