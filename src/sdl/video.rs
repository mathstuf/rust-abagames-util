// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

//! Video subsystem support
//!
//! This module contains all of the structures required to juggle the graphics resources for a
//! game. It includes bits for holding the view matrix as well as a context structure to handle
//! flushing the rendering commands to the device.

use crates::cgmath::{self, Matrix4, Vector2};
use crates::gfx;
use crates::gfx::format::{DepthStencil, Srgba8};
use crates::gfx::handle::{DepthStencilView, RenderTargetView};
use crates::gfx_device_gl::CommandBuffer as GLCommandBuffer;
use crates::gfx_device_gl::Device as GLDevice;
use crates::gfx_window_sdl;
use crates::sdl2::hint;
use crates::sdl2::video::{GLContext, GLProfile, Window};
use crates::sdl2::Sdl;

use sdl::error::*;

pub use crates::gfx_device_gl::{Factory, Resources};
/// The specialized encoder type for the games.
pub type Encoder = gfx::Encoder<Resources, GLCommandBuffer>;

/// The pixel format of the SDL surface.
pub type TargetFormat = Srgba8;

/// A context object for queuing commands to the rendering device.
pub struct EncoderContext<'a, R, C: 'a>
where
    R: gfx::Resources,
{
    /// The size of the view.
    pub size: Vector2<u32>,
    /// The view matrix for perspective rendering.
    pub perspective_matrix: Matrix4<f32>,
    /// The view matrix for orthographic rendering.
    pub orthographic_matrix: Matrix4<f32>,
    /// The encoder object.
    pub encoder: &'a mut gfx::Encoder<R, C>,
}

/// A context object to handle flushing commands to a device automatically.
pub struct EncoderDrawContext<'a, R, C: 'a, D: 'a>
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
    D: gfx::Device<Resources = R, CommandBuffer = C>,
{
    /// The encoder context.
    pub context: EncoderContext<'a, R, C>,
    device: &'a mut D,
    window: &'a mut Window,
}

impl<'a, R, C, D> Drop for EncoderDrawContext<'a, R, C, D>
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
    D: gfx::Device<Resources = R, CommandBuffer = C>,
{
    fn drop(&mut self) {
        self.context.encoder.flush(self.device);
        self.window.gl_swap_window();
        self.device.cleanup();
    }
}

/// Video support.
pub struct Video {
    window: Window,
    _gl_context: GLContext,
    device: GLDevice,
    factory: Factory,
    view: RenderTargetView<Resources, TargetFormat>,
    depth_stencil_view: DepthStencilView<Resources, DepthStencil>,

    encoder: Encoder,

    size: Vector2<u32>,
    perspective_matrix: Matrix4<f32>,
    orthographic_matrix: Matrix4<f32>,
}

const NEAR_PLANE: f32 = 0.1;
const FAR_PLANE: f32 = 1000.;
const CLEAR_COLOR: [f32; 4] = [0.; 4];

impl Video {
    /// Create a new video structure.
    ///
    /// This creates the window and rendering surface for the video subsystem as well.
    pub fn new(
        sdl_context: &Sdl,
        caption: &str,
        size: Vector2<u32>,
        windowed: bool,
    ) -> Result<Self> {
        let video = sdl_context
            .video()
            .map_err(|msg| ErrorKind::Video(VideoStep::CreateSdlContext(msg)))?;

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_flags().debug().set();
        gl_attr.set_context_version(3, 2);
        gl_attr.set_stencil_size(0);
        video
            .gl_load_library_default()
            .map_err(|msg| ErrorKind::Video(VideoStep::LoadLibrary(msg)))?;

        let mut window = video.window(caption, size.x, size.y);

        window.opengl().allow_highdpi();

        if windowed {
            window.position_centered().resizable();
        } else {
            window.fullscreen_desktop();
        }

        let (window, gl_context, device, mut factory, view, depth_stencil_view) =
            gfx_window_sdl::init(&video, window)
                .map_err(|err| ErrorKind::Video(VideoStep::Initialize(err)))?;

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|err| ErrorKind::Video(VideoStep::BuildRenderer(err)))?;
        canvas
            .set_logical_size(size.x, size.y)
            .map_err(|err| ErrorKind::Video(VideoStep::WindowSize(err)))?;
        let window = canvas.into_window();

        window
            .gl_make_current(&gl_context)
            .expect("failed to make an OpenGL context");

        hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "linear");

        sdl_context.mouse().show_cursor(false);

        let win_size = window.size().into();

        Ok(Video {
            size: win_size,
            perspective_matrix: Self::calc_perspective_matrix(win_size),
            orthographic_matrix: Self::calc_orthographic_matrix(win_size),

            encoder: factory.create_command_buffer().into(),

            window,
            _gl_context: gl_context,
            device,
            factory,
            view,
            depth_stencil_view,
        })
    }

    fn calc_perspective_matrix(size: Vector2<u32>) -> Matrix4<f32> {
        let aspect = (size.y as f32) / (size.x as f32);

        cgmath::frustum(
            -NEAR_PLANE,
            NEAR_PLANE,
            -NEAR_PLANE * aspect,
            NEAR_PLANE * aspect,
            0.1,
            FAR_PLANE,
        )
    }

    fn calc_orthographic_matrix(size: Vector2<u32>) -> Matrix4<f32> {
        cgmath::ortho(0., size.x as f32, size.y as f32, 0., -1., 1.)
    }

    /// Resize the window.
    pub fn resize(&mut self, size: Vector2<u32>) {
        self.size = size;
        self.perspective_matrix = Self::calc_perspective_matrix(size);
        self.orthographic_matrix = Self::calc_orthographic_matrix(size);
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
    pub fn factory_view(&mut self) -> (&mut Factory, &RenderTargetView<Resources, TargetFormat>) {
        (&mut self.factory, &self.view)
    }

    /// The context for the current state of the video subsystem.
    pub fn context(&mut self) -> EncoderDrawContext<Resources, GLCommandBuffer, GLDevice> {
        self.encoder.clear(&self.view, CLEAR_COLOR);
        self.encoder.clear_depth(&self.depth_stencil_view, 0.);
        self.encoder.clear_stencil(&self.depth_stencil_view, 0);

        EncoderDrawContext {
            context: EncoderContext {
                size: self.size,
                perspective_matrix: self.perspective_matrix,
                orthographic_matrix: self.orthographic_matrix,
                encoder: &mut self.encoder,
            },
            device: &mut self.device,
            window: &mut self.window,
        }
    }
}
