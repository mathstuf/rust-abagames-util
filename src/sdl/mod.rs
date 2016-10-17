// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::Sdl;

use std::error::Error;

mod audio;
mod mainloop;

pub use self::audio::*;
pub use self::mainloop::*;

pub struct SdlInfo {
    _sdl: Sdl,

    pub audio: Audio,
    pub mainloop: MainLoop,
}

pub struct SdlBuilder {
    audio: bool,
}

impl SdlBuilder {
    pub fn new() -> Self {
        SdlBuilder {
            audio: true,
        }
    }

    pub fn with_audio(&mut self, audio: bool) -> &mut Self {
        self.audio = audio;
        self
    }

    pub fn build(self) -> Result<SdlInfo, Box<Error>> {
        let sdl_context = try!(sdl2::init());

        let audio = try!(Audio::new(&sdl_context, self.audio));
        let mainloop = try!(MainLoop::new(&sdl_context));

        Ok(SdlInfo {
            _sdl: sdl_context,

            audio: audio,
            mainloop: mainloop,
        })
    }
}
