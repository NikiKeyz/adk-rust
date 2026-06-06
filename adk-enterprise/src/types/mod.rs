//! Wire types for the Enterprise Managed Agent Service.
//!
//! These types are self-contained copies of the CANON wire format.
//! They serialize/deserialize with `#[serde(rename_all = "camelCase")]`
//! and use `#[serde(default)]` on optional response fields for forward compatibility.

pub mod agent;
pub mod environment;
pub mod events;
pub mod memory;
pub mod model_ref;
pub mod pagination;
pub mod session;
pub mod tools;
pub mod vault;

pub use agent::*;
pub use environment::*;
pub use events::*;
pub use memory::*;
pub use model_ref::*;
pub use pagination::*;
pub use session::*;
pub use tools::*;
pub use vault::*;
