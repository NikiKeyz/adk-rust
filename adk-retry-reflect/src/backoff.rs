//! Backoff duration computation.

use std::time::Duration;

use crate::config::BackoffStrategy;

/// Compute the backoff duration for a given strategy, attempt number, and ceiling.
///
/// # Arguments
///
/// * `strategy` — The backoff strategy to use
/// * `attempt` — The current attempt number (1-indexed)
/// * `ceiling` — The maximum allowed backoff duration
///
/// # Returns
///
/// The computed backoff duration, capped at `ceiling`.
///
/// # Formula
///
/// - `None` → `Duration::ZERO`
/// - `Fixed(d)` → `d` (capped at ceiling)
/// - `Exponential { base_delay }` → `min(base_delay * 2^(attempt-1), ceiling)`
///
/// Uses saturating arithmetic to prevent overflow.
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use adk_retry_reflect::backoff::compute_backoff;
/// use adk_retry_reflect::config::BackoffStrategy;
///
/// let ceiling = Duration::from_secs(30);
///
/// assert_eq!(compute_backoff(&BackoffStrategy::None, 1, ceiling), Duration::ZERO);
/// assert_eq!(
///     compute_backoff(&BackoffStrategy::Fixed(Duration::from_secs(2)), 5, ceiling),
///     Duration::from_secs(2)
/// );
/// assert_eq!(
///     compute_backoff(
///         &BackoffStrategy::Exponential { base_delay: Duration::from_secs(1) },
///         3,
///         ceiling
///     ),
///     Duration::from_secs(4) // 1 * 2^2 = 4
/// );
/// ```
pub fn compute_backoff(strategy: &BackoffStrategy, attempt: u32, ceiling: Duration) -> Duration {
    let raw = match strategy {
        BackoffStrategy::None => Duration::ZERO,
        BackoffStrategy::Fixed(d) => *d,
        BackoffStrategy::Exponential { base_delay } => {
            let exponent = attempt.saturating_sub(1);
            let multiplier = 2u32.saturating_pow(exponent);
            base_delay.saturating_mul(multiplier)
        }
    };
    raw.min(ceiling)
}
