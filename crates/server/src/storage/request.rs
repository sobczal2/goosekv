use std::sync::Arc;

use futures::channel::oneshot;
use goosekv_protocol::data_type::GString;

use crate::storage::{
    response::{
        DeleteResponse,
        GetResponse,
        SetResponse,
        UpdateResponse,
    },
    value::Value,
};

pub enum Request {
    Get(GetRequest, oneshot::Sender<GetResponse>),
    Set(SetRequest, oneshot::Sender<SetResponse>),
    Delete(DeleteRequest, oneshot::Sender<DeleteResponse>),
    Update(UpdateRequest, oneshot::Sender<UpdateResponse>),
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

pub struct UpdateRequest {
    pub key: GString,
    pub f: Arc<dyn Fn(Option<Value>) -> Option<Value> + Send + Sync>,
}
