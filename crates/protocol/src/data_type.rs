use std::fmt::{self, Display};

use bytes::Bytes;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GString {
    value: Bytes,
}

impl GString {
    pub fn copy_from_slice(data: &[u8]) -> Self {
        Self { value: Bytes::copy_from_slice(data) }
    }

    pub fn bytes(&self) -> Bytes {
        self.value.clone()
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
    pub fn bytes(&self) -> Bytes {
        let value = self.value.to_string();
        Bytes::copy_from_slice(value.as_bytes())
    }
}
