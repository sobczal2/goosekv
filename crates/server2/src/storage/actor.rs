use async_channel::{Receiver, Sender};
use tracing::info;

use crate::storage::{handle::StorageHandle, request::Request, response::{GetResponse, SetResponse}, Storage};

pub struct StorageActor {
    sender: Sender<Request>,
    receiver: Receiver<Request>,
    storage: Storage,
}

impl StorageActor {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::bounded(4);
        Self { sender, receiver, storage: Storage::new() }
    }

    pub fn handle(&self) -> StorageHandle {
        StorageHandle::new(self.sender.clone())
    }

    pub async fn run(self) {
        while let Ok(request) = self.receiver.recv().await {
            match request {
                Request::Get(get_request, respond) => {
                    let value = self.storage.get(get_request.key);
                    respond.send(GetResponse { value }).await.unwrap();
                },
                Request::Set(set_request, respond) => {
                    info!("set");
                    let original_value = self.storage.get(set_request.key);
                    respond.send(SetResponse { original_value }).await.unwrap();
                },
            }
        }
    }
}

impl Default for StorageActor {
    fn default() -> Self {
        Self::new()
    }
}
