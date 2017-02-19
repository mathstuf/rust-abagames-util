// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

//! Audio subsystem support
//!
//! This module contains utilities to assist in loading any playing audio including background
//! music and sound effects.

extern crate sdl2;
use self::sdl2::mixer::{self, AudioFormat, Channel, Chunk, Music};

use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::fs;
use std::marker::PhantomData;
use std::mem;
use std::path::{Path, PathBuf};

error_chain! {}

/// Audio data information and management.
struct AudioData {
    /// The path to the audio data.
    path: PathBuf,

    /// Music files.
    music: HashMap<String, Music>,

    /// Sound effect files.
    sfx: HashMap<String, (Chunk, Channel)>,
    /// Sound effects queued for playing.
    queued_sfx: HashSet<&'static str>,
}

impl AudioData {
    /// Discover audio data in a directory.
    fn new(asset_dir: &Path) -> Result<Self> {
        let sounds_dir = asset_dir.join("sounds");

        let read_dir = fs::read_dir(sounds_dir.join("musics"))
            .chain_err(|| "failed to list the music directory")?;
        let music = read_dir.map(|entry| {
            let entry = entry.chain_err(|| "failed to fetch a directory entry")?;
            let music = Music::from_file(&entry.path()).map_err(|err| {
                ErrorKind::Msg(format!("failed to read the music file {}: {:?}",
                                       entry.path().to_string_lossy(),
                                       err))
            })?;
            let file_name = entry.file_name().to_string_lossy().into_owned();

            Ok((file_name, music))
        })
        .collect::<Result<HashMap<_, _>>>()?;

        Ok(AudioData {
            path: sounds_dir,

            music: music,

            sfx: HashMap::new(),
            queued_sfx: HashSet::new(),
        })
    }

    /// Load a sound effect.
    fn load_sfx(&mut self, name: &str, channel: i32) -> Result<()> {
        let path = self.path.join("chunks").join(name);
        let chunk = Chunk::from_file(&path)?;

        self.sfx.insert(name.to_string(), (chunk, mixer::channel(channel)));

        Ok(())
    }

    /// Play a music file.
    fn play_music(&self, name: &str, count: i32) -> bool {
        self.music
            .get(name)
            .map(|music| music.play(count))
            .is_some()
    }

    /// Mark a sound effect for playing when requested.
    fn mark_sfx(&mut self, name: &'static str) -> bool {
        self.queued_sfx.insert(name)
    }

    /// Play queued sound effects.
    fn play_sfx(&mut self) -> bool {
        let sfx_to_play = mem::replace(&mut self.queued_sfx, HashSet::new());

        sfx_to_play.iter()
            .map(|&name| {
                self.sfx
                    .get(name)
                    .map(|&(ref sfx, channel)| channel.play(sfx, 0))
                    .is_some()
            })
            .all(|b| b)
    }
}

/// Audio support.
pub struct Audio<'a> {
    /// Audio data.
    data: AudioData,
    /// Whether music is enabled or not.
    music_enabled: bool,
    /// Whether sound effects is enabled or not.
    sfx_enabled: bool,

    #[doc(hidden)]
    _phantom: PhantomData<&'a str>,
}

/// The frequency to play audio at.
static FREQUENCY: i32 = 44100;
/// The format of the audio.
static FORMAT: AudioFormat = mixer::AUDIO_S16;
/// The number of channels to play.
static CHANNELS: i32 = 1;
/// The size of the audio buffers.
static BUFFERS: i32 = 4096;
/// The number of times to repeat audio infinitely.
static PLAY_UNLIMITED: i32 = -1;
/// The amount of time, in milliseconds, over which to fade out music.
static FADE_OUT_TIME: i32 = 1280;

impl<'a> Audio<'a> {
    /// Load all audio files from a directory.
    ///
    /// Sound effects are loaded from the `sounds/chunks` subdirectory and music from the
    /// `sounds/musics` subdirectory.
    pub fn new(asset_dir: &Path) -> Result<Self> {
        mixer::open_audio(FREQUENCY, FORMAT, CHANNELS, BUFFERS)?;
        mixer::allocate_channels(CHANNELS);

        Ok(Audio {
            data: AudioData::new(asset_dir)?,
            music_enabled: true,
            sfx_enabled: true,

            _phantom: PhantomData,
        })
    }

    /// Load sound effects for playing on specific channels.
    pub fn load_sfx<I, N>(&mut self, sfx: I) -> Result<&mut Self>
        where I: Iterator<Item = (N, i32)>,
              N: AsRef<str>,
    {
        sfx.map(|(ref name, channel)| self.data.load_sfx(name.as_ref(), channel))
            .collect::<Result<Vec<_>>>()?;

        Ok(self)
    }

    /// Set whether music is enabled or not.
    pub fn set_music_enabled(&mut self, enabled: bool) -> &mut Self {
        self.music_enabled = enabled;

        self
    }

    /// Play the named music file in a loop.
    pub fn play_music(&self, name: &str) -> bool {
        if self.music_enabled {
            self.data.play_music(name, PLAY_UNLIMITED)
        } else {
            true
        }
    }

    /// Play the named music file.
    pub fn play_music_once(&self, name: &str) -> bool {
        if self.music_enabled {
            self.data.play_music(name, 1)
        } else {
            true
        }
    }

    /// Set whether sound effects are enabled or not.
    pub fn set_sfx_enabled(&mut self, enabled: bool) -> &mut Self {
        self.sfx_enabled = enabled;

        self
    }

    /// Queue a sound effect to be played.
    pub fn mark_sfx(&mut self, name: &'static str) -> bool {
        if self.sfx_enabled {
            self.data.mark_sfx(name)
        } else {
            true
        }
    }

    /// Play all queued sound effects.
    pub fn play_sfx(&mut self) -> bool {
        if self.sfx_enabled {
            self.data.play_sfx()
        } else {
            true
        }
    }

    /// Fade out the current music.
    pub fn fade(&self) {
        Music::fade_out(FADE_OUT_TIME).expect("fading out should work")
    }

    /// Stop playing all music.
    pub fn halt(&self) {
        Music::halt()
    }
}
