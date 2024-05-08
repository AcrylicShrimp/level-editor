use crate::processors::{
    process_single_file, ModelProcessor, PmxModelProcessor, Processor, ShaderProcessor,
    TextureProcessor,
};
use anyhow::{anyhow, Context, Error as AnyError};
use log::{debug, error, info, warn};
use lvl_resource::{Resource, ResourceFile, ResourceFileVersion};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

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

    let gitignore_path = input.join(".gitignore");
    let gitignore_file = if gitignore_path.is_file() {
        info!(
            "gitignore file found at `{}`. it will be used to exclude directories and/or files.",
            gitignore_path.display()
        );
        Some(gitignore::File::new(&gitignore_path)?)
    } else {
        None
    };

    let included_dirs = gitignore_file
        .map(|file| file.included_files())
        .transpose()?;
    let included_dirs: Option<HashSet<PathBuf>> = match included_dirs {
        Some(included_dirs) => Some(HashSet::from_iter(included_dirs)),
        None => None,
    };

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
                let entry_path = entry.path();
                let metadata = entry.metadata()?;

                if metadata.is_dir() {
                    if let Some(included_dirs) = &included_dirs {
                        if !included_dirs.contains(&entry_path) {
                            debug!(
                                "entry `{}` is excluded by the .gitignore file.",
                                entry_path.display()
                            );
                            continue;
                        }
                    }

                    if let Some(name) = entry_path.file_name() {
                        if name.eq_ignore_ascii_case(".git") {
                            debug!(
                                "entry `{}` is a .git directory. skipping.",
                                entry_path.display()
                            );
                            continue;
                        }
                    }

                    debug!("entry `{}` is a directory.", entry_path.display());
                    added_dirs.push(entry_path);
                    continue;
                }

                if !metadata.is_file() {
                    debug!("entry `{}` is not a file. skipping.", entry_path.display());
                    continue;
                }

                if entry.file_name().eq_ignore_ascii_case(".gitignore") {
                    debug!(
                        "entry `{}` is a .gitignore file. skipping.",
                        entry_path.display()
                    );
                    continue;
                }

                if let Some(included_dirs) = &included_dirs {
                    if !included_dirs.contains(&entry_path) {
                        debug!(
                            "entry `{}` is excluded by the .gitignore file.",
                            entry_path.display()
                        );
                        continue;
                    }
                }

                debug!("entry `{}` is a file. processing.", entry_path.display());

                let processed = match compile_single_file(&entry_path) {
                    Ok(processed) => processed,
                    Err(err) => {
                        let mut errors = Vec::new();

                        for cause in err.chain() {
                            errors.push(format!("- {}", cause.to_string()));
                        }

                        error!(
                            "failed to process the file `{}`. error:\n{}",
                            entry_path.display(),
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
        // extension if ModelProcessor::extension().contains(&extension) => {
        //     let processed = process_single_file::<ModelProcessor>(file).with_context(|| {
        //         format!("failed to process the file `{}` as a model", file.display())
        //     })?;
        //     Ok(processed)
        // }
        extension if PmxModelProcessor::extension().contains(&extension) => {
            let processed = process_single_file::<PmxModelProcessor>(file).with_context(|| {
                format!(
                    "failed to process the file `{}` as a PMX model",
                    file.display()
                )
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
