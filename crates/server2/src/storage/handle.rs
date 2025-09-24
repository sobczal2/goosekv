use crate::storage::{request::{GetRequest, Request, SetRequest}, response::{GetResponse, SetResponse}};

#[derive(Clone)]
pub struct StorageHandle {
    sender: async_channel::Sender<Request>
}

impl StorageHandle {
    pub fn new(sender: async_channel::Sender<Request>) -> Self {
        Self { sender }
    }

    pub async fn get(&self, request: GetRequest) -> GetResponse {
        let (sender, receiver) = async_channel::bounded(1);
        self.sender.send(Request::Get(request, sender)).await.unwrap();
        receiver.recv().await.unwrap()
    }

    pub async fn set(&self, request: SetRequest) -> SetResponse {
        let (sender, receiver) = async_channel::bounded(1);
        self.sender.send(Request::Set(request, sender)).await.unwrap();
        receiver.recv().await.unwrap()
    }
}
