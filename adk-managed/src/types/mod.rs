//! Wire types for the managed agent runtime.
//!
//! All types in this module conform to CANON §3 wire shapes and are
//! `#[non_exhaustive]` where polymorphic to allow additive evolution.

pub mod agent_def;
pub mod content;
pub mod error;
pub mod events;
pub mod model_ref;
pub mod session;
pub mod tools;

pub use agent_def::*;
pub use content::*;
pub use error::*;
pub use events::{ConfirmationResult, SessionEvent, StopReason, UserEvent};
pub use model_ref::*;
pub use session::*;
pub use tools::*;
