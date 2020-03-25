// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

//! Abagames Utilities
//!
//! These utilities are used by the various games by Kenta Cho.

#![warn(missing_docs)]

mod math;
mod paths;
mod pool;
mod rand;
mod sdl;
mod slice;

pub use crate::rand::*;
pub use math::*;
pub use paths::*;
pub use pool::*;
pub use sdl::*;
pub use slice::*;
