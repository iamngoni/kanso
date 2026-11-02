//! An ACP agent as the one that runs an agent task: a session of its own on
//! the agent's process, reaching the task's change set through the notes
//! server over MCP. The phone has no such agent; it runs tasks with an API
//! key ([`kanso_engine::agent_task::runner::ApiDriver`]).

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_ai::agent::Conversation;
use kanso_ai::{Event, Prompt};
use kanso_engine::agent_task::OpenTask;
use kanso_engine::agent_task::runner::TaskDriver;
use kanso_store::Store;

pub struct AcpDriver(pub Arc<Conversation>);

impl TaskDriver for AcpDriver {
    fn first_prompt(&self, task: &OpenTask) -> Prompt {
        Prompt {
            user: kanso_engine::agent_task::user_text(task, false),
            ..kanso_ai::agent::task_prompt(&task.spec.goal)
        }
    }

    // The reply continues the same ACP session.
    fn reply_prompt(&self, _task: &OpenTask, _said: &str, reply: &str) -> Prompt {
        Prompt {
            system: "The user replied to your question.".to_owned(),
            user: reply.trim().to_owned(),
        }
    }

    fn begin(
        &self,
        prompt: &Prompt,
        _task: &OpenTask,
        _store: Arc<Store>,
        mut progress: Box<dyn FnMut() + Send>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, String> {
        self.0
            .ask(prompt, &[], Box::new(move |_| progress()), cancel)
            .map_err(|e| e.to_string())
    }

    fn queues(&self) -> bool {
        true
    }
}
