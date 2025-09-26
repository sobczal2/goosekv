use glommio::channels::local_channel::LocalSender;

use crate::processor::command::{
    ProcessCommand,
    ProcessorCommand,
};

pub struct ProcessorHandle {
    sender: LocalSender<ProcessorCommand>,
}

impl ProcessorHandle {
    pub fn new(sender: LocalSender<ProcessorCommand>) -> Self {
        ProcessorHandle { sender }
    }

    pub async fn process(&mut self, command: ProcessCommand) {
        self.sender.send(ProcessorCommand::Process(command)).await.unwrap();
    }
}
