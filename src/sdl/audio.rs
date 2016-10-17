// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::Sdl;

extern crate sdl2_mixer;
use self::sdl2_mixer::{AudioFormat, Channel, Chunk, Music, Sdl2MixerContext};

use super::super::paths::Paths;

use std::collections::hash_map::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub struct AudioData {
    _sdl_mixer_context: Sdl2MixerContext,
    path: PathBuf,

    music: HashMap<String, Music>,

    sfx: HashMap<String, (Chunk, Channel)>,
}

impl AudioData {
    fn new(context: Sdl2MixerContext, asset_dir: PathBuf) -> Result<Self, Box<Error>> {
        let sounds_dir = asset_dir.join("sounds");

        let read_dir = try!(fs::read_dir(sounds_dir.join("musics")));
        let music = try!(read_dir.map(|entry| {
            let entry = try!(entry);
            let music = try!(Music::from_file(&entry.path()));
            let file_name = entry.file_name().to_string_lossy().into_owned();

            Ok((file_name, music))
        })
        .collect::<Result<HashMap<_, _>, Box<Error>>>());

        Ok(AudioData {
            _sdl_mixer_context: context,
            path: sounds_dir,

            music: music,

            sfx: HashMap::new(),
        })
    }

    fn load_sfx(&mut self, name: &str, channel: isize) -> Result<(), Box<Error>> {
        let path = self.path.join("chunks").join(name);
        let chunk = try!(Chunk::from_file(&path));

        self.sfx.insert(name.to_string(), (chunk, sdl2_mixer::channel(channel)));

        Ok(())
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

        let asset_dir = try!(Paths::new()).asset_dir;
        let data = try!(AudioData::new(context, asset_dir));

        Ok(Audio::Enabled(data))
    }

    pub fn load_sfx(&mut self, sfx: &[(&str, isize)]) -> Result<&mut Self, Box<Error>> {
        if let Audio::Enabled(ref mut data) = *self {
            try!(sfx.iter()
                .map(|&(ref name, channel)| {
                    data.load_sfx(name, channel)
                })
                .collect::<Result<Vec<_>, _>>());
        }

        Ok(self)
    }
}
