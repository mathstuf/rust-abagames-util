// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2_mixer;
use self::sdl2_mixer::{AudioFormat, Channel, Chunk, Music};

use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::fs;
use std::marker::PhantomData;
use std::mem;
use std::path::{Path, PathBuf};

error_chain! {}

pub struct AudioData {
    path: PathBuf,

    music: HashMap<String, Music>,

    sfx: HashMap<String, (Chunk, Channel)>,
    queued_sfx: HashSet<&'static str>,
}

impl AudioData {
    fn new(asset_dir: &Path) -> Result<Self> {
        let sounds_dir = asset_dir.join("sounds");

        let read_dir = try!(fs::read_dir(sounds_dir.join("musics"))
            .chain_err(|| "failed to list the music directory"));
        let music = try!(read_dir.map(|entry| {
            let entry = try!(entry.chain_err(|| "failed to fetch a directory entry"));
            let music = try!(Music::from_file(&entry.path())
                .map_err(|err| ErrorKind::Msg(format!("failed to read the music file {:?}: {}",
                                                      entry.path(),
                                                      err))));
            let file_name = entry.file_name().to_string_lossy().into_owned();

            Ok((file_name, music))
        })
        .collect::<Result<HashMap<_, _>>>());

        Ok(AudioData {
            path: sounds_dir,

            music: music,

            sfx: HashMap::new(),
            queued_sfx: HashSet::new(),
        })
    }

    fn load_sfx(&mut self, name: &str, channel: isize) -> Result<()> {
        let path = self.path.join("chunks").join(name);
        let chunk = try!(Chunk::from_file(&path));

        self.sfx.insert(name.to_string(), (chunk, sdl2_mixer::channel(channel)));

        Ok(())
    }

    fn play_music(&self, name: &str, count: isize) -> bool {
        self.music
            .get(name)
            .map(|music| music.play(count))
            .is_some()
    }

    fn mark_sfx(&mut self, name: &'static str) -> bool {
        self.queued_sfx.insert(name)
    }

    fn play_sfx(&mut self) -> bool {
        let sfx_to_play = mem::replace(&mut self.queued_sfx, HashSet::new());

        sfx_to_play.iter()
            .map(|&name| {
                self.sfx.get(name)
                    .map(|&(ref sfx, channel)| {
                        channel.play(&sfx, 0)
                    })
                    .is_some()
            })
            .all(|b| b)
    }
}

pub struct Audio<'a> {
    data: AudioData,
    music_enabled: bool,
    sfx_enabled: bool,

    _phantom: PhantomData<&'a str>,
}

static FREQUENCY: isize = 44100;
static FORMAT: AudioFormat = sdl2_mixer::AUDIO_S16;
static CHANNELS: isize = 1;
static BUFFERS: isize = 4096;
static PLAY_UNLIMITED: isize = -1;
static FADE_OUT_TIME: isize = 1280;

impl<'a> Audio<'a> {
    pub fn new(asset_dir: &Path) -> Result<Self> {
        try!(sdl2_mixer::open_audio(FREQUENCY, FORMAT, CHANNELS, BUFFERS));
        sdl2_mixer::allocate_channels(CHANNELS);

        Ok(Audio {
            data: try!(AudioData::new(asset_dir)),
            music_enabled: true,
            sfx_enabled: true,

            _phantom: PhantomData,
        })
    }

    pub fn load_sfx(&mut self, sfx: &[(&str, isize)]) -> Result<&mut Self> {
        try!(sfx.iter()
            .map(|&(ref name, channel)| {
                self.data.load_sfx(name, channel)
            })
            .collect::<Result<Vec<_>>>());

        Ok(self)
    }

    pub fn set_music_enabled(&mut self, enabled: bool) -> &mut Self {
        self.music_enabled = enabled;

        self
    }

    pub fn play_music(&self, name: &str) -> bool {
        if self.music_enabled {
            self.data.play_music(name, PLAY_UNLIMITED)
        } else {
            true
        }
    }

    pub fn play_music_once(&self, name: &str) -> bool {
        if self.music_enabled {
            self.data.play_music(name, 1)
        } else {
            true
        }
    }

    pub fn set_sfx_enabled(&mut self, enabled: bool) -> &mut Self {
        self.sfx_enabled = enabled;

        self
    }

    pub fn mark_sfx(&mut self, name: &'static str) -> bool {
        if self.sfx_enabled {
            self.data.mark_sfx(name)
        } else {
            true
        }
    }

    pub fn play_sfx(&mut self) -> bool {
        if self.sfx_enabled {
            self.data.play_sfx()
        } else {
            true
        }
    }

    pub fn fade(&self) {
        Music::fade_out(FADE_OUT_TIME).unwrap()
    }

    pub fn halt(&self) {
        Music::halt()
    }
}
