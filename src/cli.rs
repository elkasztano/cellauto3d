use bevy::prelude::Resource;
use clap::{Parser, ValueEnum};

/// Bevy app for 3D cellular automata with command line interface.
#[derive(Clone, Debug, Parser, Resource)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Population density at start
    #[arg(short, long, default_value_t = 0.1f64)]
    pub density: f64,

    /// Initial seed for the pseudorandom number generator
    #[arg(short, long, default_value_t = 111222333444555)]
    pub seed: u64,

    /// Edge length (defaults to 64 i.e. 64x64x64 cubes, min 16 max 96)
    #[arg(short, long, default_value_t = 64usize)]
    pub edge_length: usize,

    /// Minimum density (min 0.0, max 1.0)
    #[arg(long, default_value_t = 0.025)]
    pub minimum: f64,

    /// Maximum density (min 0.0, max 1.0)
    #[arg(long, default_value_t = 0.25)]
    pub maximum: f64,

    /// Color Style
    #[arg(short, long, default_value = "black-white")]
    pub color_gradient: ColorGradient,

    /// Light Style
    #[arg(short, long, default_value = "normal")]
    pub light_mode: LightMode,

    /// Density of spawned cubes when hitting 'n'
    #[arg(short, long, default_value_t = 0.025f64)]
    pub new_amount: f64,

    /// Fullscreen
    #[arg(long, default_value_t = false)]
    pub fullscreen: bool,

    /// Rules
    #[arg(short, long, default_value = "6-8/7/4/M")]
    pub rules: String,

    /// Core size
    #[arg(short, long = "divisor", default_value_t = 10)]
    pub fraction: usize,

    /// Core density
    #[arg(long, default_value_t = 0.75)]
    pub core_density: f64,
}

#[derive(ValueEnum, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum ColorGradient {
    /// rainbow colors
    Rainbow,

    /// classic black and white
    BlackWhite,

    /// petrol colors
    Petrol,
}

#[derive(ValueEnum, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum LightMode {
    /// normal light
    Normal,

    /// bloom effect
    Bloom,
}
