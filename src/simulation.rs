use bytemuck::{Pod, Zeroable};
use macaw::{Vec2};

pub const DIVISIONS: u32 = 128;
pub const PRISM_SIZE: u32 = 24;
pub const PRISM_STEP: f32 = 1.3;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WavePoint {
    /// Height at current point
    pub value: f32,
    /// How fast this point value is changing
    pub velocity: f32,
    /// Does the wave propagate through this cell
    pub medium: f32,
    /// Extra channel: used to pad out to the 4
    pub extra: f32,
}

impl Default for WavePoint {
    fn default() -> Self {
        Self {
            value: 0.0,
            velocity: 0.0,
            medium: 1.0,
            extra: 1.0,
        }
    }
}

pub struct WaveSimulation {
    divisions: usize,
    pub damping: f32,
    current_state: Vec<WavePoint>,
    previous_state: Vec<WavePoint>,
}

impl WaveSimulation {
    pub fn new(divisions: u32) -> Self {
        let divisions = divisions as usize;
         Self {
            divisions,
            damping: 0.98,
            current_state: vec![WavePoint::default(); divisions * divisions],
            previous_state: vec![WavePoint::default(); divisions * divisions],
        }
    }

    pub fn divisions(&self) -> usize {
        self.divisions
    }

    pub fn set_divisions(&mut self, divisions: usize) {
        self.divisions = divisions;
        self.current_state = vec![WavePoint::default(); divisions * divisions];
        self.previous_state = vec![WavePoint::default(); divisions * divisions];
    }

    pub fn poke_normalized(&mut self, point: Vec2) {
        let clamped = point.clamp(Vec2::ZERO, Vec2::ONE);
        let x = clamped.x * self.divisions as f32;
        let y = clamped.y * self.divisions as f32;
        self.poke(x as usize, y as usize);
    }

    pub fn poke(&mut self, x_start: usize, y_start: usize) {
        for y in y_start..(y_start + 5).min(self.divisions) {
            for x in x_start..(x_start + 5).min(self.divisions){
                let index = y * self.divisions + x;
                if index < self.current_state.len() {
                    self.current_state[index].value += 1.0;
                }
            }
        }
    }

    pub fn advance(&mut self) {
        std::mem::swap(&mut self.current_state, &mut self.previous_state);
        for y in 0..self.divisions {
            for x in 0..self.divisions {
                let index = y * self.divisions + x;
                if self.previous_state[index].medium >= 0.0 {
                    let value = self.previous_state[index].value;
                    let vel = self.previous_state[index].velocity;
                    let mut target = self.previous_state[index].medium;

                    let mut mid = 0.0;
                    if x != 0 {
                        mid += self.get_value(x - 1, y);
                    }
                    if x != self.divisions - 1 {
                        mid += self.get_value(x + 1, y);
                    }
                    if y != 0 {
                        mid += self.get_value(x, y - 1)
                    }
                    if y != self.divisions - 1 {
                        mid += self.get_value(x, y + 1)
                    }
                    mid /= 4.0;

                    target *= 1.5;
                    let new_vel = target * (mid - value) + vel * self.damping;
                    let new_value = value + new_vel;

                    self.current_state[index].value = new_value;
                    self.current_state[index].velocity = new_vel;
                } else {
                    self.current_state[index].value = 0.0;
                    self.current_state[index].velocity = 0.0;
                }
            }
        }
    }

    fn get_value(&self, x: usize, y: usize) -> f32 {
        self.previous_state[y * self.divisions + x].value.max(0.0)
    }

    pub fn current_state(&self) -> (u32, &[u8]) {
        (
            self.divisions as u32,
            bytemuck::cast_slice(self.current_state.as_slice())
        )
    }
}