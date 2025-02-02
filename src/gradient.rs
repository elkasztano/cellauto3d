use colorgrad::{Color, GradientBuilder, LinearGradient};

pub fn adjustable_spectrum(factor: f32, opposite: f32) -> LinearGradient {
    let factor_clamp = factor.max(0.0).min(1.0);
    let oppo_clamp = opposite.max(0.0).min(1.0);
    GradientBuilder::new()
        .colors(&[
            Color::new(factor_clamp, oppo_clamp, oppo_clamp, 1.0),
            Color::new(factor_clamp, factor_clamp, oppo_clamp, 1.0),
            Color::new(oppo_clamp, factor_clamp, oppo_clamp, 1.0),
            Color::new(oppo_clamp, factor_clamp, factor_clamp, 1.0),
            Color::new(oppo_clamp, oppo_clamp, factor_clamp, 1.0),
            Color::new(factor_clamp, oppo_clamp, factor_clamp, 1.0),
        ])
        .build::<LinearGradient>()
        .unwrap()
}

pub fn adjustable_bw(start: f32, end: f32) -> LinearGradient {
    let sc = start.max(0.0).min(1.0);
    let ec = end.max(0.0).min(1.0);
    GradientBuilder::new()
        .colors(&[Color::new(sc, sc, sc, 1.0), Color::new(ec, ec, ec, 1.0)])
        .build::<LinearGradient>()
        .unwrap()
}

pub fn petrol(factor: f32) -> LinearGradient {
    let fc = factor.max(0.0).min(1.0);
    GradientBuilder::new()
        .colors(&[
            Color::new(0.0 * fc, 0.0 * fc, 0.4 * fc, 1.0),
            Color::new(0.0 * fc, 0.25 * fc, 0.5 * fc, 1.0),
            Color::new(0.1 * fc, 0.5 * fc, 0.6 * fc, 1.0),
        ])
        .build::<LinearGradient>()
        .unwrap()
}
