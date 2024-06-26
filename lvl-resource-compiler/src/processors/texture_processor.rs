use super::Processor;
use anyhow::{anyhow, Error as AnyError};
use image::io::Reader as ImageReader;
use lvl_resource::{
    Resource, ResourceKind, SpriteMapping, SpriteSource, TextureElement,
    TextureElementSamplingMode, TextureElementSize, TextureElementTextureFormat,
    TextureElementWrappingMode, TextureKind, TextureSource,
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct TextureMetadata {
    pub texture_format: TextureElementTextureFormat,
    pub sampling_mode: Option<TextureElementSamplingMode>,
    pub wrapping_mode_u: Option<TextureElementWrappingMode>,
    pub wrapping_mode_v: Option<TextureElementWrappingMode>,
    pub sprites: Option<BTreeMap<String, TextureSpriteElement>>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct TextureSpriteElement {
    pub min_x: u16,
    pub min_y: u16,
    pub max_x: u16,
    pub max_y: u16,
}

pub struct TextureProcessor;

impl TextureProcessor {
    pub fn generate_texture_source(
        file: &Path,
        metadata: &TextureMetadata,
    ) -> Result<TextureSource, AnyError> {
        let element = make_texture_element(
            file,
            metadata.texture_format,
            metadata.sampling_mode,
            metadata.wrapping_mode_u,
            metadata.wrapping_mode_v,
        )?;

        Ok(TextureSource::new(TextureKind::Single(element)))
    }
}

impl Processor for TextureProcessor {
    type Metadata = TextureMetadata;

    fn extension() -> &'static [&'static str] {
        &["png", "jpg", "jpeg", "bmp", "tga"]
    }

    fn process(file: &Path, metadata: Option<&Self::Metadata>) -> Result<Vec<Resource>, AnyError> {
        let name = file.file_stem().unwrap().to_string_lossy().to_string();
        let metadata = match metadata {
            Some(metadata) => metadata,
            None => {
                return Err(anyhow!(
                    "metadata not found for texture `{}`; it will be ignored.",
                    file.display()
                ));
            }
        };
        let source = Self::generate_texture_source(file, metadata)?;
        let mut resources =
            Vec::with_capacity(1 + metadata.sprites.as_ref().map_or(0, |sprites| sprites.len()));

        resources.push(Resource {
            name: name.clone(),
            kind: ResourceKind::Texture(source),
        });

        if let Some(sprites) = &metadata.sprites {
            for (sprite_name, mapping) in sprites {
                let source = SpriteSource::new(
                    name.clone(),
                    SpriteMapping {
                        min: (mapping.min_x, mapping.min_y),
                        max: (mapping.max_x, mapping.max_y),
                    },
                );
                resources.push(Resource {
                    name: format!("{}/{}", name, sprite_name),
                    kind: ResourceKind::Sprite(source),
                });
            }
        }

        Ok(resources)
    }
}

fn make_texture_element(
    file: &Path,
    texture_format: TextureElementTextureFormat,
    sampling_mode: Option<TextureElementSamplingMode>,
    wrapping_mode_u: Option<TextureElementWrappingMode>,
    wrapping_mode_v: Option<TextureElementWrappingMode>,
) -> Result<TextureElement, AnyError> {
    let image = ImageReader::open(file)?.with_guessed_format()?;
    let decoded = image.decode()?;

    let width = decoded.width();
    let height = decoded.height();

    if (u16::MAX as u32) < width || (u16::MAX as u32) < height {
        return Err(anyhow!("image too large"));
    }

    let size = TextureElementSize {
        width: width as u16,
        height: height as u16,
    };
    let sampling_mode = sampling_mode.unwrap_or(TextureElementSamplingMode::Bilinear);
    let wrapping_mode_u = wrapping_mode_u.unwrap_or(TextureElementWrappingMode::Clamp);
    let wrapping_mode_v = wrapping_mode_v.unwrap_or(TextureElementWrappingMode::Clamp);

    let data = match texture_format {
        TextureElementTextureFormat::RG32Uint => {
            return Err(anyhow!("RG32Uint format is not supported"));
        }
        TextureElementTextureFormat::RGBA32Uint => {
            return Err(anyhow!("RGBA32Uint format is not supported"));
        }
        TextureElementTextureFormat::RGBA32Float => {
            return Err(anyhow!("RGBA32Float format is not supported"));
        }
        TextureElementTextureFormat::RGBA8Unorm => decoded.into_rgba8().to_vec(),
        TextureElementTextureFormat::RGBA8UnormSrgb => decoded.into_rgba8().to_vec(),
    };

    Ok(TextureElement {
        data,
        size,
        texture_format,
        sampling_mode,
        wrapping_mode_u,
        wrapping_mode_v,
    })
}
