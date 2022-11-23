use std::{
    ffi::{c_void, CStr, CString},
    fmt,
    rc::Rc,
};

use glow::HasContext;

mod image;
mod quad;
mod renderer;
mod triangle;

#[derive(Debug)]
pub enum Error {}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}

pub struct GlowRenderer {
    /// Allocate the glow context on the heap to reduce the size of the renderer when moved in memory.
    context: Rc<glow::Context>,
    // Pipelines
    image: image::Pipeline,
    triangle: triangle::Pipeline,
    quad: quad::Pipeline,
}

impl GlowRenderer {
    /// Creates a glow renderer given a loader function.
    ///
    /// # Safety
    ///
    /// - The context must be current.
    /// - The context must be current when calling any renderer functions.
    // TODO: How to approach the required API?
    // - External images from EGL using GL_OES_EGL_image_external are in theory only supported with GLES.
    // - But the folks over at Mesa
    pub unsafe fn from_fn<F>(f: F) -> Self
    where
        F: Fn(&CStr) -> *const c_void,
    {
        let context = glow::Context::from_loader_function(|s| {
            let str = CString::new(s).unwrap();
            f(&str)
        });

        let image = image::Pipeline::new(&context).expect("Failed to create image pipeline");
        let triangle =
            triangle::Pipeline::new(&context).expect("Failed to create triangle pipeline");
        let quad = quad::Pipeline::new(&context).expect("Failed to create quad pipeline");

        Self {
            context: Rc::new(context),
            image,
            triangle,
            quad,
        }
    }

    /// Returns a reference to the internal [`glow::Context`].
    pub fn context(&self) -> &glow::Context {
        &self.context
    }

    /// Creates an image from an OpenGL texture id.
    ///
    /// This may be used to use textures produced from another library using the same context, such as OpenXR.
    ///
    /// The returned image will not destroy the texture id.
    ///
    /// # Safety
    ///
    /// - The id must be a valid OpenGL texture.
    /// - The texture id must be valid for the lifetime of the returned image.
    pub unsafe fn from_raw(&mut self, id: u32) -> Result<GlowImage, Error> {
        self.from_raw_image(id, false)
    }

    /// Creates an image from a raw OpenGL texture id that is bound to an external texture.
    ///
    /// This function should be used when you have imported a texture from external memory using EGL. This may
    /// be used to use textures produced by another library, graphics api or process.
    ///
    /// The returned image will not destroy the texture id.
    ///
    /// # Safety
    ///
    /// - The id must be a valid OpenGL texture.
    /// - The texture id must be valid for the lifetime of the returned image.
    pub unsafe fn from_raw_image_external_egl(&mut self, id: u32) -> Result<GlowImage, Error> {
        self.from_raw_image(id, true)
    }

    unsafe fn from_raw_image(&mut self, id: u32, external: bool) -> Result<GlowImage, Error> {
        if external {
            // If the image is external, GL_OES_EGL_image_external must be supported.
            if !self
                .context
                .supported_extensions()
                .contains("GL_OES_EGL_image_external")
            {
                todo!("Not supported");
            }
        }

        let id = glow::Context::create_texture_from_gl_name(id);

        Ok(GlowImage {
            id,
            is_external: external,
            owned: false,
        })
    }

    pub fn temp_draw(&mut self, width: u32, height: u32) {
        unsafe {
            self.triangle.bind(&self.context, width, height);
            self.triangle.draw(
                &self.context,
                &[100.0, 100.0, 0.0, 800.0, 800.0, 0.0, 100.0, 800.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            );
            self.triangle.unbind(&self.context);
        }

        // Quad
        unsafe {
            self.quad.bind(&self.context, width, height);
            self.quad
                .draw(&self.context, (100, 100), (400, 400), [0.3, 0.2, 0.9, 1.0]);

            self.quad
                .draw(&self.context, (500, 400), (400, 100), [0.2, 0.2, 0.2, 1.0]);

            self.quad
                .draw(&self.context, (500, 600), (100, 100), [0.7, 0.2, 0.9, 1.0]);
            self.quad
                .draw(&self.context, (600, 600), (100, 100), [0.7, 0.2, 0.9, 1.0]);

            self.quad
                .draw(&self.context, (500, 500), (200, 100), [0.7, 0.2, 0.1, 1.0]);
            self.quad
                .draw(&self.context, (700, 500), (100, 200), [0.7, 0.1, 0.6, 1.0]);
            self.quad.unbind(&self.context);
        }

        unsafe {
            loop {
                let error = self.context.get_error();

                if error == glow::NO_ERROR {
                    break;
                }

                eprintln!("GL Error: {}", error);
            }
        }
    }

    pub fn create_encoder(&mut self) -> Result<GlowEncoder<'_>, Error> {
        let encoder = GlowEncoder { renderer: self };

        Ok(encoder)
    }
}

#[must_use = "Dropping an encoder without submitting it does nothing"]
pub struct GlowEncoder<'a> {
    renderer: &'a GlowRenderer,
}

impl<'a> GlowEncoder<'a> {
    pub fn submit(self) -> Result<(), Error> {
        todo!()
    }
}

pub struct GlowImage {
    id: glow::Texture,
    /// Whether this image needs to use `samplerExternalOES` in order to be sampled.
    is_external: bool,
    owned: bool,
}

impl GlowImage {
    /// Returns the texture id of the image.
    pub fn id(&self) -> glow::Texture {
        self.id
    }

    /// Returns whether this texture requires use of the `samplerExternalOES` sampler when being sampled.
    pub fn is_external(&self) -> bool {
        self.is_external
    }
}

#[repr(u32)]
enum ShaderType {
    Vertex = glow::VERTEX_SHADER,
    Fragment = glow::FRAGMENT_SHADER,
}

/// The returned shader must be freed by the caller.
unsafe fn create_shader(
    context: &glow::Context,
    ty: ShaderType,
    source: &str,
) -> Result<glow::Shader, String> {
    let shader = context.create_shader(ty as u32).unwrap();
    context.shader_source(shader, source);
    context.compile_shader(shader);

    if !context.get_shader_compile_status(shader) {
        let log = context.get_shader_info_log(shader);
        context.delete_shader(shader);

        return Err(log);
    }

    Ok(shader)
}

/// The returned program must be freed by the caller.
///
/// Ownership of the shaders is transferred to the function.
unsafe fn create_program(
    context: &glow::Context,
    shaders: &[glow::Shader],
    attrs: &[(u32, &str)],
) -> Result<glow::Program, String> {
    let program = context.create_program().unwrap();

    for &shader in shaders {
        context.attach_shader(program, shader);
    }

    for &(location, name) in attrs {
        context.bind_attrib_location(program, location, name);
    }

    context.link_program(program);

    if !context.get_program_link_status(program) {
        let log = context.get_program_info_log(program);

        // Detach and destroy shaders.
        for &shader in shaders {
            context.detach_shader(program, shader);
            context.delete_shader(shader);
        }

        context.delete_program(program);

        return Err(log);
    }

    // Detach and destroy shaders
    for &shader in shaders {
        context.detach_shader(program, shader);
        context.delete_shader(shader);
    }

    Ok(program)
}

trait Pipeline: Sized {
    unsafe fn bind(&self, context: &glow::Context, width: u32, height: u32);

    unsafe fn unbind(&self, context: &glow::Context);
}
