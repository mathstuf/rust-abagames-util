// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

//! Audio subsystem support
//!
//! This module contains utilities to assist in loading any playing audio including background
//! music and sound effects.

use crates::failure::{Backtrace, Context, Fail, ResultExt};
use crates::rodio::{self, Decoder, Device, Sink, Source};
use crates::rodio::dynamic_mixer::{self, DynamicMixerController};
use crates::rodio::source;

use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::fmt::{self, Display};
use std::io::Cursor;
use std::mem;
use std::sync::Arc;

type Data = Decoder<Cursor<&'static [u8]>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Fail)]
enum ErrorKind {
    #[fail(display = "failed to decode {} audio '{}'", _0, _1)]
    DecodeError(&'static str, &'static str),
    #[fail(display = "no audio device found")]
    NoDevice,
}

#[derive(Debug)]
pub struct AudioError {
    inner: Context<ErrorKind>,
}

impl Fail for AudioError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for AudioError {
    fn from(kind: ErrorKind) -> Self {
        Self {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for AudioError {
    fn from(inner: Context<ErrorKind>) -> Self {
        Self {
            inner: inner,
        }
    }
}

/// Audio data information and management.
struct AudioData {
    /// Music files.
    // XXX: https://github.com/tomaka/rodio/pull/182
    music: HashMap<&'static str, source::Repeat<source::Buffered<Data>>>,

    /// Sound effect files.
    sfx: HashMap<&'static str, source::Buffered<Data>>,
}

impl AudioData {
    /// Load audio from data.
    fn new<'a, M, S>(music: M, sfx: S) -> Result<Self, AudioError>
        where M: IntoIterator<Item = &'a (&'static str, &'static [u8])>,
              S: IntoIterator<Item = &'a (&'static str, &'static [u8])>,
    {
        Ok(AudioData {
            music: music.into_iter()
                .map(|&(name, data)| {
                    let decoder = Decoder::new(Cursor::new(data))
                        .with_context(|_| ErrorKind::DecodeError("music", name))?;
                    Ok((name, decoder.buffered().repeat_infinite()))
                })
                .collect::<Result<HashMap<_, _>, AudioError>>()?,

            sfx: sfx.into_iter()
                .map(|&(name, data)| {
                    let decoder = Decoder::new(Cursor::new(data))
                        .with_context(|_| ErrorKind::DecodeError("sfx", name))?;
                    Ok((name, decoder.buffered()))
                })
                .collect::<Result<HashMap<_, _>, AudioError>>()?,
        })
    }
}

/// Audio support.
pub struct Audio {
    /// Audio data.
    data: AudioData,

    /// Whether music is enabled or not.
    music_enabled: bool,
    /// Whether sound effects is enabled or not.
    sfx_enabled: bool,

    /// Sink for music.
    music_sink: Sink,

    /// Sound effects queued for playing.
    queued_sfx: HashSet<&'static str>,
    /// Mixer for sound effects.
    sfx_mixer: Arc<DynamicMixerController<Format>>,

    /// The device being used.
    device: Device,
}

/// The frequency to play audio at.
const FREQUENCY: u32 = 44100;
/// The format of the audio.
type Format = i16;
/// The number of channels to play.
const CHANNELS: u16 = 1;
/// The amount of time, in milliseconds, over which to fade out music.
const FADE_OUT_TIME: i32 = 1280;

impl Audio {
    /// Load audio from data.
    pub fn new<'a, M, S>(music: M, sfx: S) -> Result<Self, AudioError>
        where M: IntoIterator<Item = &'a (&'static str, &'static [u8])>,
              S: IntoIterator<Item = &'a (&'static str, &'static [u8])>,
    {
        let device = rodio::default_output_device()
            .ok_or_else(|| ErrorKind::NoDevice)?;
        let (controller, mixer) = dynamic_mixer::mixer::<Format>(CHANNELS, FREQUENCY);
        let sfx_sink = Sink::new(&device);
        sfx_sink.append(mixer);
        sfx_sink.detach();

        Ok(Audio {
            data: AudioData::new(music.into_iter(), sfx.into_iter())?,

            music_enabled: true,
            sfx_enabled: true,

            music_sink: Sink::new(&device),

            queued_sfx: HashSet::new(),
            sfx_mixer: controller,

            device: device,
        })
    }

    /// Set whether music is enabled or not.
    pub fn set_music_enabled(&mut self, enabled: bool) -> &mut Self {
        self.music_enabled = enabled;

        self
    }

    /// Play the named music file in a loop.
    pub fn play_music<N>(&mut self, name: N)
        where N: AsRef<str>,
    {
        if self.music_enabled {
            let music = self.data
                .music
                .get(name.as_ref())
                .expect("no such music")
                .clone();
            self.halt();
            self.music_sink.append(music);
        }
    }

    /// Set whether sound effects are enabled or not.
    pub fn set_sfx_enabled(&mut self, enabled: bool) -> &mut Self {
        self.sfx_enabled = enabled;

        self
    }

    /// Queue a sound effect to be played.
    pub fn mark_sfx(&mut self, name: &'static str) {
        if self.sfx_enabled {
            self.queued_sfx.insert(name);
        }
    }

    /// Play all queued sound effects.
    pub fn play_sfx(&mut self) {
        if self.sfx_enabled {
            let sfx_to_play = mem::replace(&mut self.queued_sfx, HashSet::new());

            sfx_to_play.iter()
                .for_each(|&name| {
                    self.data
                        .sfx
                        .get(name)
                        .map(|&ref sfx| self.sfx_mixer.add(sfx.clone()));
                })
        }
    }

    /// Fade out the current music.
    pub fn fade(&self) {
        // TODO: This is not supported at the moment.
        // See https://github.com/tomaka/rodio/issues/179
        // Music::fade_out(FADE_OUT_TIME).expect("fading out should work")
    }

    /// Stop playing all music.
    pub fn halt(&mut self) {
        self.music_sink = Sink::new(&self.device);
    }
}
