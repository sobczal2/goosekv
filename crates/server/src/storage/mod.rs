use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
    sync::Arc,
};

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

    /// Update a key and return new value. Returns updated value.
    ///
    /// Update function runs even if the key is not yet present.
    pub fn update(
        &mut self,
        key: GString,
        f: Arc<dyn Fn(Option<Value>) -> Option<Value> + Send + Sync>,
    ) -> Option<Value> {
        let entry = self.data.entry(key);

        match entry {
            Entry::Occupied(mut occupied_entry) => {
                let updated = f(Some(occupied_entry.get().clone()));
                match updated {
                    Some(updated) => {
                        *occupied_entry.get_mut() = updated.clone();
                        Some(updated)
                    }
                    None => {
                        occupied_entry.remove_entry();
                        None
                    }
                }
            }
            Entry::Vacant(vacant_entry) => {
                let updated = f(None);
                match updated {
                    Some(updated) => {
                        vacant_entry.insert(updated.clone());
                        Some(updated)
                    }
                    None => None,
                }
            }
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
