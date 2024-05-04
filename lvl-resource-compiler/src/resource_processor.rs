use crate::processors::Processor;
use anyhow::{anyhow, Error as AnyError};
use lvl_resource::Resource;
use std::{collections::BTreeMap, path::Path, sync::Arc};

pub struct ResourceProcessor {
    resources: Vec<Resource>,
}

impl ResourceProcessor {
    pub fn process_single_resource(&mut self, path: &Path) -> Result<(), AnyError> {
        if !path.is_file() {
            return Err(anyhow!("the given path `{}` is not a file", path.display()));
        }

        todo!()
    }
}
