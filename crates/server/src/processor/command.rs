use goosekv_protocol::command::{Key, Value};

#[derive(Debug)]
pub enum Command {
    Get(Key),
    Set(Key, Value),
}

pub enum CommandResponse {
    Empty,
    Value(Value),
}

impl CommandResponse {
    pub fn as_value(self) -> Value {
        match self {
            CommandResponse::Value(value) => value,
            CommandResponse::Empty => panic!(),
        }
    }
}
