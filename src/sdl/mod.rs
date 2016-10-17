// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::Sdl;

use std::error::Error;

mod audio;
mod mainloop;
mod video;

pub use self::audio::*;
pub use self::mainloop::*;
pub use self::video::*;

pub struct SdlInfo {
    _sdl: Sdl,

    pub audio: Audio,
    pub mainloop: MainLoop,
    pub video: Video,
}

pub struct SdlBuilder {
    audio: bool,

    caption: String,
    size: (u32, u32),
    windowed: bool,
}

impl SdlBuilder {
    pub fn new<C>(caption: C) -> Self
        where C: ToString,
    {
        SdlBuilder {
            audio: true,

            caption: caption.to_string(),
            size: (640, 480),
            windowed: false,
        }
    }

    pub fn with_audio(&mut self, audio: bool) -> &mut Self {
        self.audio = audio;
        self
    }

    pub fn with_size(&mut self, size: (u32, u32)) -> &mut Self {
        self.size = size;
        self
    }

    pub fn windowed_mode(&mut self, windowed: bool) -> &mut Self {
        self.windowed = windowed;
        self
    }

    pub fn build(self) -> Result<SdlInfo, Box<Error>> {
        let sdl_context = try!(sdl2::init());

        let audio = try!(Audio::new(&sdl_context, self.audio));
        let mainloop = try!(MainLoop::new(&sdl_context));
        let video = try!(Video::new(&sdl_context, &self.caption, &self.size, self.windowed));

        Ok(SdlInfo {
            _sdl: sdl_context,

            audio: audio,
            mainloop: mainloop,
            video: video,
        })
    }
}
