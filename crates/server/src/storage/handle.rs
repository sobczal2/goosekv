use futures::channel::oneshot;

use crate::storage::{
    request::{
        DeleteRequest,
        GetRequest,
        Request,
        SetRequest,
        UpdateRequest,
    },
    response::{
        DeleteResponse,
        GetResponse,
        SetResponse,
        UpdateResponse,
    },
};

macro_rules! handle_request {
    ($request_name:ident, $request:expr, $sender:expr) => {{
        let (sender, receiver) = oneshot::channel();
        $sender.send(Request::$request_name($request, sender)).await.unwrap();
        receiver.await.unwrap()
    }};
}

#[derive(Clone)]
pub struct StorageHandle {
    sender: async_channel::Sender<Request>,
}

impl StorageHandle {
    pub fn new(sender: async_channel::Sender<Request>) -> Self {
        Self { sender }
    }

    pub async fn get(&self, request: GetRequest) -> GetResponse {
        handle_request!(Get, request, self.sender)
    }

    pub async fn set(&self, request: SetRequest) -> SetResponse {
        handle_request!(Set, request, self.sender)
    }

    pub async fn delete(&self, request: DeleteRequest) -> DeleteResponse {
        handle_request!(Delete, request, self.sender)
    }

    pub async fn update(&self, request: UpdateRequest) -> UpdateResponse {
        handle_request!(Update, request, self.sender)
    }
}
