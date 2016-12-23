// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

//! Abagames Utilities
//!
//! These utilities are used by the various games by Kenta Cho.

#![warn(missing_docs)]

#[macro_use]
extern crate error_chain;

mod math;
mod rand;
mod paths;
mod pool;
mod sdl;
mod slice;

pub use math::*;
pub use rand::*;
pub use paths::*;
pub use pool::*;
pub use sdl::*;
pub use slice::*;
