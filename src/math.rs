// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate cgmath;
use self::cgmath::Vector2;

pub fn fast_distance(v1: &Vector2<f32>, v2: &Vector2<f32>) -> f32 {
    let ax = (v1.x - v2.x).abs();
    let ay = (v1.y - v2.y).abs();

    if ax < ay {
        ay + ax / 2.
    } else {
        ax + ay / 2.
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
