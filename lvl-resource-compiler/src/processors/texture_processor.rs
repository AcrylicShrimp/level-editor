use super::Processor;
use anyhow::{anyhow, Error as AnyError};
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use lvl_resource::{
    Resource, TextureElement, TextureElementImageEncoding, TextureElementImageFormat,
    TextureElementSize,
};
use std::path::Path;

pub struct TextureProcessor {}

impl Processor for TextureProcessor {
    fn new() -> Self {
        Self {}
    }

    fn extension(&self) -> &'static [&'static str] {
        &["png", "jpg", "jpeg", "bmp", "tga"]
    }

    fn process(&self, file: &Path) -> Result<Vec<Resource>, AnyError> {
        let image = ImageReader::open(file)?.with_guessed_format()?;
        let format = match image.format() {
            Some(format) => format,
            None => return Err(anyhow!("image format not found")),
        };

        let encoding = match format {
            ImageFormat::Png => TextureElementImageEncoding::Png,
            ImageFormat::Jpeg => TextureElementImageEncoding::Jpeg,
            ImageFormat::Bmp => TextureElementImageEncoding::Bmp,
            ImageFormat::Tga => TextureElementImageEncoding::Tga,
            _ => return Err(anyhow!("unsupported image encoding")),
        };

        // let image_data = format.into_raw_rgba8();

        todo!()
    }
}

fn make_texture_element(file: &Path) -> Result<TextureElement, AnyError> {
    let image = ImageReader::open(file)?.with_guessed_format()?;
    let format = match image.format() {
        Some(format) => format,
        None => return Err(anyhow!("image format not found")),
    };
    let decoded = image.decode()?;

    let encoding = match format {
        ImageFormat::Png => TextureElementImageEncoding::Png,
        ImageFormat::Jpeg => TextureElementImageEncoding::Jpeg,
        ImageFormat::Bmp => TextureElementImageEncoding::Bmp,
        ImageFormat::Tga => TextureElementImageEncoding::Tga,
        _ => return Err(anyhow!("unsupported image encoding")),
    };
    let format = match decoded {
        DynamicImage::ImageLuma8(_) => TextureElementImageFormat::R8,
        DynamicImage::ImageRgb8(_) => TextureElementImageFormat::RGB8,
        DynamicImage::ImageRgba8(_) => TextureElementImageFormat::RGBA8,
        DynamicImage::ImageLuma16(_) => TextureElementImageFormat::R16,
        DynamicImage::ImageRgb16(_) => TextureElementImageFormat::RGB16,
        DynamicImage::ImageRgba16(_) => TextureElementImageFormat::RGBA16,
        DynamicImage::ImageRgb32F(_) => TextureElementImageFormat::RGB32F,
        DynamicImage::ImageRgba32F(_) => TextureElementImageFormat::RGBA32F,
        _ => return Err(anyhow!("unsupported image format")),
    };

    let width = decoded.width();
    let height = decoded.height();

    if (u16::MAX as u32) < width || (u16::MAX as u32) < height {
        return Err(anyhow!("image too large"));
    }

    let size = TextureElementSize {
        width: width as u16,
        height: height as u16,
    };

    // TODO: we need to get texture format and sampling/wrapping modes somehow

    todo!()
}
