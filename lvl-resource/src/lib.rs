mod material_source;
mod mesh_source;
mod model_source;
mod shader_source;
mod texture_source;

pub use material_source::*;
pub use mesh_source::*;
pub use model_source::*;
pub use shader_source::*;
pub use texture_source::*;

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceFile {
    version: ResourceFileVersion,
    resources: BTreeMap<String, Resource>,
}

impl ResourceFile {
    pub fn new(version: ResourceFileVersion, resources: Vec<Resource>) -> Self {
        Self {
            version,
            resources: BTreeMap::from_iter(
                resources
                    .into_iter()
                    .map(|resource| (resource.name.clone(), resource)),
            ),
        }
    }

    pub fn version(&self) -> ResourceFileVersion {
        self.version
    }

    pub fn resources(&self) -> &BTreeMap<String, Resource> {
        &self.resources
    }

    pub fn find<T>(&self, name: &str) -> Option<&T>
    where
        T: FromResourceKind,
    {
        self.resources
            .get(name)
            .and_then(|resource| T::from(&resource.kind))
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Resource> {
        self.resources.get(name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceFileVersion {
    V1,
}

impl Display for ResourceFileVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::V1 => write!(f, "v1"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resource {
    pub name: String,
    pub kind: ResourceKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResourceKind {
    Material(MaterialSource),
    Mesh(MeshSource),
    Model(ModelSource),
    Shader(ShaderSource),
    Texture(TextureSource),
}

impl ResourceKind {
    pub fn as_material_source(&self) -> Option<&MaterialSource> {
        match self {
            Self::Material(material) => Some(material),
            _ => None,
        }
    }

    pub fn as_mesh_source(&self) -> Option<&MeshSource> {
        match self {
            Self::Mesh(mesh) => Some(mesh),
            _ => None,
        }
    }

    pub fn as_model_source(&self) -> Option<&ModelSource> {
        match self {
            Self::Model(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_shader_source(&self) -> Option<&ShaderSource> {
        match self {
            Self::Shader(shader) => Some(shader),
            _ => None,
        }
    }

    pub fn as_texture_source(&self) -> Option<&TextureSource> {
        match self {
            Self::Texture(texture) => Some(texture),
            _ => None,
        }
    }
}

pub trait FromResourceKind {
    fn from(kind: &ResourceKind) -> Option<&Self>;
}
