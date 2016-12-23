// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

//! Video subsystem support
//!
//! This module contains all of the structures required to juggle the graphics resources for a
//! game. It includes bits for holding the view matrix as well as a context structure to handle
//! flushing the rendering commands to the device.

extern crate gfx;
use self::gfx::format::{DepthStencil, Srgba8};
use self::gfx::handle::{DepthStencilView, RenderTargetView};

extern crate gfx_device_gl;
use self::gfx_device_gl::Device as GLDevice;
use self::gfx_device_gl::CommandBuffer as GLCommandBuffer;

extern crate gfx_window_sdl;

extern crate cgmath;
use self::cgmath::Matrix4;

extern crate sdl2;
use self::sdl2::Sdl;
use self::sdl2::hint;
use self::sdl2::video::{GLContext, GLProfile, Window};

use std::marker::PhantomData;

pub use self::gfx_device_gl::{Factory, Resources};
/// The specialized encoder type for the games.
pub type Encoder = gfx::Encoder<Resources, GLCommandBuffer>;

error_chain! {}

/// A context object for queuing commands to the rendering device.
pub struct EncoderContext<'a, R, C: 'a>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
{
    /// The view matrix for perspective rendering.
    pub perspective_matrix: Matrix4<f32>,
    /// The view matrix for orthographic rendering.
    pub orthographic_matrix: Matrix4<f32>,
    /// The encoder object.
    pub encoder: &'a mut gfx::Encoder<R, C>,
}

/// A context object to handle flushing commands to a device automatically.
pub struct EncoderDrawContext<'a, R, C: 'a, D: 'a>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          D: gfx::Device<Resources=R, CommandBuffer=C>,
{
    /// The encoder context.
    pub context: EncoderContext<'a, R, C>,
    device: &'a mut D,
    window: &'a mut Window,
}

impl<'a, R, C, D> Drop for EncoderDrawContext<'a, R, C, D>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          D: gfx::Device<Resources=R, CommandBuffer=C>,
{
    fn drop(&mut self) {
        self.context.encoder.flush(self.device);
        self.window.gl_swap_window();
        self.device.cleanup();
    }
}

/// Video support.
pub struct Video<'a> {
    window: Window,
    _gl_context: GLContext,
    device: GLDevice,
    factory: Factory,
    view: RenderTargetView<Resources, Srgba8>,
    depth_stencil_view: DepthStencilView<Resources, DepthStencil>,

    encoder: Encoder,

    perspective_matrix: Matrix4<f32>,
    orthographic_matrix: Matrix4<f32>,

    _phantom: PhantomData<&'a str>,
}

static NEAR_PLANE: f32 = 0.1;
static FAR_PLANE: f32 = 1000.;
static CLEAR_COLOR: [f32; 4] = [0.; 4];

impl<'a> Video<'a> {
    /// Create a new video structure.
    ///
    /// This creates the window and rendering surface for the video subsystem as well.
    pub fn new(sdl_context: &Sdl, caption: &str, size: &(u32, u32), windowed: bool) -> Result<Self> {
        let video = try!(sdl_context.video()
            .map_err(|err| ErrorKind::Msg(format!("failed to create the video context: {}",
                                                  err))));

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_flags().debug().set();
        gl_attr.set_context_version(3, 2);
        try!(video.gl_load_library_default()
            .map_err(|err| ErrorKind::Msg(format!("failed to load the OpenGL library: {}",
                                                  err))));

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

        let (window, gl_context, device, mut factory, view, depth_stencil_view) =
            gfx_window_sdl::init(&mut window);

        let mut renderer = try!(window.renderer().build()
            .chain_err(|| "failed to build a renderer"));
        try!(renderer.set_logical_size(width, height)
            .chain_err(|| "failed to set the logical window size"));
        let window = renderer.into_window().unwrap();

        window.gl_make_current(&gl_context).unwrap();

        hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "linear");

        sdl_context.mouse().show_cursor(false);

        let (width, height) = window.size();

        let encoder = factory.create_command_buffer().into();

        Ok(Video {
            perspective_matrix: Self::calc_perspective_matrix(width, height),
            orthographic_matrix: Self::calc_orthographic_matrix(),

            window: window,
            _gl_context: gl_context,
            device: device,
            factory: factory,
            view: view,
            depth_stencil_view: depth_stencil_view,

            encoder: encoder,

            _phantom: PhantomData,
        })
    }

    fn calc_perspective_matrix(width: u32, height: u32) -> Matrix4<f32> {
        let aspect = (height as f32) / (width as f32);

        cgmath::frustum(-NEAR_PLANE, NEAR_PLANE, -NEAR_PLANE * aspect, NEAR_PLANE * aspect, 0.1, FAR_PLANE)
    }

    fn calc_orthographic_matrix() -> Matrix4<f32> {
        // FIXME: Fix the 640x480 aspect ratio.
        cgmath::ortho(0., 640., 480., 0., -1., 1.)
    }

    /// Resize the window.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.perspective_matrix = Self::calc_perspective_matrix(width, height);
    }

    /// The perspective matrix for the window.
    pub fn perspective_matrix(&self) -> &Matrix4<f32> {
        &self.perspective_matrix
    }

    /// The factory for handling resources with the device.
    pub fn factory(&mut self) -> &mut Factory {
        &mut self.factory
    }

    /// The factory and viewpoint for the device and window.
    pub fn factory_view(&mut self) -> (&mut Factory, &RenderTargetView<Resources, Srgba8>) {
        (&mut self.factory, &self.view)
    }

    /// The context for the current state of the video subsystem.
    pub fn context<'b>(&'b mut self) -> EncoderDrawContext<'b, Resources, GLCommandBuffer, GLDevice> {
        self.encoder.clear(&mut self.view, CLEAR_COLOR);
        self.encoder.clear_depth(&mut self.depth_stencil_view, 0.);
        self.encoder.clear_stencil(&mut self.depth_stencil_view, 0);

        EncoderDrawContext {
            context: EncoderContext {
                perspective_matrix: self.perspective_matrix.clone(),
                orthographic_matrix: self.orthographic_matrix.clone(),
                encoder: &mut self.encoder,
            },
            device: &mut self.device,
            window: &mut self.window,
        }
    }
}
