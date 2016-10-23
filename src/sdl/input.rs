// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::EventPump;
use self::sdl2::keyboard::KeyboardState;
use self::sdl2::mouse::{MouseState, MouseUtil};
pub use self::sdl2::keyboard::Scancode;

pub struct Input<'a> {
    pub keyboard: KeyboardState<'a>,

    pub mouse: MouseState,
    pub mouse_pos: (i32, i32),
}

impl<'a> Input<'a> {
    pub fn new(pump: &'a EventPump, mouse_util: &MouseUtil) -> Self {
        let (mouse, mouse_x, mouse_y) = mouse_util.mouse_state();

        Input {
            keyboard: KeyboardState::new(pump),

            mouse: mouse,
            mouse_pos: (mouse_x, mouse_y),
        }
    }
}
