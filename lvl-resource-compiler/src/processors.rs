mod model_processor;
mod texture_processor;

pub use model_processor::*;
pub use texture_processor::*;

use anyhow::Error as AnyError;
use lvl_resource::Resource;
use std::path::Path;

pub trait Processor {
    fn new() -> Self;
    fn extension(&self) -> &'static [&'static str];
    fn process(&self, file: &Path) -> Result<Vec<Resource>, AnyError>;
}
