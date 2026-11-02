//! A tool whose schema is given and whose arguments stay JSON: what an
//! agent task registers for each of Kanso's notes tools, so the model sees
//! the same definitions an ACP agent does.

use std::sync::Arc;

use agent_runtime::{Tool, ToolCall, ToolDefinition, ToolOutput};
use serde_json::Value;

/// What a raw tool runs: its JSON arguments in, its JSON answer out. Blocking
/// (it may read the store).
pub type RawRun = Arc<dyn Fn(Value) -> Value + Send + Sync>;

pub struct RawTool {
    definition: ToolDefinition,
    run: RawRun,
}

impl RawTool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        run: RawRun,
    ) -> RawTool {
        RawTool {
            definition: ToolDefinition {
                name: name.into(),
                description: description.into(),
                input_schema,
            },
            run,
        }
    }
}

#[async_trait::async_trait]
impl Tool<()> for RawTool {
    fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    async fn execute(&self, _ctx: (), call: &ToolCall) -> anyhow::Result<ToolOutput> {
        let run = self.run.clone();
        let arguments = call.arguments.clone();
        let content = tokio::task::spawn_blocking(move || run(arguments))
            .await
            .map_err(|error| anyhow::anyhow!("the tool stopped: {error}"))?;
        Ok(ToolOutput { content })
    }
}
