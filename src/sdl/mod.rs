// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

use crates::cgmath::Vector2;
use crates::sdl2::{self, Sdl};

pub mod error;
pub mod input;
pub mod mainloop;
pub mod video;

pub use self::error::*;
pub use self::input::{Input, Scancode};
pub use self::mainloop::{Event, Game, MainLoop, StepResult};
pub use self::video::{EncoderContext, EncoderDrawContext, Resources, TargetFormat, Video};

/// SDL subsystem structure.
pub struct SdlInfo<'a> {
    /// The video subsystem.
    pub video: Video<'a>,
}

/// A builder object for the SDL context.
pub struct SdlBuilder {
    sdl: Sdl,

    caption: String,
    size: Vector2<u32>,
    windowed: bool,
}

impl SdlBuilder {
    /// Create a new SDL structure.
    pub fn new<C>(caption: C) -> Result<Self>
    where
        C: Into<String>,
    {
        Ok(SdlBuilder {
            sdl: sdl2::init().map_err(ErrorKind::Sdl)?,

            caption: caption.into(),
            size: (640, 480).into(),
            windowed: false,
        })
    }

    /// Resize the window.
    pub fn with_size(&mut self, size: Vector2<u32>) -> &mut Self {
        self.size = size;
        self
    }

    /// Enable or disable windowed mode.
    pub fn windowed_mode(&mut self, windowed: bool) -> &mut Self {
        self.windowed = windowed;
        self
    }

    /// Construct the subsystem structure and the main loop.
    pub fn build(&mut self) -> Result<(SdlInfo, MainLoop)> {
        let mainloop = MainLoop::new(&self.sdl);
        let video = Video::new(&self.sdl, &self.caption, self.size, self.windowed)?;

        let info = SdlInfo {
            video,
        };

        Ok((info, mainloop))
    }
}
