use std::time::{Duration, Instant};

pub struct FpsCounter {
    last: Instant,
    acc: Duration,
    frames: u32,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self { last: Instant::now(), acc: Duration::from_secs(0), frames: 0 }
    }
    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = now - self.last;
        self.last = now;
        self.acc += dt;
        self.frames += 1;
        if self.acc >= Duration::from_secs(1) {
            let fps = self.frames as f64 / self.acc.as_secs_f64();
            println!("FPS: {:.1}", fps);
            self.acc = Duration::from_secs(0);
            self.frames = 0;
        }
    }
}
