use std::{hash::{DefaultHasher, Hash, Hasher}};

use crate::storage::{handle::StorageHandle, request::{GetRequest, SetRequest}, response::{GetResponse, SetResponse}};

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
    pub async fn get(&self, request: GetRequest) -> GetResponse {
        let mut hasher = DefaultHasher::default();
        request.key.hash(&mut hasher);
        let hash = hasher.finish();

        let route = hash as usize % self.handles.len();

        self.handles[route].get(request).await
    }
    route!(set, SetRequest, SetResponse);
}

impl StorageRouter {
    pub fn new(handles: Box<[StorageHandle]>) -> Self {
        Self { handles }
    }
}
