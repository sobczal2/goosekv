use std::{fmt, rc::Rc};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GString {
    value: Rc<[u8]>,
}

impl GString {
    pub fn copy_from_slice(data: &[u8]) -> Self {
        Self { value: data.into() }
    }
}

impl fmt::Debug for GString{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GString").field("value", &String::from_utf8_lossy(&self.value)).finish()
    }
}


