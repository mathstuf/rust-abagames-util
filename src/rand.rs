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

    pub fn next_int(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.next_u32() % n
        }
    }

    pub fn next_int_signed(&mut self, n: u32) -> i32 {
        if n == 0 {
            0
        } else {
            ((self.next_u32() % (2 * n + 1)) as i32) - (n as i32)
        }
    }

    fn next_real(&mut self) -> f32 {
        ((self.next_u32() as f64) * (1. / 4294967295.)) as f32
    }

    pub fn next_float(&mut self, n: f32) -> f32 {
        if n == 0. {
            0.
        } else {
            self.next_real() % n
        }
    }

    pub fn next_float_signed(&mut self, n: f32) -> f32 {
        if n == 0. {
            0.
        } else {
            self.next_real() % (2. * n) - n
        }
    }
}

#[cfg(test)]
mod test {
    extern crate itertools;

    use super::Rand;

    fn run_rand<T, F>(closure: F) -> Vec<T>
        where F: FnMut() -> T,
    {
        itertools::repeat_call(closure)
            .take(20)
            .collect()
    }

    #[test]
    fn test_seed_is_deterministic() {
        let mut rand_0 = Rand::new();
        let mut rand_1 = Rand::new();

        rand_0.set_seed(1);
        rand_1.set_seed(1);

        assert_eq!(run_rand(|| rand_0.next_u32()),
                   run_rand(|| rand_1.next_u32()));
        assert_eq!(run_rand(|| rand_0.next_int(10)),
                   run_rand(|| rand_1.next_int(10)));
        assert_eq!(run_rand(|| rand_0.next_int(200)),
                   run_rand(|| rand_1.next_int(200)));
        assert_eq!(run_rand(|| rand_0.next_int_signed(200)),
                   run_rand(|| rand_1.next_int_signed(200)));
        assert_eq!(run_rand(|| rand_0.next_real()),
                   run_rand(|| rand_1.next_real()));
        assert_eq!(run_rand(|| rand_0.next_float(0.2)),
                   run_rand(|| rand_1.next_float(0.2)));
        assert_eq!(run_rand(|| rand_0.next_float_signed(-0.4)),
                   run_rand(|| rand_1.next_float_signed(-0.4)));
    }
}
