use core::f32;
use std::ops::{Add, Mul, Neg};

use glam::{Vec2, Vec4};
use itertools::Itertools;

use crate::random::Rng;

use super::Transformation;

#[derive(Debug, Clone)]
pub struct TransformationGenerator {
    elts: Vec<(u32 /* seed */, Vec4 /* color */)>,
}

impl TransformationGenerator {
    pub fn new(colors: Vec<Vec4>) -> Self {
        let mut rng = Rng::new();
        Self {
            elts: colors
                .into_iter()
                .map(|color| (rng.random(), color))
                .collect_vec(),
        }
    }

    pub fn generate(&self, t: f32) -> Vec<Transformation> {
        let total_scale =
            cubic_interpolate(|i| Rng::with_seed(i as u32).random::<f32>() * 0.1 + 0.85, t);
        let mut scale_sum = 0.0;

        let mut transformations = self
            .elts
            .iter()
            .map(|&(seed, color)| {
                let rng = Rng::with_seed(seed);

                let center =
                    cubic_interpolate(|i| rng.clone().hash(i as u32).hash(1).random::<Vec2>(), t)
                        - 0.5;
                let scale = cubic_interpolate(|i| rng.clone().hash(i as u32).hash(2).random(), t);
                let angle = cubic_interpolate(
                    |i| rng.clone().hash(i as u32).hash(3).random::<f32>() * f32::consts::TAU,
                    t,
                );

                scale_sum += scale * scale;

                Transformation {
                    center,
                    scale,
                    angle,
                    color,
                }
            })
            .collect_vec();

        let scale_factor = (total_scale / scale_sum).sqrt();

        for transformation in &mut transformations {
            transformation.scale *= scale_factor;
        }

        transformations
    }
}

fn cubic_interpolate<T>(mut f: impl FnMut(i32) -> T, t: f32) -> T
where
    T: Add<Output = T> + Neg<Output = T> + Mul<f32, Output = T> + Copy,
{
    // based on <https://en.wikipedia.org/wiki/Cubic_Hermite_spline>

    let ti = t as i32;
    let aa = f(ti - 1);
    let a = f(ti);
    let b = f(ti + 1);
    let bb = f(ti + 2);

    let da = (b + -aa) * 0.5;
    let db = (bb + -a) * 0.5;

    let t = t.fract();
    let t2 = t * t;
    let t3 = t2 * t;
    a * (2. * t3 - 3. * t2 + 1.)
        + da * (t3 - 2. * t2 + t)
        + b * (-2. * t3 + 3. * t2)
        + db * (t3 - t2)
}
