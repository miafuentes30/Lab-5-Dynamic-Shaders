use crate::math::{Vec3, Mat4, look_at_rh, perspective_rh};

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub eye: Vec3,       // posición
    pub center: Vec3,    // punto al que mira
    pub up: Vec3,        // up
    pub fov_y: f32,      // en radianes
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
    pub yaw: f32,        // rad
    pub pitch: f32,      // rad
    pub speed: f32,      // unidades/seg
    pub sens: f32,       // sensibilidad de rotación
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye:    Vec3::new(0.0, 0.0, 3.5),
            center: Vec3::new(0.0, 0.0, 0.0),
            up:     Vec3::new(0.0, 1.0, 0.0),
            fov_y:  60.0_f32.to_radians(),
            aspect: 16.0/9.0,
            znear:  0.01,
            zfar:   100.0,
            yaw:    0.0,
            pitch:  0.0,
            speed:  2.5,
            sens:   1.5,
        }
    }
}

impl Camera {
    pub fn new_pivot(eye: Vec3, target: Vec3, aspect: f32) -> Self {
        let mut c = Self { aspect, ..Default::default() };
        c.eye = eye;
        c.center = target;
        c
    }

    pub fn view(&self) -> Mat4 { look_at_rh(self.eye, self.center, self.up) }
    pub fn proj(&self) -> Mat4 { perspective_rh(self.fov_y, self.aspect, self.znear, self.zfar) }

    /// Control tipo "free-fly"
    pub fn move_free(&mut self, forward: f32, right: f32, up: f32, dt: f32) {
        let dir = self.forward_dir();
        let right_dir = dir.cross(self.up).normalize();
        let up_dir = self.up;

        self.eye   = self.eye
            + dir       * (forward * self.speed * dt)
            + right_dir * (right   * self.speed * dt)
            + up_dir    * (up      * self.speed * dt);
        self.center = self.eye + dir;
    }

    pub fn rotate_free(&mut self, d_yaw: f32, d_pitch: f32, dt: f32) {
        self.yaw   += d_yaw   * self.sens * dt;
        self.pitch += d_pitch * self.sens * dt;
        let limit = 89.0_f32.to_radians();
        if self.pitch > limit { self.pitch = limit; }
        if self.pitch < -limit { self.pitch = -limit; }

        let dir = self.forward_dir();
        self.center = self.eye + dir;
    }

    pub fn orbit_around(&mut self, radius: f32, d_yaw: f32, d_pitch: f32, dt: f32) {
        self.yaw   += d_yaw   * self.sens * dt;
        self.pitch += d_pitch * self.sens * dt;
        let limit = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-limit, limit);
        let x = radius * self.pitch.cos() * self.yaw.cos();
        let y = radius * self.pitch.sin();
        let z = radius * self.pitch.cos() * self.yaw.sin();

        self.eye = self.center + Vec3::new(x, y, z);
    }

    pub fn auto_orbit(&mut self, radius: f32, angular_speed: f32, dt: f32) {
        self.yaw += angular_speed * dt;
        let x = radius * self.pitch.cos() * self.yaw.cos();
        let y = radius * self.pitch.sin();
        let z = radius * self.pitch.cos() * self.yaw.sin();
        self.eye = self.center + Vec3::new(x, y, z);
    }

    pub fn forward_dir(&self) -> Vec3 {
        let cp = self.pitch.cos();
        let x = -self.yaw.sin() * cp;
        let y = self.pitch.sin();
        let z = -self.yaw.cos() * cp;
        Vec3::new(x, y, z).normalize()
    }
    
    pub fn set_aspect(&mut self, aspect: f32) { self.aspect = aspect; }
}