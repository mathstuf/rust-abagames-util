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

extern crate cgmath;
use self::cgmath::Matrix4;

extern crate sdl2;
use self::sdl2::Sdl;
use self::sdl2::hint;
use self::sdl2::video::{GLContext, GLProfile, Window};

use std::error::Error;
use std::marker::PhantomData;

pub type ObjectEncoder = Encoder<Resources, GLCommandBuffer>;

pub struct Video<'a> {
    window: Window,
    _gl_context: GLContext,
    device: GLDevice,
    factory: Factory,
    view: RenderTargetView<Resources, Srgba8>,
    depth_stencil_view: DepthStencilView<Resources, DepthStencil>,

    matrix: Matrix4<f32>,

    _phantom: PhantomData<&'a str>,
}

static NEAR_PLANE: f32 = 0.1;
static FAR_PLANE: f32 = 1000.;
static CLEAR_COLOR: [f32; 4] = [0.; 4];

impl<'a> Video<'a> {
    pub fn new(sdl_context: &Sdl, caption: &str, size: &(u32, u32), windowed: bool) -> Result<Self, Box<Error>> {
        let video = try!(sdl_context.video());

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_flags().debug().set();
        gl_attr.set_context_version(3, 2);
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

        Ok(Video {
            matrix: Self::perspective_matrix(width, height),

            window: window,
            _gl_context: gl_context,
            device: device,
            factory: factory,
            view: view,
            depth_stencil_view: depth_stencil_view,

            _phantom: PhantomData,
        })
    }

    fn perspective_matrix(width: u32, height: u32) -> Matrix4<f32> {
        let aspect = (height as f32) / (width as f32);
        let fovy = ((height as f32) / (2. * FAR_PLANE)).atan() * 2.;

        cgmath::perspective(cgmath::Rad(fovy), aspect, NEAR_PLANE, FAR_PLANE)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.matrix = Self::perspective_matrix(width, height)
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        &self.matrix
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
