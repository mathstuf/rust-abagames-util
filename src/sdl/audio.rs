// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::Sdl;

extern crate sdl2_mixer;
use self::sdl2_mixer::{AudioFormat, Sdl2MixerContext};

use std::error::Error;

pub struct AudioData {
    _sdl_mixer_context: Sdl2MixerContext,
}

impl AudioData {
    fn new(context: Sdl2MixerContext) -> Result<Self, Box<Error>> {
        Ok(AudioData {
            _sdl_mixer_context: context,
        })
    }
}

pub enum Audio {
    Disabled,
    Enabled(AudioData),
}

static FREQUENCY: isize = 44100;
static FORMAT: AudioFormat = sdl2_mixer::AUDIO_S16;
static CHANNELS: isize = 1;
static BUFFERS: isize = 4096;

impl Audio {
    pub fn new(sdl_context: &Sdl, enable: bool) -> Result<Self, Box<Error>> {
        if !enable {
            return Ok(Audio::Disabled);
        }

        try!(sdl_context.audio());

        let context = try!(sdl2_mixer::init(sdl2_mixer::INIT_OGG));
        try!(sdl2_mixer::open_audio(FREQUENCY, FORMAT, CHANNELS, BUFFERS));
        sdl2_mixer::allocate_channels(CHANNELS);

        let data = try!(AudioData::new(context));

        Ok(Audio::Enabled(data))
    }
}
