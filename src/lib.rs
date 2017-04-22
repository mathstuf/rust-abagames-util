// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

//! Abagames Utilities
//!
//! These utilities are used by the various games by Kenta Cho.

#![warn(missing_docs)]

#[macro_use]
extern crate error_chain;

mod crates {
    // public
    pub extern crate cgmath;
    pub extern crate gfx;
    pub extern crate gfx_device_gl;
    pub extern crate gfx_window_sdl;
    pub extern crate sdl2;

    // private
    pub extern crate chrono;
    pub extern crate itertools;
    pub extern crate mersenne_twister;
    pub extern crate rand;
}

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
