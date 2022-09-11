use renderer::Renderer;

use crate::{Error, GlowCommandRecorder, GlowImage, GlowRenderContext, GlowRenderer};

impl Renderer for GlowRenderer {
    // TODO: Proper error type
    type Error = Error;
    type CommandRecorder = GlowCommandRecorder;
    type Scene = GlowRenderContext;
    type Image = GlowImage;

    fn record<F>(&mut self, _f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::CommandRecorder) -> Result<(), Self::Error>,
    {
        todo!()
    }
}

impl renderer::CommandRecorder<GlowRenderer> for GlowCommandRecorder {
    fn bind_target(&mut self) {
        todo!()
    }
}

impl renderer::Scene<GlowRenderer> for GlowRenderContext {
    fn width(&self) -> u32 {
        todo!()
    }

    fn height(&self) -> u32 {
        todo!()
    }
}

impl renderer::Image<GlowRenderer> for GlowImage {
    fn width(&self) -> u32 {
        todo!()
    }

    fn height(&self) -> u32 {
        todo!()
    }
}
