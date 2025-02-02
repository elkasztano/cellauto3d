use bevy::prelude::{Resource, Timer, TimerMode};
use colorgrad::LinearGradient;
use std::time::Duration;

pub mod cli;
pub mod gradient;
pub mod helptext;
pub mod rules;
pub mod system;
pub mod update;

use crate::system::SystemDims;

// set emission intensity
pub const BLOOM: f32 = 0.8;
// set opacity
pub const ALPHA: f32 = 0.05;
// set overall scale
const FIELD_UNIT: f32 = 0.15625; // 64 x 64 x 64 blocks == 10.0 x 10.0 x 10.0 sized cube
const HALF_UNIT: f32 = FIELD_UNIT / 2.0;
// leave a small space between each cube
pub const CUBE_SIZE: f32 = FIELD_UNIT * 0.9;
// convert system coordinates to world coordinates
pub fn calc_spawn_coords(xyz: (usize, usize, usize), dims: &SystemDims) -> (f32, f32, f32) {
    let half_x = (dims.x() as f32 * FIELD_UNIT) / 2.0;
    let half_y = (dims.y() as f32 * FIELD_UNIT) / 2.0;
    let half_z = (dims.z() as f32 * FIELD_UNIT) / 2.0;
    (
        xyz.0 as f32 * FIELD_UNIT - half_x + HALF_UNIT,
        xyz.1 as f32 * FIELD_UNIT - half_y + HALF_UNIT,
        xyz.2 as f32 * FIELD_UNIT - half_z + HALF_UNIT,
    )
}
// simple helper function for conversion
pub fn isizify3(i: usize, j: usize, k: usize) -> (isize, isize, isize) {
    (i as isize, j as isize, k as isize)
}

#[derive(Resource)]
pub struct SystemTimer {
    pub timer: Timer,
    pub stopped: bool,
}

impl SystemTimer {
    pub fn millis(duration: u64) -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(duration), TimerMode::Repeating),
            stopped: false,
        }
    }
    pub fn increase_micros(&mut self, micros: u64) {
        let duration = self.timer.duration();
        self.timer
            .set_duration(duration.saturating_add(Duration::from_micros(micros)));
    }
    pub fn decrease_micros(&mut self, micros: u64) {
        let duration = self.timer.duration();
        self.timer
            .set_duration(duration.saturating_sub(Duration::from_micros(micros)));
    }
    pub fn toggle_timer(&mut self) {
        self.stopped = !self.stopped;
    }
}

// contains data that are not to be changed once initialized
#[derive(Resource, Clone)]
pub struct GlobalStatic {
    gradient: LinearGradient,
    dims: SystemDims,
    minimum: isize,
    maximum: isize,
    max_amount: usize,
    max_amount_per_thread: usize,
}

impl GlobalStatic {
    pub fn new(gradient: LinearGradient, dims: SystemDims, minimum: isize, maximum: isize) -> Self {
        Self {
            gradient,
            dims,
            minimum,
            maximum,
            max_amount: dims.max_amount(),
            max_amount_per_thread: dims.max_amount()
                / std::thread::available_parallelism()
                    .expect("unable to detect available parallelism")
                    .get(),
        }
    }
    pub fn dims(&self) -> SystemDims {
        self.dims
    }
    pub fn gradient(&self) -> &LinearGradient {
        &self.gradient
    }
    pub fn minimum(&self) -> isize {
        self.minimum
    }
    pub fn maximum(&self) -> isize {
        self.maximum
    }
}

// contains dynamic data of the entire system
#[derive(Resource)]
pub struct GlobalData {
    seed: u64,
    amount: isize,
    growth: bool,
    generation: usize,
}

impl GlobalData {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            amount: 0,
            growth: true,
            generation: 0,
        }
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }
    pub fn increase(&mut self, n: isize) {
        self.amount += n;
    }
    pub fn decrease(&mut self, n: isize) {
        self.amount -= n;
    }
    pub fn amount(&self) -> isize {
        self.amount
    }
    pub fn set_growth(&mut self) {
        self.growth = true;
    }
    pub fn unset_growth(&mut self) {
        self.growth = false;
    }
    pub fn growth(&self) -> bool {
        self.growth
    }
    pub fn generation(&self) -> usize {
        self.generation
    }
    pub fn advance_gen(&mut self) {
        self.generation += 1;
    }
}

// calculate absolute values from given cube size and density
// density saturates at 0.0 and 1.0
pub fn cube_density(edge: usize, density: f64) -> isize {
    (edge.pow(3) as f64 * density.max(0.0).min(1.0)).round() as isize
}

// calculate relative density of cubes
pub fn rel_density(edge: usize, count: isize) -> f64 {
    count as f64 / (edge.pow(3) as f64)
}
