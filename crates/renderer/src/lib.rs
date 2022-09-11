pub trait Renderer: Sized {
    type Error: std::error::Error;
    type CommandRecorder: CommandRecorder<Self>;
    type Scene: Scene<Self>;
    type Image: Image<Self>;

    // TODO: Start command submission
    // TODO: Offscreen render context?

    /// Begin recording commands to submit for rendering.
    fn record<F>(&mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::CommandRecorder) -> Result<(), Self::Error>;
}

pub trait CommandRecorder<R: Renderer> {
    // TODO: Render context for target?
    fn bind_target(&mut self);

    // TODO: Download image?
}

// TODO: Name?
pub trait Scene<R: Renderer> {
    /// Width of the render target.
    fn width(&self) -> u32;

    fn height(&self) -> u32;

    // TODO: Draw image

    // TODO: Draw path

    // TODO: Draw color

    // TODO: Apply transform

    // TODO: Set scissor boxes

    // TODO: Set clipping mask (using depth buffer/stencil buffer to occlude anything outside some bounds.)
    // This could be useful for discarding fragments outside the specified "damage boxes"

    // TODO: Draw text

    // TODO: Command recorder (used for caching).
}

pub trait Image<R: Renderer> {
    fn width(&self) -> u32;

    fn height(&self) -> u32;
}

// TODO: 3D extensions?
pub trait Renderer3D: Renderer
where
    Self::Scene: RendererContext3D<Self>,
{
}

pub trait RendererContext3D<R: Renderer3D>
where
    <R as Renderer>::Scene: RendererContext3D<R>,
{
    // TODO: Draw mesh (with image as texture mapping)

    // TODO: 3D transform
}
