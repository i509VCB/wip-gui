use glam::Mat4;
use glow::HasContext;

use crate::{create_shader, ShaderType, create_program};

pub struct Pipeline {
    pub program: glow::Program,
    pub attrib_position: u32,
    pub attrib_coords: u32,
    pub uniform_matrix: glow::UniformLocation,
    pub uniform_color: glow::UniformLocation,
    pub uniform_size: glow::UniformLocation,
    // TODO
    pub uniform_radius: glow::UniformLocation,
    pub uniform_thickness: glow::UniformLocation,
    pub uniform_fade: glow::UniformLocation,
}

// TODO: Reference https://github.com/iced-rs/iced/blob/master/glow/src/shader/compatibility/quad.frag
const VERTEX_SHADER: &str = include_str!("shader/quad_es2.vert");
const FRAGMENT_SHADER: &str = include_str!("shader/quad_es2.frag");

// TODO(Instancing): Gles 3.0+ or GL_EXT_instanced_arrays + GL_EXT_draw_instanced. 

impl Pipeline {
    pub unsafe fn new(context: &glow::Context) -> Result<Self, String> {
        let vertex = create_shader(context, ShaderType::Vertex, VERTEX_SHADER)?;
        let fragment = create_shader(context, ShaderType::Fragment, FRAGMENT_SHADER);

        // If the fragment shader failed to compile, ensure the vertex shader is freed.
        if fragment.is_err() {
            context.delete_shader(vertex);
        }

        let fragment = fragment.unwrap();
        let program = create_program(context, &[vertex, fragment], &[
            (0, "position"),
            (1, "coordinates"),
        ])?;

        // Get the attribute location of the position
        let attrib_position = context
            .get_attrib_location(program, "position")
            .ok_or("Failed to get location of attribute \"position\"")?;

        let attrib_coords = context
            .get_attrib_location(program, "coordinates")
            .ok_or("Failed to get location of attribute \"coordinates\"")?;

        let uniform_matrix = context
            .get_uniform_location(program, "matrix")
            .ok_or("Failed to get location of uniform \"matrix\"")?;

        // Uniform for color
        let uniform_color = context
            .get_uniform_location(program, "color")
            .ok_or("Failed to get location of uniform \"color\"")?;

        let uniform_size = context
            .get_uniform_location(program, "size")
            .ok_or("Failed to get location of uniform \"size\"")?;

        let uniform_radius = context
            .get_uniform_location(program, "radius")
            .ok_or("Failed to get location of uniform \"radius\"")?;

        let uniform_thickness = context
            .get_uniform_location(program, "thickness")
            .ok_or("Failed to get location of uniform \"thickness\"")?;

        let uniform_fade = context
            .get_uniform_location(program, "fade")
            .ok_or("Failed to get location of uniform \"fade\"")?;

        Ok(Self {
            program,
            attrib_position,
            attrib_coords,
            uniform_matrix,
            uniform_color,
            uniform_size,
            uniform_radius,
            uniform_thickness,
            uniform_fade,
        })
    }

    /// SAFETY: The pipeline must be bound.
    pub unsafe fn draw(&self, context: &glow::Context, position: (u32, u32), size: (u32, u32), color: [f32; 4]) {
        let x = position.0 as f32;
        let y = position.1 as f32;

        let width = size.0 as f32;
        let height = size.1 as f32;

        let verts: [f32; 30] = [
            x, y, 0.0, -1.0, -1.0, // 1
            width + x, y, 0.0, -1.0, 1.0, // 2
            width + x, height + y, 0.0, 1.0, 1.0, // 3
            //
            x, y, 0.0, -1.0, -1.0,
            width + x, height + y, 0.0, 1.0, 1.0,
            x, height + y, 0.0, 1.0, -1.0,
        ];

        context.uniform_4_f32_slice(Some(&self.uniform_color), &color);
        {
            let ratio = if width > height {
                [1.0, width / height]
            } else if height > width {
                [height / width, 1.0]
            } else {
                [1.0, 1.0]
            };
            context.uniform_2_f32_slice(Some(&self.uniform_size), &ratio);
        }

        // TODO: Parameters to adjust these uniforms
        context.uniform_1_f32(Some(&self.uniform_radius), 0.2);
        context.uniform_1_f32(Some(&self.uniform_thickness), 1.0); // 0.0725
        context.uniform_1_f32(Some(&self.uniform_fade), 0.006);

        // Allocate a buffer to draw into
        // TODO: Preallocation of buffers.
        let buffer = context.create_buffer().unwrap();
        context.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
        context.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&verts),
            glow::STATIC_DRAW,
        );

        context.vertex_attrib_pointer_f32(
            self.attrib_position,
            3,
            glow::FLOAT,
            false,
            (std::mem::size_of::<f32>() * 5) as _,
            0,
        );
        context.vertex_attrib_pointer_f32(
            self.attrib_coords,
            2,
            glow::FLOAT,
            false,
            (std::mem::size_of::<f32>() * 5) as _,
            (std::mem::size_of::<f32>() * 3) as _,
        );
        context.draw_arrays(glow::TRIANGLES, 0, (verts.len() / 5) as i32);

        context.delete_buffer(buffer);
    }
}

impl crate::Pipeline for Pipeline {
    unsafe fn bind(&self, context: &glow::Context, width: u32, height: u32) {
        context.use_program(Some(self.program));
        context.enable_vertex_attrib_array(self.attrib_position);
        context.enable_vertex_attrib_array(self.attrib_coords);
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
        context.disable_vertex_attrib_array(self.attrib_coords);
        context.use_program(None);
    }
}
