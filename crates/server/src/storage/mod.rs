use std::collections::HashMap;

use goosekv_protocol::data_type::GString;
use tracing::info;

use crate::storage::value::Value;

pub mod value;
pub mod request;
pub mod response;
pub mod handle;
pub mod router;
pub mod actor;

pub struct Storage {
    data: HashMap<GString, Value>,
}

impl Storage {
    pub fn new() -> Self {
        Self { data: Default::default() }
    }

    pub fn get(&self, key: GString) -> Option<Value> {
        info!("get");
        self.data.get(&key).cloned()
    }

    pub fn set(&mut self, key: GString, value: Value) -> Option<Value> {
        info!("set");
        self.data.insert(key, value)
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
