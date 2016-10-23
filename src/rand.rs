// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate chrono;
use self::chrono::UTC;

extern crate mersenne_twister;
use self::mersenne_twister::MT19937;

extern crate rand;
use self::rand::{Rng, SeedableRng};

pub struct Rand {
    twister: MT19937,
}

impl Rand {
    pub fn new() -> Self {
        let seed = UTC::now().timestamp() as u32;

        Rand {
            twister: MT19937::from_seed(seed),
        }
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.twister.reseed(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.twister.next_u32()
    }

    fn next_i32(&mut self) -> i32 {
        self.next_u32() as i32
    }

    pub fn next_int(&mut self, n: u32) -> u32 {
        self.next_u32() % n
    }

    pub fn next_int_signed(&mut self, n: i32) -> i32 {
        self.next_i32() % (2 * n + 1) - n
    }

    fn next_real(&mut self) -> f32 {
        ((self.next_u32() as f64) * (1. / 4294967295.)) as f32
    }

    pub fn next_float(&mut self, n: f32) -> f32 {
        self.next_real() % n
    }

    pub fn next_float_signed(&mut self, n: f32) -> f32 {
        self.next_real() % (2. * n) - n
    }
}
