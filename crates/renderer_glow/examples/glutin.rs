use std::num::NonZeroU32;

use glow::HasContext;
use glutin::{
    config::{Config, ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, Version},
    display::{Display, DisplayApiPreference},
    prelude::*,
    surface::{Surface, SurfaceAttributes, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use renderer_glow::GlowRenderer;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    let event_loop = EventLoop::new();

    let raw_display = event_loop.raw_display_handle();

    let window = WindowBuilder::new()
        .with_decorations(true)
        .with_inner_size(LogicalSize::new(1000, 1000))
        .build(&event_loop)
        .unwrap();
    let raw_window_handle = window.raw_window_handle();

    let gl_display = create_display(raw_display, raw_window_handle);

    let template = config_template(raw_window_handle);
    let config = unsafe { gl_display.find_configs(template) }
        .unwrap()
        .next()
        .unwrap();

    let surface = create_surface(&gl_display, &window, &config);

    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(Some(Version { major: 3, minor: 0 })))
        .build(Some(raw_window_handle));
    let gl_context = unsafe {
        gl_display
            .create_context(&config, &context_attributes)
            .unwrap()
    };

    let gl_context = gl_context.make_current(&surface).unwrap();

    let mut renderer = unsafe { GlowRenderer::from_fn(|addr| gl_context.get_proc_address(addr)) };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        // Some platforms like EGL require resizing GL surface to update the size
                        // Notable platforms here are Wayland and macOS, other don't require it
                        // and the function is no-op, but it's wise to resize it for portability
                        // reasons.
                        surface.resize(
                            &gl_context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                    }
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            Event::RedrawEventsCleared => {
                // window.request_redraw();

                let context = renderer.context();
                unsafe {
                    let size = window.inner_size();
                    // Set viewport
                    context.viewport(0, 0, size.width as i32, size.height as i32);
                    // context.clear_color(0., 0.3, 0.3, 0.8);
                    context.clear_color(1.0, 1.0, 1.0, 1.0);
                    context.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                    context.enable(glow::BLEND);
                    context.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
                    // context.enable(glow::DEPTH_TEST);
                    renderer.temp_draw(size.width, size.height);
                }

                surface.swap_buffers(&gl_context).unwrap();
            }
            _ => (),
        }
    });
}

fn create_display(raw_display: RawDisplayHandle, _raw_window_handle: RawWindowHandle) -> Display {
    unsafe { Display::from_raw(raw_display, DisplayApiPreference::Egl) }.unwrap()
}

fn create_surface(display: &Display, window: &Window, config: &Config) -> Surface<WindowSurface> {
    let attrs = surface_attributes(window);
    unsafe { display.create_window_surface(config, &attrs) }.unwrap()
}

fn surface_attributes(window: &Window) -> SurfaceAttributes<WindowSurface> {
    let (width, height): (u32, u32) = window.inner_size().into();
    let raw_window_handle = window.raw_window_handle();
    SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    )
}

fn config_template(raw_window_handle: RawWindowHandle) -> ConfigTemplate {
    ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(true)
        .compatible_with_native_window(raw_window_handle)
        .with_surface_type(ConfigSurfaceTypes::WINDOW)
        .build()
}
