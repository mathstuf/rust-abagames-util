// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

use crates::chrono::Utc;
use crates::mersenne_twister::MT19937;
use crates::rand::{Rng, SeedableRng};

/// Seedable and repeatable source of random numbers.
#[derive(Default)]
pub struct Rand {
    twister: MT19937,
}

impl Rand {
    /// Create a new random number source.
    pub fn new() -> Self {
        let seed = Utc::now().timestamp() as u32;

        Rand {
            twister: MT19937::from_seed(seed),
        }
    }

    #[inline]
    /// Set the seed of the source.
    pub fn set_seed(&mut self, seed: u32) {
        self.twister.reseed(seed)
    }

    #[inline]
    /// Get the next 32-bit unsigned integer.
    pub fn next_u32(&mut self) -> u32 {
        self.twister.next_u32()
    }

    #[inline]
    /// Get a 32-bit unsigned integer between 0 and `n`.
    pub fn next_int(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.next_u32() % n
        }
    }

    #[inline]
    /// Get a 32-bit signed integer in the range of `-n` to `n`.
    pub fn next_int_signed(&mut self, n: u32) -> i32 {
        if n == 0 {
            0
        } else {
            ((self.next_u32() % (2 * n + 1)) as i32) - (n as i32)
        }
    }

    #[inline]
    /// Get a real number between 0 and 1.
    fn next_real(&mut self) -> f32 {
        ((self.next_u32() as f64) * (1. / 4294967295.)) as f32
    }

    #[inline]
    /// Get a real number between 0 and `n`.
    pub fn next_float(&mut self, n: f32) -> f32 {
        self.next_real() * n
    }

    #[inline]
    /// Get a real number between `-n` and `n`.
    pub fn next_float_signed(&mut self, n: f32) -> f32 {
        self.next_real() * (2. * n) - n
    }
}

#[cfg(test)]
mod test {
    use crates::chrono::Utc;
    use crates::itertools::{self, Itertools};

    use rand::Rand;

    use std::fmt::Debug;

    fn run_rand<T, F>(closure: F) -> Vec<T>
    where
        F: FnMut() -> T,
    {
        itertools::repeat_call(closure).take(20).collect()
    }

    fn verify_rand<T, F, P>(closure: F, pred: P) -> bool
    where
        F: FnMut() -> T,
        P: Fn(T) -> bool,
        T: Debug,
    {
        itertools::repeat_call(closure)
            .take(20)
            .inspect(|t| print!("{:?}...", t))
            .all(pred)
    }

    #[test]
    fn test_ranges_work() {
        let mut rand = Rand::new();
        let seed = Utc::now().timestamp() as u32;

        println!("seed: {}", seed);
        rand.set_seed(seed);

        (0..100)
            .into_iter()
            .inspect(|n| println!("\nrand.next_int({:?})...", n))
            .foreach(|n| assert!(verify_rand(|| rand.next_int(n), |i| i < n || n == 0)));
        (0..100)
            .into_iter()
            .inspect(|n| println!("\nrand.next_int_signed({:?})...", n))
            .foreach(|n| {
                assert!(verify_rand(
                    || rand.next_int_signed(n),
                    |i| {
                        let n = n as i32;
                        -n <= i && i <= n
                    }
                ))
            });
        (0..100)
            .into_iter()
            .inspect(|_| println!("\nrand.next_real()..."))
            .foreach(|_| assert!(verify_rand(|| rand.next_real(), |f| 0. <= f && f < 1.)));
        (0..100)
            .into_iter()
            .map(|n: usize| n as f32)
            .inspect(|n| println!("\nrand.next_float({:?})...", n))
            .foreach(|n| {
                assert!(verify_rand(
                    || rand.next_float(n),
                    |f| (0. <= f && f < n) || n == 0.
                ))
            });
        (0..100)
            .into_iter()
            .map(|n: usize| n as f32)
            .inspect(|n| println!("\nrand.next_float_signed({:?})...", n))
            .foreach(|n| {
                assert!(verify_rand(
                    || rand.next_float_signed(n),
                    |f| (-n <= f && f < n) || n == 0.
                ))
            });
    }

    #[test]
    fn test_seed_is_deterministic() {
        let mut rand_0 = Rand::new();
        let mut rand_1 = Rand::new();

        rand_0.set_seed(1);
        rand_1.set_seed(1);

        assert_eq!(
            run_rand(|| rand_0.next_u32()),
            run_rand(|| rand_1.next_u32())
        );
        assert_eq!(
            run_rand(|| rand_0.next_int(10)),
            run_rand(|| rand_1.next_int(10))
        );
        assert_eq!(
            run_rand(|| rand_0.next_int(200)),
            run_rand(|| rand_1.next_int(200))
        );
        assert_eq!(
            run_rand(|| rand_0.next_int_signed(200)),
            run_rand(|| rand_1.next_int_signed(200))
        );
        assert_eq!(
            run_rand(|| rand_0.next_real()),
            run_rand(|| rand_1.next_real())
        );
        assert_eq!(
            run_rand(|| rand_0.next_float(0.2)),
            run_rand(|| rand_1.next_float(0.2))
        );
        assert_eq!(
            run_rand(|| rand_0.next_float_signed(-0.4)),
            run_rand(|| rand_1.next_float_signed(-0.4))
        );
    }
}
