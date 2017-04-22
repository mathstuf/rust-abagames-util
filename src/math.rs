// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

use crates::cgmath::{One, Vector2};

use std::ops::{Add, Sub};

/// Compute a Manhattan distance between two points.
pub fn fast_distance(v1: &Vector2<f32>, v2: &Vector2<f32>) -> f32 {
    let ax = (v1.x - v2.x).abs();
    let ay = (v1.y - v2.y).abs();

    if ax < ay {
        ay + ax / 2.
    } else {
        ax + ay / 2.
    }
}

/// Return `true` if a value is between two bounds (inclusive).
fn between<T>(low: T, expect: T, high: T) -> bool
    where T: PartialOrd,
{
    low <= expect && expect <= high
}

/// Determine whether `v2` is within `radius` units of `v1`.
pub fn contains(v1: &Vector2<f32>, v2: &Vector2<f32>, radius: f32) -> bool {
    between(-v1.x * radius, v2.x, v1.x * radius) && between(-v1.y * radius, v2.y, v1.y * radius)
}

#[inline]
/// Increment a value by one around within a range.
pub fn wrap_inc<T>(value: T, max: T) -> T
    where T: PartialOrd + One + Add<T, Output = T> + Sub<T, Output = T> + Copy,
{
    wrap_inc_by(value, max, T::one())
}

#[inline]
/// Increment a value by a given step around within a range.
pub fn wrap_inc_by<T>(value: T, max: T, step: T) -> T
    where T: PartialOrd + Add<T, Output = T> + Sub<T, Output = T> + Copy,
{
    let new_value = value + step;
    if new_value >= max {
        new_value - max
    } else {
        new_value
    }
}

#[inline]
/// Decrement a value by one around within a range.
pub fn wrap_dec<T>(value: T, max: T) -> T
    where T: PartialOrd + One + Add<T, Output = T> + Sub<T, Output = T> + Copy,
{
    wrap_dec_by(value, max, T::one())
}

#[inline]
/// Decrement a value by a given step around within a range.
pub fn wrap_dec_by<T>(value: T, max: T, step: T) -> T
    where T: PartialOrd + Add<T, Output = T> + Sub<T, Output = T> + Copy,
{
    if value < step {
        value + max - step
    } else {
        value - step
    }
}

#[cfg(test)]
mod test {
    use math::{wrap_dec, wrap_dec_by, wrap_inc, wrap_inc_by};

    #[test]
    fn test_wrap_inc() {
        assert_eq!(wrap_inc(1, 2), 0);
        assert_eq!(wrap_inc(1, 3), 2);
        assert_eq!(wrap_inc(0, 3), 1);
    }

    #[test]
    fn test_wrap_inc_by() {
        assert_eq!(wrap_inc_by(1, 2, 2), 1);
        assert_eq!(wrap_inc_by(1, 3, 2), 0);
        assert_eq!(wrap_inc_by(1, 4, 2), 3);
    }

    #[test]
    fn test_wrap_dec() {
        assert_eq!(wrap_dec(1, 2), 0);
        assert_eq!(wrap_dec(1, 3), 0);
        assert_eq!(wrap_dec(0, 3), 2);
    }

    #[test]
    fn test_wrap_dec_by() {
        assert_eq!(wrap_dec_by(1, 2, 2), 1);
        assert_eq!(wrap_dec_by(1, 3, 2), 2);
        assert_eq!(wrap_dec_by(1, 4, 2), 3);
    }
}
