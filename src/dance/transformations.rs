use core::f32;
use std::ops::{Add, BitXor, Mul, Neg};

use glam::{vec2, Vec4};
use itertools::Itertools;
use rand::Rng;

use super::Transformation;

#[derive(Debug, Clone)]
pub struct TransformationGenerator {
    elts: Vec<(RandomFloat, Vec4 /* color */)>,
}

impl TransformationGenerator {
    pub fn new(colors: Vec<Vec4>) -> Self {
        let mut rng = rand::rng();
        Self {
            elts: colors
                .into_iter()
                .map(|color| (RandomFloat::new(rng.random()), color))
                .collect_vec(),
        }
    }

    pub fn generate(&self, t: f32) -> Vec<Transformation> {
        const A: u32 = 0x5765041B;
        const B: u32 = 0x7600A429;
        const C: u32 = 0xFEA3DCC7;
        const D: u32 = 0xF55E8E71;
        const E: u32 = 0x331CDF31;

        let total_scale = cubic_interpolate(|i| self.elts[0].0.random(i, A) * 0.1 + 0.85, t);
        let mut scale_sum = 0.0;

        let mut transformations = self
            .elts
            .iter()
            .map(|(rng, color)| {
                let center =
                    0.5 * cubic_interpolate(|i| vec2(rng.random(i, B), rng.random(i, C)), t);
                let scale = cubic_interpolate(|i| rng.random_positive(i, D), t);
                let angle = cubic_interpolate(|i| rng.random_positive(i, E) * f32::consts::TAU, t);

                scale_sum += scale * scale;

                Transformation {
                    center,
                    scale,
                    angle,
                    color: *color,
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

#[derive(Debug, Clone)]
struct RandomFloat {
    seed: u32,
}

impl RandomFloat {
    fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn random(&self, idx: i32, salt: u32) -> f32 {
        self.random_positive(idx, salt) * 2.0 - 1.0
    }

    fn random_positive(&self, idx: i32, salt: u32) -> f32 {
        // Based on FxHash
        // See <https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html>
        const K: u32 = 0x517c_c1b7;
        let n = (idx as u32)
            .rotate_left(5)
            .bitxor(self.seed)
            .wrapping_mul(K)
            .rotate_left(5)
            .bitxor(salt)
            .wrapping_mul(K) as u16 as f32;
        n / u16::MAX as f32
    }
}
