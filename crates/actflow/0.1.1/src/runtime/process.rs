use std::sync::Arc;

use tokio::runtime::Runtime;

use crate::{
    ActflowError, Result,
    common::{Queue, Vars},
    dispatcher::Dispatcher,
    events::{GraphEvent, WorkflowEvent},
    model::WorkflowModel,
    runtime::{Channel, ChannelOptions, Context, channel::ChannelEvent},
    store::Store,
    utils,
    workflow::Workflow,
};

const COMMAND_QUEUE_SIZE: usize = 100;

pub type ProcessId = String;

#[derive(Debug, Clone)]
pub enum WorkflowCommand {
    Start,
    Abort,
}

#[derive(Clone)]
pub struct Process {
    id: ProcessId,
    wid: String,
    dispatcher: Arc<Dispatcher>,
    command_queue: Arc<Queue<WorkflowCommand>>,
    channel: Arc<Channel>,
}

impl Process {
    pub fn new(
        model: &WorkflowModel,
        store: Arc<Store>,
        channel: Arc<Channel>,
        runtime: Arc<Runtime>,
    ) -> Result<Arc<Process>> {
        let pid = utils::longid();

        let proc = store.procs().find(&pid);
        if proc.is_ok() {
            return Err(ActflowError::Process(format!(
                "proc_id({pid}) is duplicated in running process list"
            )));
        }

        let workflow = Workflow::try_from(model)?;

        let command_queue = Queue::new(COMMAND_QUEUE_SIZE);

        let ctx = Arc::new(Context::new(pid.to_owned(), channel.clone()));

        // set env variables
        model.env.iter().for_each(|(k, v)| ctx.env().set(k.clone(), v.clone()));

        let dispatcher = Arc::new(Dispatcher::new(
            ctx.clone(),
            Arc::new(workflow),
            command_queue.clone(),
            runtime.clone(),
        ));

        Ok(Arc::new(Process {
            id: pid,
            wid: model.id.clone(),
            dispatcher,
            command_queue,
            channel,
        }))
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn wid(&self) -> &str {
        &self.wid
    }

    pub fn start(&self) {
        self.dispatcher.start();

        let dispatcher = self.dispatcher.clone();

        ChannelEvent::channel(self.channel.clone(), ChannelOptions::with_pid(self.id.to_owned())).on_event(move |event| {
            let dispatcher = dispatcher.clone();
            match &event.event {
                GraphEvent::Workflow(e) => match e {
                    WorkflowEvent::Succeeded | WorkflowEvent::Failed(_) | WorkflowEvent::Aborted(_) => {
                        dispatcher.stop();
                    }
                    _ => {}
                },
                _ => {}
            }
        });

        // Send start command to the command queue
        let _ = self.command_queue.send(WorkflowCommand::Start);
    }

    pub fn abort(&self) {
        let _ = self.command_queue.send(WorkflowCommand::Abort);
    }

    pub fn get_outputs(&self) -> Vars {
        self.dispatcher.outputs()
    }

    pub fn is_complete(&self) -> bool {
        self.dispatcher.is_complete()
    }
}
