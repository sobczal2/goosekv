use bytes::Bytes;
use goosekv_protocol::data_type::{
    GInteger,
    GString,
};

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
    pub fn from_gstring(data: GString) -> Self {
        if let Ok(ginteger) = ginteger_from_gstring(data.clone()) {
            return Self::Integer(ginteger);
        }

        Self::String(data)
    }

    pub fn bytes(&self) -> Bytes {
        match self {
            Data::String(gstring) => gstring.bytes(),
            Data::Integer(ginteger) => ginteger.bytes(),
        }
    }

    pub fn to_gstring(&self) -> GString {
        match self {
            Data::String(gstring) => gstring.clone(),
            Data::Integer(ginteger) => GString::copy_from_slice(ginteger.bytes().as_ref()),
        }
    }
}

fn ginteger_from_gstring(data: GString) -> Result<GInteger, ()> {
    println!("{data:?}");
    let bytes = data.bytes();
    let utf8 = str::from_utf8(&bytes).map_err(|_| ())?;
    utf8.parse().map_err(|_| ())
}
