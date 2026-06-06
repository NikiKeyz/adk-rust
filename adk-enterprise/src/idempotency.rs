//! Idempotency key generation — ensures create operations are replay-safe.
//!
//! An idempotency key is generated ONCE per logical create operation and reused
//! across retries. This prevents duplicate resource creation when network errors
//! cause automatic retries.

/// Generate a new idempotency key (UUID v4).
///
/// This key should be generated once per logical create operation and reused
/// on every retry attempt for that operation. The server uses this key to
/// deduplicate requests — if the same key is seen twice, the second request
/// returns the result of the first rather than creating a duplicate.
///
/// # Example
///
/// ```
/// use adk_enterprise::idempotency::generate_idempotency_key;
///
/// let key = generate_idempotency_key();
/// assert_eq!(key.len(), 36); // UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
/// ```
pub fn generate_idempotency_key() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Header name for idempotency keys sent on create endpoints.
pub const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_generate_idempotency_key_format() {
        let key = generate_idempotency_key();
        // UUID v4 format: 8-4-4-4-12 hex digits with hyphens
        assert_eq!(key.len(), 36);
        let parts: Vec<&str> = key.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn test_generate_idempotency_key_version_4() {
        let key = generate_idempotency_key();
        // Version nibble (13th character) should be '4'
        let chars: Vec<char> = key.chars().collect();
        assert_eq!(chars[14], '4');
    }

    #[test]
    fn test_generate_idempotency_key_uniqueness() {
        let keys: HashSet<String> = (0..100).map(|_| generate_idempotency_key()).collect();
        assert_eq!(keys.len(), 100, "All generated keys should be unique");
    }

    #[test]
    fn test_idempotency_key_header_name() {
        assert_eq!(IDEMPOTENCY_KEY_HEADER, "Idempotency-Key");
    }
}
