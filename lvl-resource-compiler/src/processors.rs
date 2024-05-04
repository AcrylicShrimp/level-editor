mod model_processor;
mod texture_processor;

pub use model_processor::*;
pub use texture_processor::*;

use anyhow::Error as AnyError;
use lvl_resource::Resource;
use serde::Deserialize;
use std::path::Path;

pub trait Processor {
    type Metadata: for<'de> Deserialize<'de>;

    fn extension() -> &'static [&'static str];
    fn process(file: &Path, metadata: Option<&Self::Metadata>) -> Result<Vec<Resource>, AnyError>;
}
