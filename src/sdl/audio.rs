// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

//! Audio subsystem support
//!
//! This module contains utilities to assist in loading any playing audio including background
//! music and sound effects.

use crates::sdl2::mixer::{self, AudioFormat, Channel, Chunk, LoaderRWops, Music};

use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::mem;

use sdl::error::*;

/// Audio data information and management.
struct AudioData<'a> {
    /// Music files.
    music: HashMap<&'a str, Music<'a>>,

    /// Sound effect files.
    sfx: HashMap<&'a str, (Chunk, Channel)>,
    /// Sound effects queued for playing.
    queued_sfx: HashSet<&'static str>,
}

impl<'a> AudioData<'a> {
    /// Load audio from data.
    fn new<M, S, D>(music: M, sfx: S) -> SdlResult<Self>
    where
        M: IntoIterator<Item = &'a (&'a str, D)>,
        S: IntoIterator<Item = &'a (&'a str, D, i32)>,
        D: LoaderRWops<'a> + 'a,
    {
        Ok(AudioData {
            music: music
                .into_iter()
                .map(|&(name, ref loader)| {
                    Ok((name, loader.load_music().map_err(SdlError::Audio)?))
                })
                .collect::<SdlResult<HashMap<_, _>>>()?,

            sfx: sfx
                .into_iter()
                .map(|&(name, ref loader, channel)| {
                    Ok((
                        name,
                        (
                            loader.load_wav().map_err(SdlError::Audio)?,
                            Channel(channel),
                        ),
                    ))
                })
                .collect::<SdlResult<HashMap<_, _>>>()?,
            queued_sfx: HashSet::new(),
        })
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

        sfx_to_play
            .iter()
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
    data: AudioData<'a>,
    /// Whether music is enabled or not.
    music_enabled: bool,
    /// Whether sound effects is enabled or not.
    sfx_enabled: bool,
}

/// The frequency to play audio at.
const FREQUENCY: i32 = 44100;
/// The format of the audio.
const FORMAT: AudioFormat = mixer::AUDIO_S16;
/// The number of channels to play.
const CHANNELS: i32 = 1;
/// The size of the audio buffers.
const BUFFERS: i32 = 4096;
/// The number of times to repeat audio infinitely.
const PLAY_UNLIMITED: i32 = -1;
/// The amount of time, in milliseconds, over which to fade out music.
const FADE_OUT_TIME: i32 = 1280;

impl<'a> Audio<'a> {
    /// Load audio from data.
    pub fn new<M, S, D>(music: M, sfx: S) -> SdlResult<Self>
    where
        M: IntoIterator<Item = &'a (&'a str, D)>,
        S: IntoIterator<Item = &'a (&'a str, D, i32)>,
        D: LoaderRWops<'a> + 'a,
    {
        mixer::open_audio(FREQUENCY, FORMAT, CHANNELS, BUFFERS).map_err(SdlError::Audio)?;
        mixer::allocate_channels(CHANNELS);

        Ok(Audio {
            data: AudioData::new(music.into_iter(), sfx.into_iter())?,
            music_enabled: true,
            sfx_enabled: true,
        })
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
