pub trait Renderer: Sized {
    type Error: std::error::Error;
    type Image: Image<Self>;
}

pub trait Image<R: Renderer> {
    fn width(&self) -> u32;

    fn height(&self) -> u32;
}

// TODO: Mesh/Triangles?

pub struct Quad {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    // TODO: Radius
    // TODO: Fade
    // TODO: Thickness
    // TODO: Gradient instead of color?
    pub color: [f32; 4],
}

pub trait RenderQuad: Sized {
    // TODO: Texture?
    fn draw_quad(&mut self, quad: Quad);
}

// TODO: DrawImage
