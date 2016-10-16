// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate nalgebra;
use self::nalgebra::Vector2;

use std::f32::consts::PI;

static PI_2: f32 = 2f32 * PI;

fn normalize(mut rot: f32, half: f32, full: f32) -> f32 {
    if rot < -half {
        rot = full - (-rot % full);
    }

    rot = (rot + half) % full;
    rot - half
}

pub fn normalize_radians(rad: f32) -> f32 {
    normalize(rad, PI, PI_2)
}

pub fn normalize_degrees(deg: f32) -> f32 {
    normalize(deg, 180f32, 360f32)
}

pub fn fast_distance(v1: &Vector2<f32>, v2: &Vector2<f32>) -> f32 {
    let ax = (v1.x - v2.x).abs();
    let ay = (v1.y - v2.y).abs();

    if ax < ay {
        ay + ax / 2f32
    } else {
        ax + ay / 2f32
    }
}

fn between<T>(low: T, expect: T, high: T) -> bool
    where T: PartialOrd,
{
    low <= expect && expect <= high
}

pub fn contains_raw(v1: &Vector2<f32>, x: f32, y: f32, radius: f32) -> bool {
    between(-v1.x * radius, x, v1.x * radius) && between(-v1.y * radius, y, v1.y * radius)
}

pub fn contains(v1: &Vector2<f32>, v2: &Vector2<f32>, radius: f32) -> bool {
    contains_raw(v1, v2.x, v2.y, radius)
}

#[cfg(test)]
mod test {
    extern crate nalgebra;
    use self::nalgebra::ApproxEq;

    use std::f32::consts::PI;

    static PI_2: f32 = 2f32 * PI;

    use super::*;

    #[test]
    fn test_normalize_radians() {
        assert_approx_eq_eps!(normalize_radians(1f32), 1f32, 1e-6);
        assert_approx_eq_eps!(normalize_radians(PI_2), 0f32, 1e-6);
        assert_approx_eq_eps!(normalize_radians(-3f32 * PI), -PI, 1e-6);
    }

    #[test]
    fn test_normalize_degrees() {
        assert_approx_eq_eps!(normalize_degrees(45f32), 45f32, 1e-6);
        assert_approx_eq_eps!(normalize_degrees(360f32), 0f32, 1e-6);
        assert_approx_eq_eps!(normalize_degrees(-540f32), -180f32, 1e-6);
    }
}
