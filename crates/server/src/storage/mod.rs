use std::collections::HashMap;

use goosekv_protocol::data_type::GString;

use crate::storage::value::Value;

pub mod actor;
pub mod handle;
pub mod request;
pub mod response;
pub mod router;
pub mod value;

pub struct Storage {
    data: HashMap<GString, Value>,
}

impl Storage {
    pub fn new() -> Self {
        Self { data: Default::default() }
    }

    pub fn get(&self, key: &GString) -> Option<Value> {
        self.data.get(key).cloned()
    }

    pub fn set(&mut self, key: GString, value: Value) -> Option<Value> {
        self.data.insert(key, value)
    }

    pub fn delete(&mut self, key: &GString) -> Option<Value> {
        self.data.remove(key)
    }

    pub fn update<F, R>(&mut self, key: GString, f: F) -> Option<R>
    where F: FnOnce(&mut Value) -> R
    {
        let entry = self.data.entry(key).and_modify(f);

        todo!()
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
