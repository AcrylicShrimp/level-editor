use crate::gfx::GfxContext;
use lvl_resource::{TextureElement, TextureElementTextureFormat};
use wgpu::{
    Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages,
};

#[derive(Debug)]
pub struct Texture {
    width: u16,
    height: u16,
    handle: wgpu::Texture,
}

impl Texture {
    pub fn load_from_source(source: &TextureElement, gfx_ctx: &GfxContext) -> Self {
        let handle = gfx_ctx.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: source.size.width as u32,
                height: source.size.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: match source.texture_format {
                TextureElementTextureFormat::RG32Uint => TextureFormat::Rg32Uint,
                TextureElementTextureFormat::RGBA32Uint => TextureFormat::Rgba32Uint,
                TextureElementTextureFormat::RGBA32Float => TextureFormat::Rgba32Float,
                TextureElementTextureFormat::RGBA8Unorm => TextureFormat::Rgba8Unorm,
            },
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        gfx_ctx.queue.write_texture(
            ImageCopyTexture {
                texture: &handle,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &source.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(match source.texture_format {
                    TextureElementTextureFormat::RG32Uint => 8 * source.size.width as u32,
                    TextureElementTextureFormat::RGBA32Uint => 16 * source.size.width as u32,
                    TextureElementTextureFormat::RGBA32Float => 16 * source.size.width as u32,
                    TextureElementTextureFormat::RGBA8Unorm => 4 * source.size.width as u32,
                }),
                rows_per_image: None,
            },
            Extent3d {
                width: source.size.width as u32,
                height: source.size.height as u32,
                depth_or_array_layers: 1,
            },
        );

        Self {
            width: source.size.width,
            height: source.size.height,
            handle,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn handle(&self) -> &wgpu::Texture {
        &self.handle
    }
}
