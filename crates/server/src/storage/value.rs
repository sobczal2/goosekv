use bytes::Bytes;
use goosekv_protocol::data_type::GString;

#[derive(Debug, Clone)]
pub struct Value {
    pub data: Data,
}

#[derive(Debug, Clone)]
pub enum Data {
    String(GString),
}

impl Data {
    pub fn bytes(&self) -> Bytes {
        match self {
            Data::String(gstring) => gstring.bytes(),
        }
    }
}
