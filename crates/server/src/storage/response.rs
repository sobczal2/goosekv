use crate::storage::value::Value;

#[derive(Debug)]
pub struct GetResponse {
    pub value: Option<Value>,
}

#[derive(Debug)]
pub struct SetResponse {
    pub original_value: Option<Value>,
}

#[derive(Debug)]
pub struct DeleteResponse {
    pub deleted: Option<Value>,
}

#[derive(Debug)]
pub struct UpdateResponse {
    pub updated: Option<Value>,
}
