use std::{fmt, num::ParseIntError, str::FromStr};

use bytes::Bytes;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GString {
    value: Bytes,
}

impl GString {
    pub fn new() -> Self {
        Self { value: Bytes::new() }
    }

    pub fn copy_from_slice(data: &[u8]) -> Self {
        Self { value: Bytes::copy_from_slice(data) }
    }

    pub fn from_static(data: &'static [u8]) -> Self {
        Self { value: Bytes::from_static(data) }
    }

    pub fn bytes(&self) -> Bytes {
        self.value.clone()
    }
    
    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for GString {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for GString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GString").field("value", &String::from_utf8_lossy(&self.value)).finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GInteger {
    value: i64,
}

impl GInteger {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
    pub fn bytes(&self) -> Bytes {
        let value = self.value.to_string();
        Bytes::copy_from_slice(value.as_bytes())
    }
}

impl FromStr for GInteger {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse()?;
        Ok(Self { value })
    }
}
