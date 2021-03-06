// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

use cgmath::Vector2;
use sdl2::mixer::{self, Sdl2MixerContext};
use sdl2::rwops::RWops;
use sdl2::{self, Sdl};

pub mod audio;
pub mod error;
pub mod input;
pub mod mainloop;
pub mod video;

pub use self::audio::Audio;
pub use self::error::*;
pub use self::input::{Input, Scancode};
pub use self::mainloop::{Event, Game, MainLoop, StepResult};
pub use self::video::{EncoderContext, EncoderDrawContext, Resources, TargetFormat, Video};

/// SDL subsystem structure.
pub struct SdlInfo<'a> {
    /// The audio subsystem.
    pub audio: Option<Audio<'a>>,
    /// The video subsystem.
    pub video: Video<'a>,
}

/// A builder object for the SDL context.
pub struct SdlBuilder<'a> {
    sdl: Sdl,
    sdl_mixer_context: Option<Sdl2MixerContext>,

    audio: bool,
    music_data: Vec<(&'a str, RWops<'a>)>,
    sfx_data: Vec<(&'a str, RWops<'a>, i32)>,

    caption: String,
    size: Vector2<u32>,
    windowed: bool,
}

impl<'a> SdlBuilder<'a> {
    /// Create a new SDL structure.
    pub fn new<C>(caption: C) -> SdlResult<Self>
    where
        C: Into<String>,
    {
        Ok(SdlBuilder {
            sdl: sdl2::init().map_err(SdlError::Sdl)?,
            sdl_mixer_context: None,

            audio: true,
            music_data: Vec::new(),
            sfx_data: Vec::new(),

            caption: caption.into(),
            size: (640, 480).into(),
            windowed: false,
        })
    }

    /// Enable or disable the audio subsystem.
    pub fn with_audio(&mut self, audio: bool) -> &mut Self {
        self.audio = audio;
        self
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

    /// Load audio from data.
    pub fn with_music<M>(&mut self, music: M) -> &mut Self
    where
        M: IntoIterator<Item = &'a (&'a str, &'a [u8])>,
    {
        self.music_data = music
            .into_iter()
            .map(|&(name, data)| (name, RWops::from_bytes(data).unwrap()))
            .collect();
        self
    }

    /// Load audio from data.
    pub fn with_sfx<S>(&mut self, sfx: S) -> &mut Self
    where
        S: IntoIterator<Item = &'a (&'a str, &'a [u8], i32)>,
    {
        self.sfx_data = sfx
            .into_iter()
            .map(|&(name, data, channel)| (name, RWops::from_bytes(data).unwrap(), channel))
            .collect();
        self
    }

    /// Construct the subsystem structure and the main loop.
    pub fn build(&mut self) -> SdlResult<(SdlInfo, MainLoop)> {
        let audio = if self.audio {
            self.sdl.audio().map_err(SdlError::Sdl)?;
            self.sdl_mixer_context =
                Some(mixer::init(mixer::InitFlag::OGG).map_err(SdlError::Sdl)?);
            Some(Audio::new(self.music_data.iter(), self.sfx_data.iter())?)
        } else {
            None
        };

        let mainloop = MainLoop::new(&self.sdl);
        let video = Video::new(&self.sdl, &self.caption, self.size, self.windowed)?;

        let info = SdlInfo {
            audio,
            video,
        };

        Ok((info, mainloop))
    }
}
