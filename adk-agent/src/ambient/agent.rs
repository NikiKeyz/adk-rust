use std::sync::Arc;

use adk_core::{AdkError, Agent, EventStream, Result};
use futures::StreamExt;
use tokio::sync::{Notify, RwLock};
use tokio::task::JoinHandle;

use super::event_source::EventSource;

/// Callback invoked when the ambient agent's event source fires.
///
/// Receives the trigger event and the agent reference. The callback is responsible
/// for creating an appropriate `InvocationContext` (e.g. via a Runner) and invoking
/// the agent. Return the resulting event stream for the ambient agent to consume.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use adk_agent::ambient::{AmbientAgent, TriggerHandler};
///
/// let handler: TriggerHandler = Arc::new(move |event, agent| {
///     let runner = runner.clone();
///     Box::pin(async move {
///         // Use the event payload as user content and run through a Runner
///         let content = Content::new("user").with_text(&event.payload.to_string());
///         runner.run("user".into(), "session".into(), content).await
///     })
/// });
/// ```
pub type TriggerHandler = Arc<
    dyn Fn(
            super::event_source::TriggerEvent,
            Arc<dyn Agent>,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<EventStream>> + Send>>
        + Send
        + Sync,
>;

/// Lifecycle status of an [`AmbientAgent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientAgentStatus {
    /// The agent is actively processing events.
    Running,
    /// The agent is paused — subscription is alive but events are buffered, not processed.
    Paused,
    /// The agent is stopped — no background task is running.
    Stopped,
}

/// A background agent triggered by an event source.
///
/// Wraps an [`Agent`] and an [`EventSource`], providing lifecycle control
/// (start, stop, pause, resume) over the background event processing loop.
///
/// # Lifecycle
///
/// ```text
/// Stopped → start() → Running → pause() → Paused → resume() → Running
///                        │                     │
///                        └── stop() → Stopped ←┘
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use adk_agent::ambient::{AmbientAgent, CronTrigger};
///
/// let trigger = CronTrigger::new("0 * * * * *")?;
/// let mut ambient = AmbientAgent::new(agent, Arc::new(trigger));
/// ambient.start().await?;
/// // ... later
/// ambient.stop().await?;
/// ```
pub struct AmbientAgent {
    agent: Arc<dyn Agent>,
    source: Arc<dyn EventSource>,
    trigger_handler: Option<TriggerHandler>,
    status: Arc<RwLock<AmbientAgentStatus>>,
    resume_notify: Arc<Notify>,
    handle: Option<JoinHandle<()>>,
}

impl AmbientAgent {
    /// Create a new ambient agent wrapping the given agent and event source.
    ///
    /// The agent starts in [`AmbientAgentStatus::Stopped`] state.
    pub fn new(agent: Arc<dyn Agent>, source: Arc<dyn EventSource>) -> Self {
        Self {
            agent,
            source,
            trigger_handler: None,
            status: Arc::new(RwLock::new(AmbientAgentStatus::Stopped)),
            resume_notify: Arc::new(Notify::new()),
            handle: None,
        }
    }

    /// Set a trigger handler that will be called when the event source fires.
    ///
    /// The handler receives the trigger event and agent, and should invoke the
    /// agent via a Runner or other mechanism. Without a handler, the ambient
    /// agent only logs trigger events.
    pub fn with_trigger_handler(mut self, handler: TriggerHandler) -> Self {
        self.trigger_handler = Some(handler);
        self
    }

    /// Start listening for events and invoking the agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is already running or paused.
    pub async fn start(&mut self) -> Result<()> {
        let current = *self.status.read().await;
        if current != AmbientAgentStatus::Stopped {
            return Err(AdkError::agent("agent already running"));
        }

        // Subscribe to the event source
        let stream = self.source.subscribe().await?;

        let status = Arc::clone(&self.status);
        let resume_notify = Arc::clone(&self.resume_notify);
        let agent = Arc::clone(&self.agent);
        let trigger_handler = self.trigger_handler.clone();

        *self.status.write().await = AmbientAgentStatus::Running;

        let handle = tokio::spawn(async move {
            let mut stream = stream;

            while let Some(event) = stream.next().await {
                // Check if paused — wait until resumed
                loop {
                    let current_status = *status.read().await;
                    match current_status {
                        AmbientAgentStatus::Running => break,
                        AmbientAgentStatus::Paused => {
                            // Wait for resume signal
                            resume_notify.notified().await;
                        }
                        AmbientAgentStatus::Stopped => return,
                    }
                }

                // Process the event — invoke the agent via the trigger handler
                tracing::info!(
                    agent = agent.name(),
                    source = %event.source,
                    "ambient agent triggered"
                );
                tracing::debug!(payload = %event.payload, "trigger event payload");

                if let Some(ref handler) = trigger_handler {
                    match handler(event, agent.clone()).await {
                        Ok(mut event_stream) => {
                            // Consume the event stream, logging results
                            while let Some(result) = event_stream.next().await {
                                match result {
                                    Ok(ev) => {
                                        tracing::debug!(
                                            author = %ev.author,
                                            "ambient agent produced event"
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            error = %e,
                                            "ambient agent invocation error"
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                error = %e,
                                "ambient agent trigger handler failed"
                            );
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Stop the agent and cancel in-progress work.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is already stopped.
    pub async fn stop(&mut self) -> Result<()> {
        let current = *self.status.read().await;
        if current == AmbientAgentStatus::Stopped {
            return Err(AdkError::agent("agent already stopped"));
        }

        *self.status.write().await = AmbientAgentStatus::Stopped;

        // Wake the task if paused so it can observe the Stopped state
        self.resume_notify.notify_one();

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }

        Ok(())
    }

    /// Pause event processing. The subscription remains alive but events are buffered.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not currently running.
    pub async fn pause(&mut self) -> Result<()> {
        let current = *self.status.read().await;
        if current != AmbientAgentStatus::Running {
            return Err(AdkError::agent("can only pause a running agent"));
        }

        *self.status.write().await = AmbientAgentStatus::Paused;
        Ok(())
    }

    /// Resume event processing after a pause.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not currently paused.
    pub async fn resume(&mut self) -> Result<()> {
        let current = *self.status.read().await;
        if current != AmbientAgentStatus::Paused {
            return Err(AdkError::agent("can only resume a paused agent"));
        }

        *self.status.write().await = AmbientAgentStatus::Running;
        self.resume_notify.notify_one();
        Ok(())
    }

    /// Read the current lifecycle status.
    pub async fn status(&self) -> AmbientAgentStatus {
        *self.status.read().await
    }
}

impl Drop for AmbientAgent {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl std::fmt::Debug for AmbientAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AmbientAgent")
            .field("agent", &self.agent.name())
            .field("source", &self.source.name())
            .finish()
    }
}
