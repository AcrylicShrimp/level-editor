use lvl_resource::SpriteSource;
use std::sync::Arc;
use wgpu::TextureView;

#[derive(Debug)]
pub struct Sprite {
    texture: Arc<TextureView>,
    mapping: SpriteMapping,
}

impl Sprite {
    pub fn load_from_source(
        mut texture_loader: impl FnMut(&str) -> Option<Arc<TextureView>>,
        source: &SpriteSource,
    ) -> Self {
        let texture = texture_loader(source.texture_name()).unwrap();

        Self {
            texture,
            mapping: SpriteMapping {
                min: source.mapping().min,
                max: source.mapping().max,
            },
        }
    }

    pub fn texture(&self) -> &Arc<TextureView> {
        &self.texture
    }

    pub fn mapping(&self) -> SpriteMapping {
        self.mapping
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteMapping {
    pub min: (u16, u16),
    pub max: (u16, u16),
}
