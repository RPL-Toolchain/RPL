//! Configuration for match session solving.

/// Maximum number of session results produced per pattern item (0 = unlimited).
pub const DEFAULT_MAX_SESSION_RESULTS: usize = 256;

#[derive(Debug, Clone, Copy)]
pub struct SessionConfig {
    pub max_results: usize,
}

impl SessionConfig {
    /// True when `n` results already meet the cap (`max_results == 0` means unlimited).
    pub fn is_at_cap(self, n: usize) -> bool {
        self.max_results > 0 && n >= self.max_results
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_results: DEFAULT_MAX_SESSION_RESULTS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_results_one_is_truncated() {
        let cfg = SessionConfig { max_results: 1 };
        assert!(!cfg.is_at_cap(0));
        assert!(cfg.is_at_cap(1));
        assert!(cfg.is_at_cap(2));
        let unlimited = SessionConfig { max_results: 0 };
        assert!(!unlimited.is_at_cap(1000));
    }
}
