use std::iter;

use lyon::{
    lyon_tessellation::{geometry_builder::simple_builder, VertexBuffers, StrokeOptions},
    math::{point, Box2D, Point},
    path::{builder::BorderRadii, Winding},
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let mut geometry = VertexBuffers::<Point, u16>::new();
    let mut geo_builder = simple_builder(&mut geometry);

    let mut tess = lyon::tessellation::StrokeTessellator::new();
    let mut builder = tess.builder(&StrokeOptions::DEFAULT, &mut geo_builder);

    builder.add_rounded_rectangle(
        &Box2D {
            min: point(0., 0.),
            max: point(100., 500.),
        },
        &BorderRadii {
            top_left: 10.,
            top_right: 5.,
            bottom_left: 20.,
            bottom_right: 25.,
        },
        Winding::Positive,
    );

    let _ = builder.build();
    dbg!(&geometry.vertices);
    dbg!(&geometry.indices);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("text rendering tests")
        .with_inner_size(PhysicalSize::new(1280, 800))
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY | wgpu::Backends::GL);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("No available adapters");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("wgpu device"),
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .expect("Failed to create device");

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface
            .get_supported_formats(&adapter)
            .first()
            .copied()
            .expect("Failed to find a supported surface format"),
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: surface
            .get_supported_present_modes(&adapter)
            .iter()
            .copied()
            .find(|&mode| mode == wgpu::PresentMode::Mailbox)
            .unwrap_or(wgpu::PresentMode::AutoVsync),
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
    };

    surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&device, &config);
                }

                WindowEvent::Destroyed | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }

                _ => (),
            },

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Rendering"),
                });

                let surface_texture = surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture");
                let view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        label: Some("surface texture view"),
                        ..Default::default()
                    });

                {
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    // The render pass is finished on drop.
                }

                let command_buffer = encoder.finish();
                queue.submit(iter::once(command_buffer));
                surface_texture.present();
            }

            _ => {}
        }
    });
}
