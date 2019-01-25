// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

//! Input subsystem support
//!
//! This module takes all of the input available from the event queue and stores it. This structure
//! is used for storing and reading back replay data.

use crates::sdl2::keyboard::KeyboardState;
pub use crates::sdl2::keyboard::Scancode;
use crates::sdl2::mouse::MouseState;
use crates::sdl2::EventPump;

/// Input snapshot.
pub struct Input<'a> {
    /// The keyboard state.
    pub keyboard: KeyboardState<'a>,

    /// The mouse state.
    pub mouse: MouseState,
}

impl<'a> Input<'a> {
    /// Snapshot the current input from the event queue.
    pub fn new(pump: &'a EventPump) -> Self {
        let mouse = pump.mouse_state();

        Input {
            keyboard: KeyboardState::new(pump),

            mouse,
        }
    }
}
