use crate::{FromResourceKind, ResourceKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderSource {
    render_type: ShaderRenderType,
    source: String,
    vs_main: String,
    fs_main: String,
    // TODO: add more detailed reflection data
}

impl ShaderSource {
    pub fn new(
        render_type: ShaderRenderType,
        source: String,
        vs_main: String,
        fs_main: String,
    ) -> Self {
        Self {
            render_type,
            source,
            vs_main,
            fs_main,
        }
    }
}

impl FromResourceKind for ShaderSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::Shader(shader) => Some(shader),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShaderRenderType {
    Opaque,
    Transparent,
}
