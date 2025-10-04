use async_channel::{
    Receiver,
    Sender,
};
use tracing::debug;

use crate::storage::{
    Storage,
    handle::StorageHandle,
    request::Request,
    response::{
        DeleteResponse,
        GetResponse,
        SetResponse,
        UpdateResponse,
    },
};

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

    pub async fn run(mut self) {
        while let Ok(request) = self.receiver.recv().await {
            match request {
                Request::Get(get_request, respond) => {
                    debug!("get value for key: {:?}", get_request.key);
                    let value = self.storage.get(&get_request.key);
                    respond.send(GetResponse { value }).unwrap();
                }
                Request::Set(set_request, respond) => {
                    debug!("set value for key: {:?}", set_request.key);
                    let original_value = self.storage.set(set_request.key, set_request.value);
                    respond.send(SetResponse { original_value }).unwrap();
                }
                Request::Delete(delete_request, respond) => {
                    debug!("delete value for key: {:?}", delete_request.key);
                    let deleted = self.storage.delete(&delete_request.key);
                    respond.send(DeleteResponse { deleted }).unwrap()
                }
                Request::Update(update_request, respond) => {
                    debug!("update value for key: {:?}", update_request.key);
                    let updated = self.storage.update(update_request.key, update_request.f);
                    respond.send(UpdateResponse { updated }).unwrap()
                }
            }
        }
    }
}

impl Default for StorageActor {
    fn default() -> Self {
        Self::new()
    }
}
