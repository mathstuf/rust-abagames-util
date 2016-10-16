// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate chrono;
use self::chrono::UTC;

extern crate mersenne_twister;
use self::mersenne_twister::MT19937;

extern crate rand;
use self::rand::{Rng, SeedableRng};

use std::cell::RefCell;

pub struct Rand {
    twister: RefCell<MT19937>,
}

impl Rand {
    pub fn new() -> Self {
        let seed = UTC::now().timestamp() as u32;

        Rand {
            twister: RefCell::new(MT19937::from_seed(seed)),
        }
    }

    pub fn set_seed(&self, seed: u32) {
        self.twister.borrow_mut().reseed(seed)
    }

    pub fn next_u32(&self) -> u32 {
        self.twister.borrow_mut().next_u32()
    }

    fn next_i32(&self) -> i32 {
        self.next_u32() as i32
    }

    pub fn next_int(&self, n: u32) -> u32 {
        self.next_u32() % n
    }

    pub fn next_int_signed(&self, n: i32) -> i32 {
        self.next_i32() % (2 * n + 1) - n
    }

    fn next_real(&self) -> f32 {
        ((self.next_u32() as f64) * (1f64 / 4294967295f64)) as f32
    }

    pub fn next_float(&self, n: f32) -> f32 {
        self.next_real() % n
    }

    pub fn next_float_signed(&self, n: f32) -> f32 {
        self.next_real() % (2f32 * n) - n
    }
}
