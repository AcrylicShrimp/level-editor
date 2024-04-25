use lvl_resource::{ResourceFile, ResourceFileVersion};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResourceFileLoadError {
    #[error("failed to decode: {0}")]
    BincodeError(#[from] bincode::Error),
    #[error("unsupported version: {0}")]
    UnsupportedVersion(ResourceFileVersion),
}

pub fn load_resource_file(bytes: &[u8]) -> Result<ResourceFile, ResourceFileLoadError> {
    let resource_file = bincode::deserialize::<ResourceFile>(bytes)?;

    if resource_file.version() != ResourceFileVersion::V1 {
        return Err(ResourceFileLoadError::UnsupportedVersion(
            resource_file.version(),
        ));
    }

    Ok(resource_file)
}
