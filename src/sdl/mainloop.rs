// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::Sdl;
pub use self::sdl2::event::Event;

use super::input::Input;

use std::error;
use std::result;

pub enum StepResult {
    Slowdown(f32),
    Done,
}

impl StepResult {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (StepResult::Done, _) |
            (_, StepResult::Done) => StepResult::Done,
            (StepResult::Slowdown(s1), StepResult::Slowdown(s2)) => {
                StepResult::Slowdown(s1 + s2)
            },
        }
    }
}

error_chain! {}

pub trait Game {
    type Error: error::Error + Send + 'static;

    fn init(&mut self) -> result::Result<(), Self::Error>;
    fn handle_event(&mut self, event: &Event) -> result::Result<bool, Self::Error>;
    fn step(&mut self, input: &Input) -> result::Result<StepResult, Self::Error>;
    fn draw(&mut self) -> result::Result<(), Self::Error>;
    fn quit(&mut self) -> result::Result<(), Self::Error>;
}

pub struct MainLoop<'a> {
    sdl_context: &'a Sdl,
}

static INTERVAL_BASE: f32 = 16.;
static MAX_SKIP_FRAME: i32 = 5;
static NO_WAIT: bool = false;
static ACCELERATE_FRAME: bool = false;
static SLOWDOWN_START_RATIO: f32 = 1.;
static SLOWDOWN_MAX_RATIO: f32 = 1.75;

impl<'a> MainLoop<'a> {
    pub fn new(sdl_context: &'a Sdl) -> Self {
        MainLoop {
            sdl_context: &sdl_context,
        }
    }

    pub fn run<G: Game>(&self, mut game: G) -> Result<()> {
        let mut pump = try!(self.sdl_context.event_pump());
        let mut timer = try!(self.sdl_context.timer());

        let mut prev_tick = 0;
        let mut interval = INTERVAL_BASE;

        try!(game.init()
            .chain_err(|| "failed to initialize the game"));

        loop {
            let event = pump.poll_event();

            let mut is_done = if let Some(event) = event {
                if let &Event::Quit{..} = &event {
                    true
                } else {
                    try!(game.handle_event(&event)
                        .chain_err(|| "failed to handle an event"))
                }
            } else {
                false
            };

            let now_tick = timer.ticks();
            let frame = (((now_tick as f32) - (prev_tick as f32)) / interval) as i32;

            let frames = if frame <= 0 {
                let interval_u32 = interval as u32;
                timer.delay(prev_tick + interval_u32 - now_tick);

                if ACCELERATE_FRAME {
                    prev_tick = timer.ticks();
                } else {
                    prev_tick += interval_u32;
                }

                1
            } else if frame > MAX_SKIP_FRAME {
                prev_tick = now_tick;

                MAX_SKIP_FRAME
            } else {
                prev_tick = now_tick;

                frame
            };

            let input = Input::new(&pump);

            let step_result = try!((0..frames)
                .map(|_| Ok(try!(game.step(&input).chain_err(|| "failed to step the game"))))
                .collect::<Result<Vec<_>>>())
                .into_iter()
                .fold(StepResult::Slowdown(0.), StepResult::merge);

            let slowdown = match step_result {
                StepResult::Done => {
                    is_done = true;
                    0.
                },
                StepResult::Slowdown(s) => s,
            };

            try!(game.draw()
                .chain_err(|| "failed to draw a frame"));

            if !NO_WAIT {
                interval = Self::calculate_interval(interval, slowdown / (frames as f32));
            }

            if is_done {
                break;
            }
        }

        try!(game.quit()
            .chain_err(|| "failed to quit the game"));

        Ok(())
    }

    fn calculate_interval(interval: f32, slowdown: f32) -> f32 {
        interval + if slowdown > SLOWDOWN_START_RATIO {
            let ratio = f32::min(slowdown / SLOWDOWN_START_RATIO, SLOWDOWN_MAX_RATIO);
            (ratio * INTERVAL_BASE - interval) * 0.1
        } else {
            (INTERVAL_BASE - interval) * 0.08
        }
    }
}
