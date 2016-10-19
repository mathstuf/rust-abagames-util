// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate draw_state;
use self::draw_state::target::Rect;

extern crate gfx;
use self::gfx::Encoder;

extern crate gfx_core;
use self::gfx_core::Device;
use self::gfx_core::draw::CommandBuffer;
use self::gfx_core::format::{DepthStencil, Srgba8};
use self::gfx_core::handle::{DepthStencilView, RenderTargetView};

extern crate gfx_device_gl;
use self::gfx_device_gl::Device as GLDevice;
use self::gfx_device_gl::CommandBuffer as GLCommandBuffer;
use self::gfx_device_gl::{Factory, Resources};

extern crate gfx_window_sdl;

extern crate nalgebra;
use self::nalgebra::{Matrix4, Perspective3, PerspectiveMatrix3};

extern crate sdl2;
use self::sdl2::Sdl;
use self::sdl2::hint;
use self::sdl2::video::{GLContext, GLProfile, Window};

use std::error::Error;

pub type ObjectEncoder = Encoder<Resources, GLCommandBuffer>;

pub struct Video {
    window: Window,
    _gl_context: GLContext,
    device: GLDevice,
    factory: Factory,
    view: RenderTargetView<Resources, Srgba8>,
    depth_stencil_view: DepthStencilView<Resources, DepthStencil>,

    perspective_matrix: PerspectiveMatrix3<f32>,
}

static NEAR_PLANE: f32 = 0.1;
static FAR_PLANE: f32 = 1000.;
static CLEAR_COLOR: [f32; 4] = [0.; 4];

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

        let (window, gl_context, device, factory, view, depth_stencil_view) =
            gfx_window_sdl::init(&mut window);

        let mut renderer = try!(window.renderer().build());
        try!(renderer.set_logical_size(width, height));
        let window = renderer.into_window().unwrap();

        hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "linear");

        sdl_context.mouse().show_cursor(false);

        let (width, height) = window.size();

        let aspect = (height as f32) / (width as f32);
        let fovy = ((height as f32) / (2. * FAR_PLANE)).atan() * 2.;

        let matrix = Perspective3::new(aspect, fovy, NEAR_PLANE, FAR_PLANE)
            .to_perspective_matrix();

        Ok(Video {
            perspective_matrix: matrix,

            window: window,
            _gl_context: gl_context,
            device: device,
            factory: factory,
            view: view,
            depth_stencil_view: depth_stencil_view,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let aspect = (height as f32) / (width as f32);
        let fovy = ((height as f32) / (2. * FAR_PLANE)).atan() * 2.;

        self.perspective_matrix.set_aspect(aspect);
        self.perspective_matrix.set_fovy(fovy);
    }

    pub fn perspective_matrix(&self) -> &Matrix4<f32> {
        self.perspective_matrix.as_matrix()
    }

    pub fn render_with<F>(&mut self, mut render: F)
        where F: FnMut(&mut ObjectEncoder),
    {
        let mut buffer = self.factory.create_command_buffer();

        let size = self.window.size();
        buffer.set_scissor(Rect {
            x: 0,
            y: 0,
            w: size.0 as u16,
            h: size.1 as u16,
        });

        let mut encoder: ObjectEncoder = self.factory
            .create_command_buffer()
            .into();

        encoder.clear(&mut self.view, CLEAR_COLOR);
        encoder.clear_depth(&mut self.depth_stencil_view, 0.);
        encoder.clear_stencil(&mut self.depth_stencil_view, 0);

        render(&mut encoder);

        encoder.flush(&mut self.device);
        self.window.gl_swap_window();
        self.device.cleanup();
    }
}
