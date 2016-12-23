// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::Sdl;

extern crate sdl2_mixer;
use self::sdl2_mixer::Sdl2MixerContext;

use super::paths::Paths;

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

pub struct SdlInfo<'a> {
    pub audio: Option<Audio<'a>>,
    pub video: Video<'a>,
}

pub struct SdlBuilder {
    sdl: Sdl,
    sdl_mixer_context: Option<Sdl2MixerContext>,
    pub paths: Paths,

    audio: bool,

    caption: String,
    size: (u32, u32),
    windowed: bool,
}

impl SdlBuilder {
    pub fn new<C, P>(caption: C, source_path: P) -> Result<Self>
        where C: ToString,
              P: AsRef<Path>,
    {
        Ok(SdlBuilder {
            sdl: try!(sdl2::init()),
            sdl_mixer_context: None,
            paths: try!(Paths::new(source_path)
                .chain_err(|| "failed to set up paths")),

            audio: true,

            caption: caption.to_string(),
            size: (640, 480),
            windowed: false,
        })
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

    pub fn build<'a>(&'a mut self) -> Result<(SdlInfo<'a>, MainLoop<'a>)> {
        let audio = if self.audio {
            try!(self.sdl.audio());
            self.sdl_mixer_context = Some(try!(sdl2_mixer::init(sdl2_mixer::INIT_OGG)));
            Some(try!(Audio::new(&self.paths.asset_dir)))
        } else {
            None
        };

        let mainloop = MainLoop::new(&self.sdl);
        let video = try!(Video::new(&self.sdl, &self.caption, &self.size, self.windowed));

        let info = SdlInfo {
            audio: audio,
            video: video,
        };

        Ok((info, mainloop))
    }
}
