use renderer::Renderer;

use crate::{Error, GlowImage, GlowRenderer};

impl Renderer for GlowRenderer {
    // TODO: Proper error type
    type Error = Error;
    type Image = GlowImage;
}

impl renderer::Image<GlowRenderer> for GlowImage {
    fn width(&self) -> u32 {
        todo!()
    }

    fn height(&self) -> u32 {
        todo!()
    }
}
