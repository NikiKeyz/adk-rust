//! Session lifecycle status types.

use serde::{Deserialize, Serialize};

/// Session lifecycle state. Entry state is `Queued`.
///
/// State machine:
/// ```text
/// Queued → Running → Idle (per turn) → Running (next turn)
///                  → Rescheduling → Running (on retry success)
///                  → Rescheduling → Failed (on retry exhaust)
///                  → Paused → Running (on resume)
///                  → Completed / Failed / Archived
/// ```
///
/// # Example
///
/// ```
/// use adk_managed::types::SessionStatus;
///
/// let status = SessionStatus::Queued;
/// let json = serde_json::to_string(&status).unwrap();
/// assert_eq!(json, r#""queued""#);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SessionStatus {
    /// Session created, waiting for first event to process.
    Queued,
    /// Actively processing a turn.
    Running,
    /// Turn complete, awaiting next input.
    Idle,
    /// Transient error encountered; auto-retrying with backoff.
    /// Transitions: Running → Rescheduling → Running (success) | Failed (exhaust).
    Rescheduling,
    /// Explicitly paused by caller; checkpoint saved.
    Paused,
    /// Session completed successfully (terminal).
    Completed,
    /// Session failed (terminal).
    Failed,
    /// Session archived (terminal, data retained for read).
    Archived,
}

impl SessionStatus {
    /// Returns `true` if this is a terminal state (no further transitions allowed).
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Archived)
    }
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Idle => "idle",
            Self::Rescheduling => "rescheduling",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Archived => "archived",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_serializes_lowercase() {
        let cases = [
            (SessionStatus::Queued, "\"queued\""),
            (SessionStatus::Running, "\"running\""),
            (SessionStatus::Idle, "\"idle\""),
            (SessionStatus::Rescheduling, "\"rescheduling\""),
            (SessionStatus::Paused, "\"paused\""),
            (SessionStatus::Completed, "\"completed\""),
            (SessionStatus::Failed, "\"failed\""),
            (SessionStatus::Archived, "\"archived\""),
        ];

        for (status, expected) in cases {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, expected, "serialization failed for {status:?}");
        }
    }

    #[test]
    fn test_session_status_deserializes_lowercase() {
        let cases = [
            ("\"queued\"", SessionStatus::Queued),
            ("\"running\"", SessionStatus::Running),
            ("\"idle\"", SessionStatus::Idle),
            ("\"rescheduling\"", SessionStatus::Rescheduling),
            ("\"paused\"", SessionStatus::Paused),
            ("\"completed\"", SessionStatus::Completed),
            ("\"failed\"", SessionStatus::Failed),
            ("\"archived\"", SessionStatus::Archived),
        ];

        for (json, expected) in cases {
            let status: SessionStatus = serde_json::from_str(json).unwrap();
            assert_eq!(status, expected, "deserialization failed for {json}");
        }
    }

    #[test]
    fn test_session_status_round_trip() {
        let all_statuses = [
            SessionStatus::Queued,
            SessionStatus::Running,
            SessionStatus::Idle,
            SessionStatus::Rescheduling,
            SessionStatus::Paused,
            SessionStatus::Completed,
            SessionStatus::Failed,
            SessionStatus::Archived,
        ];

        for status in all_statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: SessionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized, "round-trip failed for {status:?}");
        }
    }

    #[test]
    fn test_session_status_rejects_unknown() {
        let result: Result<SessionStatus, _> = serde_json::from_str("\"unknown_status\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_status_is_terminal() {
        assert!(!SessionStatus::Queued.is_terminal());
        assert!(!SessionStatus::Running.is_terminal());
        assert!(!SessionStatus::Idle.is_terminal());
        assert!(!SessionStatus::Rescheduling.is_terminal());
        assert!(!SessionStatus::Paused.is_terminal());
        assert!(SessionStatus::Completed.is_terminal());
        assert!(SessionStatus::Failed.is_terminal());
        assert!(SessionStatus::Archived.is_terminal());
    }

    #[test]
    fn test_session_status_display() {
        assert_eq!(SessionStatus::Queued.to_string(), "queued");
        assert_eq!(SessionStatus::Running.to_string(), "running");
        assert_eq!(SessionStatus::Idle.to_string(), "idle");
        assert_eq!(SessionStatus::Rescheduling.to_string(), "rescheduling");
        assert_eq!(SessionStatus::Paused.to_string(), "paused");
        assert_eq!(SessionStatus::Completed.to_string(), "completed");
        assert_eq!(SessionStatus::Failed.to_string(), "failed");
        assert_eq!(SessionStatus::Archived.to_string(), "archived");
    }
}
