use futures::channel::oneshot;
use goosekv_protocol::data_type::GString;

use crate::storage::{response::{GetResponse, SetResponse}, value::Value};

pub enum Request {
    Get(GetRequest, oneshot::Sender<GetResponse>),
    Set(SetRequest, oneshot::Sender<SetResponse>),
}

pub struct GetRequest {
    pub key: GString,
}

pub struct SetRequest {
    pub key: GString,
    pub value: Value,
}

