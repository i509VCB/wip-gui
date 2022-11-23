use std::ops::Deref;

use swash::{CacheKey, FontRef};

pub struct Font<Storage> {
    storage: Storage,
    offset: u32,
    key: CacheKey,
}

impl<Storage> Font<Storage>
where
    Storage: AsRef<[u8]>,
{
    pub fn from_vec(storage: Storage, index: usize) -> Result<Self, ()> {
        // Create a temporary font ref for validation and obtaining a cache key
        let font = FontRef::from_index(storage.as_ref(), index).ok_or(())?;
        let offset = font.offset;
        let key = font.key;

        Ok(Self {
            storage,
            offset,
            key,
        })
    }

    pub fn as_ref(&self) -> FontRef<'_> {
        FontRef {
            data: &self.storage.as_ref(),
            offset: self.offset,
            key: self.key,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use swash::{
        scale::{Render, ScaleContext, Source, StrikeWith},
        shape::{cluster::Glyph, ShapeContext},
        zeno::Format,
        FontDataRef,
    };

    #[test]
    fn test() {
        let data = std::fs::read("/usr/share/fonts/TTF/Hack-Regular.ttf").unwrap();
        let fonts = FontDataRef::new(&data).unwrap();
        let font = fonts.get(0).unwrap();

        let glyph = font.charmap().map('#');

        let mut context = ShapeContext::new();
        let mut shaper = context.builder(font).size(16.).build();
        shaper.add_str("#");

        let mut glyphs = Vec::<Glyph>::new();

        shaper.shape_with(|cluster| {
            glyphs.extend(cluster.glyphs);
            // dbg!(cluster);
        });

        let mut scale_context = ScaleContext::new();
        let mut scaler = scale_context.builder(font).hint(true).build();

        let image = Render::new(&[
            Source::ColorBitmap(StrikeWith::ExactSize),
            Source::ColorOutline(0),
            Source::ColorOutline(0),
            Source::Outline,
        ])
        .format(Format::Subpixel)
        .render(&mut scaler, glyphs.get(0).unwrap().id)
        .unwrap();

        write_png(&image.data, image.placement.width, image.placement.height);
    }

    fn write_png(buf: &[u8], width: u32, height: u32) {
        let path = std::env::current_dir().unwrap().join("example.png");
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .unwrap();

        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_color(png::ColorType::Rgba);
        let mut png_writer = encoder.write_header().unwrap();

        png_writer.write_image_data(buf).unwrap();
        png_writer.finish().unwrap();
    }
}
