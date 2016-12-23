// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::EventPump;
use self::sdl2::keyboard::KeyboardState;
use self::sdl2::mouse::MouseState;
pub use self::sdl2::keyboard::Scancode;

pub struct Input<'a> {
    pub keyboard: KeyboardState<'a>,

    pub mouse: MouseState,
}

impl<'a> Input<'a> {
    pub fn new(pump: &'a EventPump) -> Self {
        let mouse = pump.mouse_state();

        Input {
            keyboard: KeyboardState::new(pump),

            mouse: mouse,
        }
    }
}
