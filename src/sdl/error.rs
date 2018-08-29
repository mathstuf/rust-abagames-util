// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

use crates::failure::{Backtrace, Context, Fail};
use crates::gfx_window_sdl::InitError;
use crates::sdl2::IntegerOrSdlError;

use std::fmt::{self, Display};
use std::result::Result as StdResult;

// https://github.com/Rust-SDL2/rust-sdl2/pull/791
// #[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Debug)]
pub enum VideoStep {
    CreateSdlContext(String),
    LoadLibrary(String),
    Initialize(InitError),
    BuildRenderer(IntegerOrSdlError),
    WindowSize(IntegerOrSdlError),
}

impl VideoStep {
    // A hack to make implementing `PartialEq` easier.
    fn id(&self) -> u16 {
        match *self {
            VideoStep::CreateSdlContext(_) => 0,
            VideoStep::LoadLibrary(_) => 1,
            VideoStep::Initialize(_) => 2,
            VideoStep::BuildRenderer(_) => 3,
            VideoStep::WindowSize(_) => 4,
        }
    }
}

impl PartialEq for VideoStep {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Display for VideoStep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            VideoStep::CreateSdlContext(ref err) => {
                format!("failed to create the video context: {}", err)
            },
            VideoStep::LoadLibrary(ref err) => {
                format!("failed to load the OpenGL library: {}", err)
            },
            VideoStep::Initialize(ref err) => {
                format!("failed to initialize the video subsystem: {:?}", err)
            },
            VideoStep::BuildRenderer(ref err) => format!("failed to build a renderer: {:?}", err),
            VideoStep::WindowSize(ref err) => {
                format!("failed to set the logical window size: {:?}", err)
            },
        };

        write!(f, "{}", msg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStep {
    /// Failed to initialize the game.
    Initialize,
    HandleEvent,
    StepGame,
    DrawFrame,
    Quit,
}

impl GameStep {
    fn msg(&self) -> &'static str {
        match *self {
            GameStep::Initialize => "failed to initialize the game",
            GameStep::HandleEvent => "failed to handle an event",
            GameStep::StepGame => "failed to step the game",
            GameStep::DrawFrame => "failed to draw a frame",
            GameStep::Quit => "failed to quit the game",
        }
    }
}

impl Display for GameStep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg())
    }
}

// #[derive(Debug, Clone, PartialEq, Fail)]
#[derive(Debug, PartialEq, Fail)]
pub enum ErrorKind {
    /// An error from SDL itself.
    #[fail(display = "an error from SDL: {}", _0)]
    Sdl(String),
    /// An error from the audio subsystem.
    #[fail(display = "an error from the audio subsystem: {}", _0)]
    Audio(String),
    /// An error from the video subsystem.
    #[fail(display = "an error from the video subsystem: {}", _0)]
    Video(VideoStep),
    /// An error from the main loop and game itself.
    #[fail(display = "an error from the main loop and game itself: {}", _0)]
    Mainloop(GameStep),
}

impl From<VideoStep> for ErrorKind {
    fn from(step: VideoStep) -> Self {
        ErrorKind::Video(step)
    }
}

impl From<GameStep> for ErrorKind {
    fn from(step: GameStep) -> Self {
        ErrorKind::Mainloop(step)
    }
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

pub type Result<T> = StdResult<T, Error>;

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Self {
            inner: inner,
        }
    }
}
