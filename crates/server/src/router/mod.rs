use std::{cell::RefCell, rc::Rc};

use futures::{future::{select, Either}, stream::select_all, FutureExt, Stream, StreamExt};
use glommio::{channels::{channel_mesh::{FullMesh, MeshBuilder, Receivers, Senders}, local_channel::{self, LocalReceiver, LocalSender}, shared_channel::{self, ConnectedReceiver, ConnectedSender, SharedReceiver, SharedSender}}, executor, spawn_local, yield_if_needed};
use tracing::info;

struct Message<RQ, RS>
    where RS: Send
{
    request: RQ,
    respond: SharedSender<RS>,
}

struct LocalMessage<RQ, RS>
{
    request: RQ,
    respond: LocalSender<RS>,
}

pub struct Router<RQ, RS>
    where
        RQ: Send,
        RS: Send
{
    mesh: FullMesh<Message<RQ, RS>>,
}

impl<RQ, RS> Clone for Router<RQ, RS>
    where
        RQ: Send,
        RS: Send
{
    fn clone(&self) -> Self {
        Self { mesh: self.mesh.clone() }
    }
}

impl<RQ, RS> Router<RQ, RS>
    where
        RQ: Send + 'static,
        RS: Send + 'static
{
    pub fn new(size: usize) -> Self {
        Self { mesh: MeshBuilder::full(size, 32) }
    }
}

impl<RQ, RS> Router<RQ, RS>
    where
        RQ: Send + 'static,
        RS: Send + 'static
{
    pub async fn join(self) -> (SourceRouter<RQ, RS>, TargetRouter<RQ, RS>) {
        let (senders, mut receivers) = self.mesh.join().await.unwrap();
        let (local_sender, local_receiver) = local_channel::new_unbounded();
        let streams = receivers.streams().into_iter().map(|(_, s)| s).collect::<Vec<_>>();
        let stream = Box::new(select_all(streams));
        (SourceRouter { senders, local: local_sender }, TargetRouter { stream: Rc::new(RefCell::new(stream)), local: Rc::new(local_receiver) })
    }
}

pub struct SourceRouter<RQ, RS>
    where
        RQ: Send,
        RS: Send
{
    senders: Senders<Message<RQ, RS>>,
    local: LocalSender<LocalMessage<RQ, RS>>
}

impl<RQ, RS> SourceRouter<RQ, RS>
    where
        RQ: Send + 'static,
        RS: Send + 'static
{
    pub async fn send(&self, to: usize,  request: RQ) -> RS {
        info!("to: {to}");
        if to != self.senders.producer_id().unwrap() {
            let (sender, receiver) = shared_channel::new_bounded(1);
            let message = Message { request, respond: sender };
            self.senders.send_to(to, message).await.unwrap();
            let receiver = receiver.connect().await;
            receiver.recv().await.unwrap()
        } else {
            let (sender, receiver) = local_channel::new_bounded(1);
            let message = LocalMessage { request, respond: sender };
            self.local.send(message).await.unwrap();
            receiver.recv().await.unwrap()
        }
    }

    pub fn targets(&self) -> usize {
        self.senders.nr_consumers()
    }
}

pub struct TargetRouter<RQ, RS>
    where
        RQ: Send,
        RS: Send
{
    stream: Rc<RefCell<dyn Stream<Item = Message<RQ, RS>> + Unpin>>,
    local: Rc<LocalReceiver<LocalMessage<RQ, RS>>>,
}

impl<RQ, RS> TargetRouter<RQ, RS>
    where
        RQ: Send + 'static,
        RS: Send + 'static,
{
    pub async fn handle<H, F>(&mut self, handle: H)
    where H: FnOnce(RQ) -> F,
    F: Future<Output = RS>
    {
        let local = self.local.clone();
        let stream = self.stream.clone();
        let local_future = spawn_local(async move { local.recv().await.unwrap() });
        let remote_future = spawn_local(async move { 
            let mut borrowed = loop {
                match stream.try_borrow_mut() {
                    Ok(borrowed) => break borrowed,
                    Err(_) => {
                        yield_if_needed().await;
                    },
                }

            };
            borrowed.next().await.unwrap()
        });
        let select = select(local_future, remote_future);
        match select.await {
            Either::Left((local_message, _)) => {
                let response = handle(local_message.request).await;
                local_message.respond.send(response).await.unwrap();
            },
            Either::Right((remote_message, _)) => {
                let response = handle(remote_message.request).await;
                let respond = remote_message.respond.connect().await;
                respond.send(response).await.unwrap();
            },
            _ => unreachable!()
        }
    }
}
