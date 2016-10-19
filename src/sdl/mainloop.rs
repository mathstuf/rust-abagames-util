// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate sdl2;
use self::sdl2::Sdl;
pub use self::sdl2::event::Event;

use std::error::Error;

pub trait Game {
    fn init(&mut self);
    fn handle_event(&mut self, event: &Event) -> f32;
    fn step_frame(&mut self) -> f32;
    fn draw(&mut self);
    fn quit(&mut self);
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

    pub fn run(&mut self, game: &mut Game) {
        let mut pump = self.sdl_context.event_pump().unwrap();
        let mut timer = self.sdl_context.timer().unwrap();

        let mut prev_tick = 0;
        let mut interval = INTERVAL_BASE;

        game.init();

        for event in pump.poll_iter() {
            game.handle_event(&event);

            let is_done = if let &Event::Quit{..} = &event {
                true
            } else {
                false
            };

            let now_tick = timer.ticks();
            let interval_u32 = interval as u32;
            let frame = ((now_tick - prev_tick) as f32 / interval) as i32;

            let frames = if frame <= 0 {
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

            let slowdown: f32 = [0..frames].iter()
                .map(|_| game.step_frame())
                .sum();

            game.draw();

            if !NO_WAIT {
                interval = Self::calculate_interval(interval, slowdown / (frames as f32));
            }

            if is_done {
                break;
            }
        }
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