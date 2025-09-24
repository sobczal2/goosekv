use goosekv_protocol::data_type::GString;

use crate::storage::{response::{GetResponse, SetResponse}, value::Value};

pub enum Request {
    Get(GetRequest, async_channel::Sender<GetResponse>),
    Set(SetRequest, async_channel::Sender<SetResponse>),
}

pub struct GetRequest {
    pub key: GString,
}

pub struct SetRequest {
    pub key: GString,
    pub value: Value,
}

