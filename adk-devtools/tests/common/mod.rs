//! Minimal `ToolContext` test double (the dev tools ignore the context).

use std::sync::Arc;

use adk_core::{
    CallbackContext, Content, EventActions, MemoryEntry, ReadonlyContext, Result, ToolContext,
};
use async_trait::async_trait;

pub struct TestCtx;

#[async_trait]
impl ReadonlyContext for TestCtx {
    fn invocation_id(&self) -> &str {
        "inv-1"
    }
    fn agent_name(&self) -> &str {
        "test-agent"
    }
    fn user_id(&self) -> &str {
        "tester"
    }
    fn app_name(&self) -> &str {
        "test-app"
    }
    fn session_id(&self) -> &str {
        "session-1"
    }
    fn branch(&self) -> &str {
        ""
    }
    fn user_content(&self) -> &Content {
        static CONTENT: std::sync::OnceLock<Content> = std::sync::OnceLock::new();
        CONTENT.get_or_init(|| Content::new("user").with_text("hi"))
    }
}

#[async_trait]
impl CallbackContext for TestCtx {
    fn artifacts(&self) -> Option<Arc<dyn adk_core::Artifacts>> {
        None
    }
}

#[async_trait]
impl ToolContext for TestCtx {
    fn function_call_id(&self) -> &str {
        "call-1"
    }
    fn actions(&self) -> EventActions {
        EventActions::default()
    }
    fn set_actions(&self, _actions: EventActions) {}
    async fn search_memory(&self, _query: &str) -> Result<Vec<MemoryEntry>> {
        Ok(vec![])
    }
}
