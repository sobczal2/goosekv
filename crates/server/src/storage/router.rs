use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use crate::storage::{
    handle::StorageHandle,
    request::{
        DeleteRequest,
        GetRequest,
        SetRequest,
    },
    response::{
        DeleteResponse,
        GetResponse,
        SetResponse,
    },
};

pub struct StorageRouter {
    handles: Box<[StorageHandle]>,
}

macro_rules! route {
    ($method:ident, $request:ty, $response:ty) => {
        pub async fn $method(&self, request: $request) -> $response {
            let mut hasher = DefaultHasher::default();
            request.key.hash(&mut hasher);
            let hash = hasher.finish();

            let route = hash as usize % self.handles.len();

            self.handles[route].$method(request).await
        }
    };
}

impl StorageRouter {
    route!(get, GetRequest, GetResponse);
    route!(set, SetRequest, SetResponse);
    route!(delete, DeleteRequest, DeleteResponse);
}

impl StorageRouter {
    pub fn new(handles: Box<[StorageHandle]>) -> Self {
        Self { handles }
    }
}
