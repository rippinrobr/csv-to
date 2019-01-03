//! The Cache Module
//! In order to help speed up the process of code generation the Cache
//! module stores information learned about the input data during the
//! parsing process.
use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::ColumnDef;

pub mod json;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CacheType {
    Db
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cache {
    name: String,
    cache_type: CacheType,
    data_definitions: Vec<DataDefinition>,
}

impl Cache {
    pub fn new(name: String, cache_type: CacheType) -> Self {
        Self {
            name,
            cache_type,
            data_definitions: Vec::new(),
        }
    }

    pub fn add_data_definition(&mut self, data_def: DataDefinition) {
        self.data_definitions.push(data_def);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataDefinition {
    object_name: String,
    columns: Vec<ColumnDef>
}

impl DataDefinition {
    pub fn new(object_name: String, columns: Vec<ColumnDef>) -> Self {
        Self {
            object_name,
            columns,
        }
    }
}
// A trait for interacting with a cache
pub trait CacheService {
    fn read(self, name: String) -> Result<Cache, failure::Error>;
    fn write(self, cache: Cache) -> Result<(), failure::Error>;
}