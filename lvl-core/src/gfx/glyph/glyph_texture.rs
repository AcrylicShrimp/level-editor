use crate::gfx::elements::{Font, Texture};
use std::sync::Arc;
use wgpu::{
    Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub struct GlyphTexture {
    texture: Arc<Texture>,
    texture_view: Arc<TextureView>,
    font: Arc<Font>,
    offset_x: u16,
    offset_y: u16,
    line_height: u16,
}

impl GlyphTexture {
    const TEXTURE_SIZE: u16 = 2048;

    pub fn new(font: Arc<Font>, device: &Device) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("glyph-texture"),
            size: Extent3d {
                width: Self::TEXTURE_SIZE as u32,
                height: Self::TEXTURE_SIZE as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 0,
            sample_count: 0,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture: Arc::new(Texture::new(
                Self::TEXTURE_SIZE,
                Self::TEXTURE_SIZE,
                texture,
            )),
            texture_view: Arc::new(texture_view),
            font,
            offset_x: 0,
            offset_y: 0,
            line_height: 0,
        }
    }

    pub fn texture(&self) -> Arc<Texture> {
        self.texture.clone()
    }

    pub fn texture_view(&self) -> Arc<TextureView> {
        self.texture_view.clone()
    }

    pub fn font(&self) -> Arc<Font> {
        self.font.clone()
    }

    pub fn bake_glyph(
        &mut self,
        sdf_width: u16,
        sdf_height: u16,
        sdf: &[u8],
        queue: &Queue,
    ) -> Option<GlyphTexelMapping> {
        if 2048 < self.offset_y + sdf_height {
            return None;
        }

        if 2048 < self.offset_x + sdf_width {
            self.offset_x = 0;
            self.offset_y += self.line_height;
            self.line_height = sdf_height;

            if 2048 < self.offset_y + sdf_height {
                return None;
            }
        }

        let mapping = GlyphTexelMapping {
            min_x: self.offset_x,
            max_x: (self.offset_x + sdf_width),
            min_y: self.offset_y,
            max_y: (self.offset_y + sdf_height),
        };

        queue.write_texture(
            ImageCopyTexture {
                texture: self.texture.handle(),
                mip_level: 0,
                origin: Origin3d {
                    x: self.offset_x as u32,
                    y: self.offset_y as u32,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            &sdf,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(sdf_width as u32),
                rows_per_image: Some(sdf_height as u32),
            },
            Extent3d {
                width: sdf_width as u32,
                height: sdf_height as u32,
                ..Default::default()
            },
        );

        self.offset_x += sdf_width;
        self.line_height = self.line_height.max(sdf_height);

        Some(mapping)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphTexelMapping {
    pub min_x: u16,
    pub max_x: u16,
    pub min_y: u16,
    pub max_y: u16,
}
