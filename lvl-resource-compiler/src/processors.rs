mod pmx_model_animation_processor;
mod pmx_model_processor;
mod shader_processor;
mod texture_processor;

pub use pmx_model_animation_processor::*;
pub use pmx_model_processor::*;
pub use shader_processor::*;
pub use texture_processor::*;

use anyhow::{Context, Error as AnyError};
use log::{debug, warn};
use lvl_resource::Resource;
use serde::Deserialize;
use std::path::Path;

pub trait Processor {
    type Metadata: for<'de> Deserialize<'de>;

    fn extension() -> &'static [&'static str];
    fn process(file: &Path, metadata: Option<&Self::Metadata>) -> Result<Vec<Resource>, AnyError>;
}

pub fn process_single_file<P: Processor>(file: &Path) -> Result<Vec<Resource>, AnyError> {
    let extension = match file.extension() {
        Some(extension) => extension.to_string_lossy().to_string(),
        None => {
            debug!("the file `{}` has no extension. ignoring.", file.display());
            return Ok(vec![]);
        }
    };

    if !P::extension().contains(&extension.as_str()) {
        debug!(
            "the file `{}` has an unsupported extension. ignoring.",
            file.display()
        );
        return Ok(vec![]);
    }

    let metadata = load_metadata::<P::Metadata>(file)?;
    P::process(file, metadata.as_ref())
}

fn load_metadata<T>(file_path: &Path) -> Result<Option<T>, AnyError>
where
    T: for<'de> Deserialize<'de>,
{
    let metadata_extension = match file_path.extension() {
        Some(extension) => format!("{}.meta", extension.to_string_lossy().to_string()),
        None => "meta".to_owned(),
    };
    let metadata_path = file_path.with_extension(metadata_extension);

    if !metadata_path.is_file() {
        debug!(
            "the metadata `{}` does not exist. skipping.",
            metadata_path.display()
        );
        return Ok(None);
    }

    let content = match std::fs::read_to_string(&metadata_path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            debug!(
                "the metadata `{}` does not exist. skipping.",
                metadata_path.display()
            );
            return Ok(None);
        }
        Err(err) => {
            warn!(
                "failed to read the metadata `{}`: {}",
                metadata_path.display(),
                err
            );
            return Ok(None);
        }
    };

    let metadata = serde_json::from_str(&content)
        .with_context(|| format!("parsing the metadata `{}`", metadata_path.display()))?;

    Ok(Some(metadata))
}
