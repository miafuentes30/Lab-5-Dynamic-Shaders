mod math;
mod scene;
mod renderer;
mod shaders;
mod utils;

use std::time::Instant;

use math::{Vec3, viewport};
use renderer::{Framebuffer, Uniforms, PlanetParams, uniforms::StarParams};
use renderer::pipeline::draw_mesh;
use scene::{load_obj, Camera, Input, Action};
use shaders::noise::NoiseType;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), String> {
    // Ventana 
    let width: u32 = 960;
    let height: u32 = 540;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Lab 05 - Star Shader (Rust) - Parametros Ajustables")
        .with_inner_size(LogicalSize::new(width as f64, height as f64))
        .build(&event_loop)
        .map_err(|e| format!("Window: {e:?}"))?;

    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = Pixels::new(width, height, surface_texture)
        .map_err(|e| format!("Pixels: {e}"))?;

    // Framebuffer
    let mut fb = Framebuffer::new(width as usize, height as usize);

    // Carga esfera
    println!("Cargando esfera...");
    let mesh = load_obj("assets/sphere.obj")?;
    println!("OK Esfera cargada: {} vertices, {} triangulos", 
             mesh.vertices.len(), mesh.indices.len());

    // Camara 
    let mut cam = Camera::default();
    cam.eye = Vec3::new(0.0, 0.0, 3.5); 
    cam.center = Vec3::new(0.0, 0.0, 0.0);
    cam.set_aspect(width as f32 / height as f32);
    
    println!("Camara: eye={:?}, center={:?}", cam.eye, cam.center);

    // Input 
    let mut input = Input::new();

    // Uniforms base
    let mut uniforms = Uniforms {
        time: 0.0,
        light_dir: Vec3::new(0.5, 0.7, 0.2).normalize(),
        view: cam.view(),
        proj: cam.proj(),
        model: math::Mat4::identity(),
        camera_pos: cam.eye,
        planet: PlanetParams::default(),
        star: StarParams::default(),
    };

    // Shader de estrella
    let mut star_shader = crate::shaders::star::Star::default();

    let mut running = true;
    let mut last = Instant::now();
    let mut frame_count = 0;
    let mut saved_screenshot = false;

    print_controls();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    running = false;
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { 
                    input: KeyboardInput { state, virtual_keycode: Some(vk), .. }, .. 
                } => {
                    let is_down = state == ElementState::Pressed;
                    let action_opt = match vk {
                        // Movimiento
                        VirtualKeyCode::W => Some(Action::MoveForward),
                        VirtualKeyCode::S => Some(Action::MoveBackward),
                        VirtualKeyCode::A => Some(Action::MoveLeft),
                        VirtualKeyCode::D => Some(Action::MoveRight),
                        VirtualKeyCode::Space => Some(Action::MoveUp),
                        VirtualKeyCode::LShift => Some(Action::MoveDown),

                        // Rotacion camara
                        VirtualKeyCode::Left  => Some(Action::YawLeft),
                        VirtualKeyCode::Right => Some(Action::YawRight),
                        VirtualKeyCode::Up    => Some(Action::PitchUp),
                        VirtualKeyCode::Down  => Some(Action::PitchDown),

                        // Ajustes de temperatura (Q/E)
                        VirtualKeyCode::Q => Some(Action::TempDec),
                        VirtualKeyCode::E => Some(Action::TempInc),

                        // Ajustes de flares (Z/X)
                        VirtualKeyCode::Z => Some(Action::FlareDec),
                        VirtualKeyCode::X => Some(Action::FlareInc),

                        // Ajustes de noise scale (C/V)
                        VirtualKeyCode::C => Some(Action::NoiseScaleDec),
                        VirtualKeyCode::V => Some(Action::NoiseScaleInc),

                        // Ajustes de velocidad rotacion (B/N)
                        VirtualKeyCode::B => Some(Action::RotSpeedDec),
                        VirtualKeyCode::N => Some(Action::RotSpeedInc),

                        // Tipos de ruido (1/2/3)
                        VirtualKeyCode::Key1 => Some(Action::NoisePerlin),
                        VirtualKeyCode::Key2 => Some(Action::NoiseSimplex),
                        VirtualKeyCode::Key3 => Some(Action::NoiseCellular),
                        VirtualKeyCode::Key4 => Some(Action::ToggleCellularFlares),

                        // Utilidad
                        VirtualKeyCode::P => Some(Action::Screenshot),
                        VirtualKeyCode::H => { if is_down { print_controls(); } None },
                        VirtualKeyCode::Escape => Some(Action::Quit),
                        _ => None,
                    };
                    
                    if let Some(a) = action_opt {
                        if is_down { 
                            input.action_down(a); 
                        } else { 
                            input.action_up(a); 
                        }
                    }
                }
                WindowEvent::Resized(size) => {
                    let _ = pixels.resize_surface(size.width, size.height);
                    let _ = pixels.resize_buffer(size.width, size.height);
                    cam.set_aspect(size.width as f32 / size.height as f32);
                }
                _ => {}
            },

            // Logica y pedir redraw
            Event::MainEventsCleared => {
                if !running { 
                    *control_flow = ControlFlow::Exit; 
                    return; 
                }

                let now = Instant::now();
                let dt = (now - last).as_secs_f32();
                last = now;

                if input.is_pressed(Action::Quit) { 
                    *control_flow = ControlFlow::Exit; 
                    return; 
                }

                // Actualizar parametros de la estrella
                let step = dt * 0.5;
                
                // Temperatura
                if input.is_held(Action::TempInc) { 
                    uniforms.star.temp_norm = (uniforms.star.temp_norm + step).min(1.0);
                    println!("Temperatura: {:.2}", uniforms.star.temp_norm);
                }
                if input.is_held(Action::TempDec) { 
                    uniforms.star.temp_norm = (uniforms.star.temp_norm - step).max(0.0);
                    println!("Temperatura: {:.2}", uniforms.star.temp_norm);
                }
                
                // Flares
                if input.is_held(Action::FlareInc) { 
                    uniforms.star.flare_intensity = (uniforms.star.flare_intensity + step).min(2.0);
                    println!("Flare Intensity: {:.2}", uniforms.star.flare_intensity);
                }
                if input.is_held(Action::FlareDec) { 
                    uniforms.star.flare_intensity = (uniforms.star.flare_intensity - step).max(0.0);
                    println!("Flare Intensity: {:.2}", uniforms.star.flare_intensity);
                }
                
                // Noise Scale
                if input.is_held(Action::NoiseScaleInc) { 
                    uniforms.star.noise_scale = (uniforms.star.noise_scale + step).min(5.0);
                    println!("Noise Scale: {:.2}", uniforms.star.noise_scale);
                }
                if input.is_held(Action::NoiseScaleDec) { 
                    uniforms.star.noise_scale = (uniforms.star.noise_scale - step).max(0.1);
                    println!("Noise Scale: {:.2}", uniforms.star.noise_scale);
                }
                
                // Rotation Speed
                if input.is_held(Action::RotSpeedInc) { 
                    uniforms.star.rot_speed = (uniforms.star.rot_speed + step*0.5).min(2.0);
                    println!("Rotation Speed: {:.2}", uniforms.star.rot_speed);
                }
                if input.is_held(Action::RotSpeedDec) { 
                    uniforms.star.rot_speed = (uniforms.star.rot_speed - step*0.5).max(0.0);
                    println!("Rotation Speed: {:.2}", uniforms.star.rot_speed);
                }
                
                // Tipos de ruido
                if input.is_pressed(Action::NoisePerlin) {
                    uniforms.star.noise_type = NoiseType::Perlin;
                    println!("Noise Type: PERLIN");
                }
                if input.is_pressed(Action::NoiseSimplex) {
                    uniforms.star.noise_type = NoiseType::Simplex;
                    println!("Noise Type: SIMPLEX");
                }
                if input.is_pressed(Action::NoiseCellular) {
                    uniforms.star.noise_type = NoiseType::Cellular;
                    println!("Noise Type: CELLULAR");
                }
                if input.is_pressed(Action::ToggleCellularFlares) {
                    uniforms.star.use_cellular_flares = !uniforms.star.use_cellular_flares;
                    println!("Cellular Flares: {}", if uniforms.star.use_cellular_flares { "ON" } else { "OFF" });
                }

                // Actualizar camara
                update_camera(&mut cam, &input, dt);

                // Actualizar uniforms
                uniforms.time += dt;
                uniforms.view = cam.view();
                uniforms.proj = cam.proj();
                uniforms.camera_pos = cam.eye;

                window.request_redraw();
            }

            // Render 
            Event::RedrawRequested(_) => {
                frame_count += 1;

                let size = window.inner_size();
                let fw = size.width as usize;
                let fh = size.height as usize;

                // Resize framebuffer si es necesario
                if fb.width != fw || fb.height != fh {
                    fb = Framebuffer::new(fw, fh);
                    cam.set_aspect(fw as f32 / fh as f32);
                }

                let vp = viewport(0.0, 0.0, fw as f32, fh as f32, 1.0);

                // Limpiar buffers con fondo negro espacial
                fb.clear_color(renderer::buffers::Color::rgb(2, 2, 5));
                fb.clear_depth();

                // Renderizar estrella centrada con escala apropiada
                let mut u_star = uniforms;
                u_star.model = math::mat::scale(Vec3::new(1.2, 1.2, 1.2));
                
                draw_mesh(&mut fb, &mesh, &mut star_shader, &u_star, vp);

                // Captura de pantalla
                if input.is_pressed(Action::Screenshot) {
                    std::fs::create_dir_all("screenshots").ok();
                    let path = format!("screenshots/star_{:.0}.png", uniforms.time*1000.0);
                    match fb.save_png(&path) {
                        Ok(_) => println!("Screenshot guardado: {}", path),
                        Err(e) => eprintln!("Error guardando screenshot: {}", e),
                    }
                }

                // Auto-screenshot del primer frame
                if !saved_screenshot {
                    std::fs::create_dir_all("screenshots").ok();
                    let path = "screenshots/star_initial.png".to_string();
                    match fb.save_png(&path) {
                        Ok(_) => println!("Screenshot inicial guardado: {}", path),
                        Err(e) => eprintln!("Error guardando screenshot inicial: {}", e),
                    }
                    saved_screenshot = true;
                }

                // Copiar framebuffer a pixels
                let frame = pixels.frame_mut();
                let px_count = (frame.len() / 4).min(fb.color.len());
                for i in 0..px_count {
                    let c = fb.color[i];
                    let o = i * 4;
                    frame[o + 0] = c.r;
                    frame[o + 1] = c.g;
                    frame[o + 2] = c.b;
                    frame[o + 3] = c.a;
                }

                if let Err(e) = pixels.render() {
                    eprintln!("Error en pixels.render: {e}");
                }
                
                input.begin_frame();
            }

            _ => {}
        }
    });
}

fn print_controls() {
    println!("\n=============================================================");
    println!("              CONTROLES DE ESTRELLA                          ");
    println!("=============================================================");
    println!("  CAMARA:");
    println!("    WASD          - Mover camara");
    println!("    Space/Shift   - Subir/Bajar");
    println!("    Flechas       - Rotar camara");
    println!("-------------------------------------------------------------");
    println!("  AJUSTES DE ESTRELLA:");
    println!("    Q/E - Temperatura (color)");
    println!("    Z/X - Intensidad de flares");
    println!("    C/V - Escala de ruido");
    println!("    B/N - Velocidad de rotacion");
    println!("-------------------------------------------------------------");
    println!("  TIPOS DE RUIDO:");
    println!("    1 - Perlin Noise");
    println!("    2 - Simplex Noise");
    println!("    3 - Cellular Noise");
    println!("    4 - Toggle Cellular para Flares");
    println!("-------------------------------------------------------------");
    println!("  UTILIDAD:");
    println!("    P   - Captura de pantalla");
    println!("    H   - Mostrar esta ayuda");
    println!("    Esc - Salir");
    println!("=============================================================\n");
}

// Helpers 
fn update_camera(cam: &mut Camera, input: &Input, dt: f32) {
    let fwd   = (input.is_held(Action::MoveForward) as i32 - input.is_held(Action::MoveBackward) as i32) as f32;
    let right = (input.is_held(Action::MoveRight)   as i32 - input.is_held(Action::MoveLeft)    as i32) as f32;
    let up    = (input.is_held(Action::MoveUp)      as i32 - input.is_held(Action::MoveDown)    as i32) as f32;
    cam.move_free(fwd, right, up, dt);

    let yaw   = (input.is_held(Action::YawRight) as i32 - input.is_held(Action::YawLeft)  as i32) as f32;
    let pitch = (input.is_held(Action::PitchUp)  as i32 - input.is_held(Action::PitchDown)as i32) as f32;
    cam.rotate_free(yaw, pitch, dt);
}