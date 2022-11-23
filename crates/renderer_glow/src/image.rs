use glam::Mat4;
use glow::HasContext;

use crate::{create_program, create_shader, ShaderType};

const VERTEX_SHADER: &str = include_str!("shader/image.vert");
const FRAGMENT_SHADER: &str = include_str!("shader/image.frag");

pub struct Pipeline {
    pub program: glow::Program,
    pub attrib_position: u32,
    pub uniform_matrix: glow::UniformLocation,
}

impl Pipeline {
    pub unsafe fn new(context: &glow::Context) -> Result<Self, String> {
        let vertex = create_shader(context, ShaderType::Vertex, VERTEX_SHADER)?;
        let fragment = create_shader(context, ShaderType::Fragment, FRAGMENT_SHADER);

        // If the fragment shader failed to compile, ensure the vertex shader is freed.
        if fragment.is_err() {
            context.delete_shader(vertex);
        }

        let fragment = fragment.unwrap();
        let program = create_program(context, &[vertex, fragment], &[(0, "position")])?;

        let attrib_position = context
            .get_attrib_location(program, "position")
            .ok_or("Failed to get location of attribute \"position\"")?;

        let uniform_matrix = context
            .get_uniform_location(program, "matrix")
            .ok_or("Failed to get location of uniform \"matrix\"")?;

        Ok(Pipeline {
            program,
            attrib_position,
            uniform_matrix,
        })
    }

    /// SAFETY: The pipeline must be bound.
    pub unsafe fn draw(&self, _context: &glow::Context) {}
}

impl crate::Pipeline for Pipeline {
    unsafe fn bind(&self, context: &glow::Context, width: u32, height: u32) {
        context.use_program(Some(self.program));
        context.enable_vertex_attrib_array(self.attrib_position);

        let matrix = Mat4::orthographic_rh(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);

        // Flip to compensate for the OpenGL coordinate system
        let matrix = Mat4::from_rotation_x(std::f32::consts::PI) * matrix;
        let array = matrix.to_cols_array();

        // Set the size of the viewport
        context.uniform_matrix_4_f32_slice(Some(&self.uniform_matrix), false, &array);
    }

    unsafe fn unbind(&self, context: &glow::Context) {
        context.disable_vertex_attrib_array(self.attrib_position);
        context.use_program(None);
    }
}
