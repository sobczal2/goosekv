use bytes::Bytes;
use goosekv_protocol::data_type::{GInteger, GString};

#[derive(Debug, Clone)]
pub struct Value {
    pub data: Data,
}

#[derive(Debug, Clone)]
pub enum Data {
    String(GString),
    Integer(GInteger),
}

impl Data {
    pub fn bytes(&self) -> Bytes {
        match self {
            Data::String(gstring) => gstring.bytes(),
            Data::Integer(ginteger) => ginteger.bytes(),
        }
    }
}
