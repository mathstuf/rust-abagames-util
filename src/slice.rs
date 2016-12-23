// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate gfx;
use self::gfx::IntoIndexBuffer;

use std::iter;

/// Compute a slice for a line loop of the given size.
pub fn slice_for_loop<R, F>(factory: &mut F, size: u32) -> gfx::Slice<R>
    where R: gfx::Resources,
          F: gfx::Factory<R>,
{
    let indices = (0..size)
        .into_iter()
        .chain(iter::once(0))
        .collect::<Vec<_>>();

    gfx::Slice {
        start: 0,
        end: indices.len() as u32,
        base_vertex: 0,
        instances: None,
        buffer: indices.into_index_buffer(factory),
    }
}

/// Compute a slice for a fan consisting of a number of triangles.
pub fn slice_for_fan<R, F>(factory: &mut F, size: u32) -> gfx::Slice<R>
    where R: gfx::Resources,
          F: gfx::Factory<R>,
{
    let mut indices: Vec<u32> = Vec::with_capacity((size as usize) * 2 - 2);

    for (i, j) in (1..size).zip((2..size)) {
        indices.push(0);
        indices.push(i);
        indices.push(j);
    }

    gfx::Slice {
        start: 0,
        end: indices.len() as u32,
        base_vertex: 0,
        instances: None,
        buffer: indices.into_index_buffer(factory),
    }
}
