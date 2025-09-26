use futures::channel::oneshot;
use goosekv_protocol::data_type::GString;

use crate::storage::{
    response::{
        DeleteResponse,
        GetResponse,
        SetResponse,
    },
    value::Value,
};

pub enum Request {
    Get(GetRequest, oneshot::Sender<GetResponse>),
    Set(SetRequest, oneshot::Sender<SetResponse>),
    Delete(DeleteRequest, oneshot::Sender<DeleteResponse>),
}

pub struct GetRequest {
    pub key: GString,
}

pub struct SetRequest {
    pub key: GString,
    pub value: Value,
}

pub struct DeleteRequest {
    pub key: GString,
}
