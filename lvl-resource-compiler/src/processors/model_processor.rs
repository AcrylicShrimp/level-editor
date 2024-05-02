use super::Processor;
use anyhow::Error as AnyError;
use lvl_pmx::Pmx;
use lvl_resource::Resource;
use std::path::Path;

pub struct ModelProcessor {}

impl Processor for ModelProcessor {
    fn new() -> Self {
        Self {}
    }

    fn extension(&self) -> &'static [&'static str] {
        &["pmx"]
    }

    fn process(&self, file: &Path) -> Result<Vec<Resource>, AnyError> {
        let content = std::fs::read(file)?;
        let pmx = Pmx::parse(&content)?;

        todo!()
    }
}

mod pmx {
    use super::*;
    use anyhow::anyhow;
    use lvl_pmx::PmxTexture;
    use lvl_resource::{MeshSource, TextureSource};

    pub fn make_mesh() -> Result<MeshSource, AnyError> {
        todo!()
    }

    pub fn make_texture(file: &Path, texture: &PmxTexture) -> Result<TextureSource, AnyError> {
        let texture_path = file.join(&texture.path);

        if !texture_path.is_file() {
            return Err(anyhow!(
                "the path `{}` is not a file (texture path is `{}`)",
                texture_path.display(),
                texture.path
            ));
        }

        todo!()
    }
}
