// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

use crates::sdl2::{self, Sdl};
use crates::sdl2::mixer::{self, Sdl2MixerContext};

use paths::Paths;

pub mod audio;
pub mod input;
pub mod mainloop;
pub mod video;

use std::path::Path;

pub use self::audio::Audio;
pub use self::input::{Input, Scancode};
pub use self::mainloop::{Event, Game, MainLoop, StepResult};
pub use self::video::{EncoderContext, EncoderDrawContext, Resources, Video};

error_chain! {
    links {
        Audio(audio::Error, audio::ErrorKind)
            #[doc = "errors from the audio subsystem"];
        Mainloop(mainloop::Error, mainloop::ErrorKind)
            #[doc = "errors from the main loop and game itself"];
        Video(video::Error, video::ErrorKind)
            #[doc = "errors from the video subsystem"];
    }
}

/// SDL subsystem structure.
pub struct SdlInfo<'a> {
    /// The audio subsystem.
    pub audio: Option<Audio<'a>>,
    /// The video subsystem.
    pub video: Video<'a>,
}

/// A builder object for the SDL context.
pub struct SdlBuilder {
    sdl: Sdl,
    sdl_mixer_context: Option<Sdl2MixerContext>,
    paths: Paths,

    audio: bool,

    caption: String,
    size: (u32, u32),
    windowed: bool,
}

impl SdlBuilder {
    /// Create a new SDL structure.
    pub fn new<C, P>(caption: C, source_path: P) -> Result<Self>
        where C: ToString,
              P: AsRef<Path>,
    {
        Ok(SdlBuilder {
            sdl: sdl2::init()?,
            sdl_mixer_context: None,
            paths: Paths::new(source_path).chain_err(|| "failed to set up paths")?,

            audio: true,

            caption: caption.to_string(),
            size: (640, 480),
            windowed: false,
        })
    }

    /// Enable or disable the audio subsystem.
    pub fn with_audio(&mut self, audio: bool) -> &mut Self {
        self.audio = audio;
        self
    }

    /// Resize the window.
    pub fn with_size(&mut self, size: (u32, u32)) -> &mut Self {
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
        let audio = if self.audio {
            self.sdl.audio()?;
            self.sdl_mixer_context = Some(mixer::init(mixer::INIT_OGG)?);
            Some(Audio::new(&self.paths.asset_dir)?)
        } else {
            None
        };

        let mainloop = MainLoop::new(&self.sdl);
        let video = Video::new(&self.sdl, &self.caption, &self.size, self.windowed)?;

        let info = SdlInfo {
            audio: audio,
            video: video,
        };

        Ok((info, mainloop))
    }
}
