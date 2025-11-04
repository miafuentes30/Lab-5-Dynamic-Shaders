// ---------- noise (value noise 3D + fbm) ----------
fn hash3(p: vec3<f32>) -> f32 {
    let h = dot(p, vec3<f32>(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453);
}

fn noise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let n000 = hash3(i + vec3<f32>(0.0,0.0,0.0));
    let n100 = hash3(i + vec3<f32>(1.0,0.0,0.0));
    let n010 = hash3(i + vec3<f32>(0.0,1.0,0.0));
    let n110 = hash3(i + vec3<f32>(1.0,1.0,0.0));
    let n001 = hash3(i + vec3<f32>(0.0,0.0,1.0));
    let n101 = hash3(i + vec3<f32>(1.0,0.0,1.0));
    let n011 = hash3(i + vec3<f32>(0.0,1.0,1.0));
    let n111 = hash3(i + vec3<f32>(1.0,1.0,1.0));

    let u = f * f * (3.0 - 2.0 * f);

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);

    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);

    return mix(nxy0, nxy1, u.z);
}

fn ridgenoise(p: vec3<f32>) -> f32 {
    return 1.0 - abs(noise3(p) * 2.0 - 1.0);
}

fn voronoi(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    var min_dist = 1.0;
    for(var x: i32 = -1; x <= 1; x = x + 1) {
        for(var y: i32 = -1; y <= 1; y = y + 1) {
            for(var z: i32 = -1; z <= 1; z = z + 1) {
                let neighbor = vec3<f32>(f32(x), f32(y), f32(z));
                let point = neighbor + vec3<f32>(hash3(i + neighbor), 
                                               hash3(i + neighbor + vec3<f32>(37.0, 17.0, 53.0)),
                                               hash3(i + neighbor + vec3<f32>(13.0, 71.0, 29.0))) - f;
                min_dist = min(min_dist, dot(point, point));
            }
        }
    }
    return sqrt(min_dist);
}

fn fbm(p: vec3<f32>, octaves: u32) -> f32 {
    var sum = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    var i: u32 = 0u;
    loop {
        if (i >= octaves) { break; }
        sum = sum + amp * noise3(p * freq);
        freq = freq * 2.0;
        amp = amp * 0.5;
        i = i + 1u;
    }
    return sum;
}

fn ridge_fbm(p: vec3<f32>, octaves: u32) -> f32 {
    var sum = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    var i: u32 = 0u;
    loop {
        if (i >= octaves) { break; }
        sum = sum + amp * ridgenoise(p * freq);
        freq = freq * 2.0;
        amp = amp * 0.5;
        i = i + 1u;
    }
    return sum;
}

// ---------- uniforms ----------
struct Camera {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
};

struct Params {
    time: f32,
    freq: f32,
    amp: f32,
    speed: f32,
    octaves: u32,
    seed: u32,
    temp_kelvin: f32,
    cycle_t: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<uniform> camera: Camera;

// ---------- VS/FS IO ----------
struct VSIn {
    @location(0) pos: vec3<f32>,
    @location(1) nrm: vec3<f32>,
};

struct VSOut {
    @builtin(position) pos_cs: vec4<f32>,
    @location(0) nrm_ws: vec3<f32>,
    @location(1) pos_ws: vec3<f32>,
    @location(2) orig_pos: vec3<f32>,
    @location(3) view_dir: vec3<f32>,
};

// ---------- Vertex Shader ----------
@vertex
fn vs_main(in: VSIn) -> VSOut {
    let p = normalize(in.nrm);
    let t = params.time * params.speed;

    // Multicapa de distorsión para simular actividad solar
    let base_turb = ridge_fbm(p * params.freq + vec3<f32>(t, t*0.7, -t*0.4), params.octaves);
    let detail_turb = fbm(p * params.freq * 2.0 + vec3<f32>(t*0.3, -t*0.5, t*0.2), params.octaves);
    let cells = voronoi(p * params.freq * 3.0 + vec3<f32>(-t*0.2, t*0.4, t*0.3));
    
    // Distorsión radial dinámica con múltiples capas
    let base_distortion = pow(max(0.0, base_turb), 2.0) * 0.8;
    let detail_distortion = (detail_turb - 0.5) * 0.4;
    let cell_distortion = (cells - 0.5) * 0.3;
    
    let total_distortion = params.amp * (base_distortion + detail_distortion + cell_distortion);
    
    // Posición final con distorsión
    let pos_ws = (1.0 + total_distortion) * in.pos;
    let pos_cs = camera.view_proj * vec4<f32>(pos_ws, 1.0);

    var out: VSOut;
    out.pos_cs = pos_cs;
    out.nrm_ws = normalize(in.nrm);
    out.pos_ws = pos_ws;
    out.orig_pos = in.pos;
    out.view_dir = normalize(camera.camera_pos - pos_ws);
    return out;
}

// ---------- util color ----------
fn kelvin_to_rgb(temp: f32) -> vec3<f32> {
    var temperature = clamp(temp, 1000.0, 40000.0);
    temperature = temperature / 100.0;
    
    var color = vec3<f32>(1.0);
    
    // Rojo
    if(temperature <= 66.0) {
        color.r = 1.0;
    } else {
        color.r = clamp(329.698727446 * pow(temperature - 60.0, -0.1332047592), 0.0, 1.0) / 255.0;
    }
    
    // Verde
    if(temperature <= 66.0) {
        color.g = clamp(99.4708025861 * log(temperature) - 161.1195681661, 0.0, 255.0) / 255.0;
    } else {
        color.g = clamp(288.1221695283 * pow(temperature - 60.0, -0.0755148492), 0.0, 255.0) / 255.0;
    }
    
    // Azul
    if(temperature >= 66.0) {
        color.b = 1.0;
    } else if(temperature <= 19.0) {
        color.b = 0.0;
    } else {
        color.b = clamp(138.5177312231 * log(temperature - 10.0) - 305.0447927307, 0.0, 255.0) / 255.0;
    }
    
    return color;
}

fn atmospheric_scattering(view_dir: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let fresnel = pow(max(0.0, 1.0 - dot(view_dir, normal)), 2.5);
    return vec3<f32>(1.0, 0.4, 0.0) * fresnel * 3.5;
}

// ---------- Fragment Shader ----------
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let t = params.time * params.speed;
    
    // Turbulencia compleja con múltiples capas
    let base_turb = ridge_fbm(in.nrm_ws * params.freq + vec3<f32>(t, t*0.7, -t*0.4), params.octaves);
    let detail_turb = fbm(in.nrm_ws * params.freq * 2.0 + vec3<f32>(t*0.3, -t*0.5, t*0.2), params.octaves);
    let plasma = voronoi(in.nrm_ws * params.freq * 3.0 + vec3<f32>(-t*0.2, t*0.4, t*0.3));
    
    // Pulsación dinámica con variaciones
    let pulse = 0.5 + 0.5 * sin(6.28318 * params.cycle_t);
    let pulse_var = 0.5 + 0.3 * sin(6.28318 * params.cycle_t * 1.3);
    
    // Efectos solares mejorados
    let solar_flares = pow(max(0.0, base_turb * 1.5), 2.5);
    let plasma_effect = pow(max(0.0, 1.0 - plasma), 2.0);
    let granulation = pow(max(0.0, detail_turb), 1.8);
    
    // ZONAS DE COLOR DIFERENCIADAS
    // Zona 1: Centro super caliente (blanco-amarillo)
    let hot_zones = smoothstep(0.7, 1.0, solar_flares);
    
    // Zona 2: Media (amarillo-naranja)
    let medium_zones = smoothstep(0.4, 0.7, solar_flares);
    
    // Zona 3: Más fría (naranja-rojo)
    let cool_zones = smoothstep(0.0, 0.4, solar_flares);
    
    // Paleta de colores VIBRANTES
    let white_hot = vec3<f32>(1.0, 1.0, 0.9);        // Blanco caliente
    let yellow_hot = vec3<f32>(1.0, 0.9, 0.2);       // Amarillo brillante
    let orange_medium = vec3<f32>(1.0, 0.5, 0.0);    // Naranja intenso
    let red_cool = vec3<f32>(0.9, 0.2, 0.0);         // Rojo profundo
    let dark_red = vec3<f32>(0.6, 0.1, 0.0);         // Rojo oscuro
    
    // Mezclar colores según las zonas
    var base_color = dark_red;
    base_color = mix(base_color, red_cool, cool_zones);
    base_color = mix(base_color, orange_medium, medium_zones);
    base_color = mix(base_color, yellow_hot, hot_zones);
    base_color = mix(base_color, white_hot, smoothstep(0.85, 1.0, solar_flares));
    
    // Añadir variación de color por plasma
    let plasma_color = mix(
        vec3<f32>(1.0, 0.3, 0.0),  // Naranja
        vec3<f32>(1.0, 0.7, 0.2),  // Amarillo-naranja
        plasma_effect
    );
    base_color = mix(base_color, plasma_color, plasma_effect * 0.4);
    
    // Intensidad total MUY AUMENTADA
    let core_intensity = 3.5 + 2.0 * pulse;
    let flare_intensity = 3.0 * solar_flares;
    let plasma_intensity = 2.5 * plasma_effect * pulse_var;
    let grain_intensity = 1.5 * granulation;
    
    let total_intensity = clamp(
        core_intensity + 
        flare_intensity + 
        plasma_intensity + 
        grain_intensity, 
        0.0, 12.0
    );
    
    // Aplicar intensidad con saturación extra
    var core_color = base_color * total_intensity;
    
    // Efecto de corona ULTRA BRILLANTE en los bordes
    let dist_from_center = length(in.pos_ws);
    let edge_factor = smoothstep(0.8, 1.1, dist_from_center);
    let corona_glow = pow(edge_factor, 1.5) * (4.0 + 2.0 * pulse);
    
    // Corona con degradado de colores calientes
    let inner_corona = vec3<f32>(1.0, 0.8, 0.1);     // Amarillo brillante
    let mid_corona = vec3<f32>(1.0, 0.4, 0.0);       // Naranja fuego
    let outer_corona = vec3<f32>(0.9, 0.1, 0.0);     // Rojo intenso
    
    var corona_color = mix(inner_corona, mid_corona, smoothstep(0.8, 1.0, edge_factor));
    corona_color = mix(corona_color, outer_corona, smoothstep(1.0, 1.1, edge_factor));
    corona_color = corona_color * corona_glow;
    
    // Efecto atmosférico MÁS INTENSO
    let atmosphere = atmospheric_scattering(in.view_dir, in.nrm_ws);
    let atmosphere_color = atmosphere * (2.5 + 1.5 * pulse);
    
    // Agregar manchas solares (zonas más oscuras y rojizas)
    let dark_spots = smoothstep(0.65, 0.85, plasma) * 0.5;
    let spot_color = vec3<f32>(0.3, 0.05, 0.0);
    core_color = mix(core_color, spot_color, dark_spots);
    
    // Combinar todos los efectos
    var final_color = core_color + corona_color + atmosphere_color;
    
    // Tone mapping AJUSTADO para colores más saturados
    let exposure = 0.5;  // Menos exposición para colores más puros
    let exposed = final_color * exposure;
    
    // Reinhard tone mapping modificado
    let tone_mapped = exposed / (exposed + vec3<f32>(0.8));
    
    // SUPER saturación
    let luma = dot(tone_mapped, vec3<f32>(0.2126, 0.7152, 0.0722));
    let saturated = mix(vec3<f32>(luma), tone_mapped, 2.5);  // Saturación extrema
    
    // Ajuste de gamma para colores vibrantes
    let gamma_corrected = pow(saturated, vec3<f32>(0.7));
    
    // Boost final de colores cálidos
    var final_output = gamma_corrected;
    final_output.r = pow(final_output.r, 0.9);  // Más rojo
    final_output.g = pow(final_output.g, 1.0);  // Verde normal
    final_output.b = pow(final_output.b, 1.3);  // Menos azul
    
    return vec4<f32>(final_output, 1.0);
}