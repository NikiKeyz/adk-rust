//! `bash` — run a shell command inside the workspace.
//!
//! Phase 1 executes host-local (`sh -c`) with the working directory pinned to the
//! workspace root and a timeout. It is **not** strongly isolated; production
//! deployments should run it behind a containerized `CodeExecutor` (see the
//! coding-agent design, §9). Mutating use requires [`Workspace::bash_allowed`].

use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use adk_core::{Result, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::io::AsyncReadExt;

use crate::error::DevToolError;
use crate::tools::read::require_str;
use crate::workspace::Workspace;

/// Runs a shell command in the workspace root with a timeout.
pub struct BashTool {
    workspace: Workspace,
}

impl BashTool {
    /// Create a `bash` tool bound to `workspace`.
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Run a shell command in the workspace root and return stdout, stderr, and the \
         exit code. Use for builds, tests, and other commands. Has a timeout."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "The shell command to run." },
                "timeout_secs": { "type": "integer", "description": "Optional timeout in seconds." }
            },
            "required": ["command"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        if !self.workspace.bash_allowed() {
            return Err(DevToolError::BashDisabled.into());
        }
        let command = require_str(&args, "command")?;
        let timeout = args
            .get("timeout_secs")
            .and_then(Value::as_u64)
            .map(Duration::from_secs)
            .unwrap_or_else(|| self.workspace.bash_timeout_value());

        let mut child = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .current_dir(self.workspace.root())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(DevToolError::from)?;

        let mut stdout_pipe = child.stdout.take();
        let mut stderr_pipe = child.stderr.take();
        let cap = self.workspace.max_output();

        let result = tokio::time::timeout(timeout, async {
            let status = child.wait().await?;
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut p) = stdout_pipe.take() {
                let _ = p.read_to_string(&mut stdout).await;
            }
            if let Some(mut p) = stderr_pipe.take() {
                let _ = p.read_to_string(&mut stderr).await;
            }
            Ok::<_, std::io::Error>((status, stdout, stderr))
        })
        .await;

        match result {
            Ok(Ok((status, stdout, stderr))) => {
                let (stdout, out_trunc) = truncate(stdout, cap);
                let (stderr, err_trunc) = truncate(stderr, cap);
                Ok(json!({
                    "command": command,
                    "exit_code": status.code(),
                    "stdout": stdout,
                    "stderr": stderr,
                    "truncated": out_trunc || err_trunc,
                }))
            }
            Ok(Err(e)) => Err(DevToolError::from(e).into()),
            Err(_) => {
                let _ = child.start_kill();
                Err(DevToolError::Timeout(timeout).into())
            }
        }
    }
}

fn truncate(mut s: String, cap: usize) -> (String, bool) {
    if s.len() <= cap {
        return (s, false);
    }
    s.truncate(cap);
    s.push_str("\n…[truncated]");
    (s, true)
}
