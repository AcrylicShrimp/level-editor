use crate::processors::{ModelProcessor, Processor, TextureProcessor};
use anyhow::{anyhow, Context, Error as AnyError};
use log::{debug, info, warn};
use lvl_resource::{Resource, ResourceFile, ResourceFileVersion};
use serde::Deserialize;
use std::path::Path;

pub fn compile(
    input: Option<impl AsRef<Path>>,
    output: Option<impl AsRef<Path>>,
) -> Result<(), AnyError> {
    info!("compiling resources.");

    let input = match input {
        Some(input) => match input.as_ref().canonicalize() {
            Ok(input) => input,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(anyhow!(
                    "the input path `{}` does not exist",
                    input.as_ref().display()
                ));
            }
            Err(err) => {
                return Err(anyhow!(err)).with_context(|| {
                    format!(
                        "failed to canonicalize the input path `{}`",
                        input.as_ref().display()
                    )
                })
            }
        },
        None => std::env::current_dir()?,
    };

    if !input.is_dir() {
        return Err(anyhow!(
            "the input path `{}` must be a directory",
            input.display()
        ));
    }

    let output = match output {
        Some(output) => output.as_ref().to_owned(),
        None => {
            let output = std::env::current_dir()?.join("resource.res");
            warn!(
                "the output path is not specified, defaulting to `{}`.",
                output.display()
            );
            output
        }
    };

    if output.exists() {
        warn!(
            "the output path `{}` already exists, it will be overwritten.",
            output.display()
        );
    }

    let mut dirs = vec![input];
    let mut resources = Vec::new();

    loop {
        if dirs.is_empty() {
            break;
        }

        let mut added_dirs = Vec::new();

        for dir in &dirs {
            let dir_entries = dir.read_dir()?;

            for entry in dir_entries {
                let entry = entry?;
                let metadata = entry.metadata()?;

                if metadata.is_dir() {
                    let path = entry.path();
                    debug!("entry `{}` is a directory.", path.display());
                    added_dirs.push(path);
                    continue;
                }

                if !metadata.is_file() {
                    debug!(
                        "entry `{}` is not a file. skipping.",
                        entry.path().display()
                    );
                    continue;
                }

                debug!("entry `{}` is a file. processing.", entry.path().display());

                let processed = compile_single_file(&entry.path())?;
                resources.extend(processed);
            }
        }

        dirs = added_dirs;
    }

    let resource_file = ResourceFile::new(ResourceFileVersion::V1, resources);
    let resource_file_data = bincode::serialize(&resource_file)
        .with_context(|| format!("failed to serialize the resource file"))?;

    let output_dir = match output.parent() {
        Some(output_dir) => output_dir,
        None => return Err(anyhow!("the output path is not valid")),
    };
    std::fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "failed to create the output directory `{}`",
            output.display()
        )
    })?;
    std::fs::write(&output, &resource_file_data).with_context(|| {
        format!(
            "failed to write the resource file to `{}`",
            output.display()
        )
    })?;

    Ok(())
}

fn compile_single_file(file: &Path) -> Result<Vec<Resource>, AnyError> {
    let extension = match file.extension() {
        Some(extension) => extension,
        None => {
            debug!("the file `{}` has no extension. ignoring.", file.display());
            return Ok(vec![]);
        }
    };

    match extension.to_string_lossy().to_string().as_str() {
        extension if ModelProcessor::extension().contains(&extension) => {
            Ok(ModelProcessor::process(file, Some(&())).with_context(|| {
                format!("failed to process the file `{}` as a model", file.display())
            })?)
        }
        extension if TextureProcessor::extension().contains(&extension) => {
            let metadata = load_metadata(file)?;
            Ok(
                TextureProcessor::process(file, metadata.as_ref()).with_context(|| {
                    format!(
                        "failed to process the file `{}` as a texture",
                        file.display()
                    )
                })?,
            )
        }
        _ => {
            debug!(
                "the file `{}` has an unsupported extension. ignoring.",
                file.display()
            );
            return Ok(vec![]);
        }
    }
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
