// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate gfx;
use self::gfx::IntoIndexBuffer;

use std::iter;

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

pub fn slice_for_fan<R, F>(factory: &mut F, size: u32) -> gfx::Slice<R>
    where R: gfx::Resources,
          F: gfx::Factory<R>,
{
    let mut indices = Vec::with_capacity((size as usize) * 2 - 2);

    indices.push(0);
    indices.push(1);
    for i in 2..size - 1 {
        indices.push(i);
        indices.push(i);
    }
    indices.push(size - 1);

    gfx::Slice {
        start: 0,
        end: indices.len() as u32,
        base_vertex: 0,
        instances: None,
        buffer: indices.into_index_buffer(factory),
    }
}
