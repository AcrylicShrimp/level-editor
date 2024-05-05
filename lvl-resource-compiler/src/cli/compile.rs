use crate::processors::{
    process_single_file, ModelProcessor, Processor, ShaderProcessor, TextureProcessor,
};
use anyhow::{anyhow, Context, Error as AnyError};
use log::{debug, error, info, warn};
use lvl_resource::{Resource, ResourceFile, ResourceFileVersion};
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

                let processed = match compile_single_file(&entry.path()) {
                    Ok(processed) => processed,
                    Err(err) => {
                        let mut errors = Vec::new();

                        for cause in err.chain() {
                            errors.push(format!("- {}", cause.to_string()));
                        }

                        error!(
                            "failed to process the file `{}`. error:\n{}",
                            entry.path().display(),
                            errors.join("\n")
                        );
                        continue;
                    }
                };

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

    info!("compilation finished.");

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
            let processed = process_single_file::<ModelProcessor>(file).with_context(|| {
                format!("failed to process the file `{}` as a model", file.display())
            })?;
            Ok(processed)
        }
        extension if ShaderProcessor::extension().contains(&extension) => {
            let processed = process_single_file::<ShaderProcessor>(file).with_context(|| {
                format!(
                    "failed to process the file `{}` as a shader",
                    file.display()
                )
            })?;
            Ok(processed)
        }
        extension if TextureProcessor::extension().contains(&extension) => {
            let processed = process_single_file::<TextureProcessor>(file).with_context(|| {
                format!(
                    "failed to process the file `{}` as a texture",
                    file.display()
                )
            })?;
            Ok(processed)
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
