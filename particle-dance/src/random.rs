use std::{ops::BitXor, sync::OnceLock};

use glam::{Vec2, vec2};

use crate::time::Instant;

#[derive(Debug, Clone)]
pub struct Rng {
    seed: u32,
}

pub trait Random {
    fn random(rng: &mut Rng) -> Self;
}

const K: u32 = 0x517c_c1b7; // (2^32 - 1) / pi
const L: u32 = 0x5e2d_58d8; // (2^32 - 1) / e

impl Rng {
    pub fn new() -> Self {
        static EPOCH: OnceLock<Instant> = OnceLock::new();
        let epoch = *EPOCH.get_or_init(Instant::now);
        Self::with_seed((Instant::now() - epoch).as_nanos() as u32)
    }

    pub fn with_seed(seed: u32) -> Self {
        Self { seed }
    }

    pub fn hash(&mut self, rhs: u32) -> &mut Self {
        self.seed = self.seed.rotate_left(5).bitxor(rhs).wrapping_mul(K);
        self
    }

    pub fn random<T: Random>(&mut self) -> T {
        T::random(self)
    }

    pub fn random_u32(&mut self) -> u32 {
        // based on FxHash <https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html>
        self.hash(L).seed
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

impl Random for u32 {
    fn random(rng: &mut Rng) -> Self {
        rng.random_u32()
    }
}

impl Random for f32 {
    fn random(rng: &mut Rng) -> f32 {
        rng.random_u32() as u16 as f32 / u16::MAX as f32
    }
}

impl Random for Vec2 {
    fn random(rng: &mut Rng) -> Self {
        vec2(rng.random(), rng.random())
    }
}
