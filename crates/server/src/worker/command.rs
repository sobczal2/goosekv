use std::fmt::{
    self,
    Debug,
    Formatter,
};

use glommio::channels::shared_channel::SharedSender;
use goosekv_protocol::command::{
    Key,
    Value,
};

pub enum WorkerCommand {
    Get(Key, SharedSender<GetResponse>),
    Set(Key, Value, SharedSender<SetResponse>),
}

impl Debug for WorkerCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Get(key, _) => f.debug_tuple("Get").field(key).finish(),
            Self::Set(key, value, _) => f.debug_tuple("Set").field(key).field(value).finish(),
        }
    }
}

pub struct GetResponse {
    pub value: Option<Value>,
}

pub struct SetResponse;
