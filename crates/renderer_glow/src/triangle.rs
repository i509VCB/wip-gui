use glam::Mat4;
use glow::HasContext;

use crate::{create_program, create_shader, ShaderType};

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub program: glow::Program,
    pub attrib_position: u32,
    pub uniform_matrix: glow::UniformLocation,
    pub uniform_color: glow::UniformLocation,
}

const VERTEX_SHADER: &str = include_str!("shader/triangle.vert");
const FRAGMENT_SHADER: &str = include_str!("shader/triangle.frag");

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

        // Get the attribute location of the position
        let attrib_position = context
            .get_attrib_location(program, "position")
            .ok_or("Failed to get location of attribute \"position\"")?;

        let uniform_matrix = context
            .get_uniform_location(program, "matrix")
            .ok_or("Failed to get location of uniform \"matrix\"")?;

        // Uniform for color
        let uniform_color = context
            .get_uniform_location(program, "u_color")
            .ok_or("Failed to get location of uniform \"u_color\"")?;

        Ok(Self {
            program,
            attrib_position,
            uniform_matrix,
            uniform_color,
        })
    }

    /// SAFETY: The pipeline must be bound.
    pub unsafe fn draw(&self, context: &glow::Context, verts: &[f32], color: [f32; 4]) {
        context.uniform_4_f32_slice(Some(&self.uniform_color), &color);

        // Allocate a buffer to draw into
        // TODO: Preallocation of buffers.
        let buffer = context.create_buffer().unwrap();
        context.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
        context.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(verts),
            glow::STATIC_DRAW,
        );

        context.vertex_attrib_pointer_f32(
            0,
            3,
            glow::FLOAT,
            false,
            (std::mem::size_of::<f32>() * 3) as _,
            0,
        );
        context.draw_arrays(glow::TRIANGLES, 0, (verts.len() / 3) as i32);

        context.delete_buffer(buffer);
    }
}

impl crate::Pipeline for Pipeline {
    unsafe fn bind(&self, context: &glow::Context, width: u32, height: u32) {
        context.use_program(Some(self.program));
        context.enable_vertex_attrib_array(self.attrib_position);
        // Set the color to a default value
        context.uniform_4_f32_slice(Some(&self.uniform_color), &[1.0, 1.0, 1.0, 1.0]);

        let matrix = Mat4::orthographic_rh(
            0.0,
            width as f32,
            0.0,
            height as f32,
            -1.0,
            1.0,
        );

        // Flip to compensate for the OpenGL coordinate system
        let matrix = Mat4::from_rotation_x(std::f32::consts::PI) * matrix;
        let array = matrix.to_cols_array();

        // Set the size of the viewport
        context.uniform_matrix_4_f32_slice(
            Some(&self.uniform_matrix),
            false,
            &array
        );
    }

    unsafe fn unbind(&self, context: &glow::Context) {
        context.disable_vertex_attrib_array(self.attrib_position);
        context.use_program(None);
    }
}
