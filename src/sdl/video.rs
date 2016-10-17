// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate gfx_device_gl;
use self::gfx_device_gl::Device as GLDevice;
use self::gfx_device_gl::Factory;

extern crate gfx_window_sdl;

extern crate nalgebra;
use self::nalgebra::{Matrix4, Perspective3, PerspectiveMatrix3};

extern crate sdl2;
use self::sdl2::Sdl;
use self::sdl2::hint;
use self::sdl2::video::{GLContext, GLProfile, Window};

use std::error::Error;

pub struct Video {
    window: Window,
    _gl_context: GLContext,
    device: GLDevice,
    factory: Factory,

    perspective_matrix: PerspectiveMatrix3<f32>,
}

static NEAR_PLANE: f32 = 0.1;
static FAR_PLANE: f32 = 1000f32;

impl Video {
    pub fn new(sdl_context: &Sdl, caption: &str, size: &(u32, u32), windowed: bool) -> Result<Self, Box<Error>> {
        let video = try!(sdl_context.video());

        video.gl_attr()
            .set_context_profile(GLProfile::GLES);
        try!(video.gl_load_library_default());

        let &(width, height) = size;

        let mut window = video.window(caption, width, height);

        window.opengl()
            .allow_highdpi();

        if windowed {
            window.position_centered()
                .resizable();
        } else {
            window.fullscreen_desktop();
        }

        let (window, gl_context, device, factory, _, _) =
            gfx_window_sdl::init(&mut window);

        let mut renderer = try!(window.renderer().build());
        try!(renderer.set_logical_size(width, height));
        let window = renderer.into_window().unwrap();

        hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "linear");

        sdl_context.mouse().show_cursor(false);

        let (width, height) = window.size();

        let aspect = (height as f32) / (width as f32);
        let fovy = ((height as f32) / (2f32 * FAR_PLANE)).atan() * 2f32;

        let matrix = Perspective3::new(aspect, fovy, NEAR_PLANE, FAR_PLANE)
            .to_perspective_matrix();

        Ok(Video {
            perspective_matrix: matrix,

            window: window,
            _gl_context: gl_context,
            device: device,
            factory: factory,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let aspect = (height as f32) / (width as f32);
        let fovy = ((height as f32) / (2f32 * FAR_PLANE)).atan() * 2f32;

        self.perspective_matrix.set_aspect(aspect);
        self.perspective_matrix.set_fovy(fovy);
    }

    pub fn perspective_matrix(&self) -> &Matrix4<f32> {
        self.perspective_matrix.as_matrix()
    }
}
