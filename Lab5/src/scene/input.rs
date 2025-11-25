use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Action {
    // Navegación camara
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    YawLeft,
    YawRight,
    PitchUp,
    PitchDown,

    // Toggle/ajustes de shaders
    Shader1,
    Shader2,
    Shader3,
    Shader4,
    Shader5,
    Shader6,
    ToggleRings,
    ToggleMoon,
    PauseRotation,

    // Tuning de estrella
    TempInc,           // Aumentar temperatura
    TempDec,           // Disminuir temperatura
    FlareInc,          // Aumentar intensidad de flares
    FlareDec,          // Disminuir intensidad de flares
    NoiseScaleInc,     // Aumentar escala de ruido
    NoiseScaleDec,     // Disminuir escala de ruido
    RotSpeedInc,       // Aumentar velocidad de rotación
    RotSpeedDec,       // Disminuir velocidad de rotación
    
    // Selección de tipo de ruido
    NoisePerlin,       // Cambiar a Perlin
    NoiseSimplex,      // Cambiar a Simplex
    NoiseCellular,     // Cambiar a Cellular
    ToggleCellularFlares, // Toggle cellular para flares

    // Utilidad
    Screenshot,
    Quit,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ActionState {
    pub held: bool,
    pub pressed: bool,  
    pub released: bool, 
}

#[derive(Default)]
pub struct Input {
    map: HashMap<Action, ActionState>,
}

impl Input {
    pub fn new() -> Self { Self { map: HashMap::new() } }

    /// Llamar cuando una acción cambia a "down"
    pub fn action_down(&mut self, a: Action) {
        let st = self.map.entry(a).or_default();
        if !st.held {
            st.held = true;
            st.pressed = true;
        }
    }

    /// Llamar cuando una acción cambia a "up"
    pub fn action_up(&mut self, a: Action) {
        let st = self.map.entry(a).or_default();
        if st.held {
            st.held = false;
            st.released = true;
        }
    }

    /// Limpia flags "transitorios" al inicio de cada frame
    pub fn begin_frame(&mut self) {
        for (_, st) in self.map.iter_mut() {
            st.pressed = false;
            st.released = false;
        }
    }

    pub fn is_held(&self, a: Action) -> bool {
        self.map.get(&a).map(|s| s.held).unwrap_or(false)
    }
    pub fn is_pressed(&self, a: Action) -> bool {
        self.map.get(&a).map(|s| s.pressed).unwrap_or(false)
    }
    pub fn is_released(&self, a: Action) -> bool {
        self.map.get(&a).map(|s| s.released).unwrap_or(false)
    }
}