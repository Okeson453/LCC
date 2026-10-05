//! Exponential backoff with full jitter.
//!
//! Source §39.2: standard tier base=2s, max retries=5, total spread 2–60s;
//! heavy (publish) tier base=5s, max retries=5, total spread 5–80s.
//! All with ±20% full jitter to avoid thundering-herd.

use rand::Rng;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffTier {
    Standard,
    Heavy,
}

impl BackoffTier {
    pub fn base_ms(self) -> u64 {
        match self {
            BackoffTier::Standard => 2_000,
            BackoffTier::Heavy => 5_000,
        }
    }

    pub fn max_retries(self) -> u32 {
        5
    }
}

/// Compute the backoff delay for the n-th retry (0-indexed).
///
/// Returns a Duration with ±20% jitter applied.
pub fn delay_for(retry_index: u32, tier: BackoffTier) -> Duration {
    let base_ms = tier.base_ms();
    // Exponential: base * 2^retry_index, capped at 60s for standard / 80s for heavy.
    let cap_ms = match tier {
        BackoffTier::Standard => 60_000u64,
        BackoffTier::Heavy => 80_000u64,
    };
    let exp_ms = base_ms
        .saturating_mul(2u64.saturating_pow(retry_index))
        .min(cap_ms);

    // Apply ±20% jitter.
    let mut rng = rand::thread_rng();
    let jitter_factor: f64 = rng.gen_range(0.8..=1.2);
    let with_jitter_ms = ((exp_ms as f64) * jitter_factor) as u64;

    Duration::from_millis(with_jitter_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_tier_grows_exponentially() {
        let d0 = delay_for(0, BackoffTier::Standard);
        let d1 = delay_for(1, BackoffTier::Standard);
        let d2 = delay_for(2, BackoffTier::Standard);
        // d0 ≈ 2000ms, d1 ≈ 4000ms, d2 ≈ 8000ms (±20% jitter).
        assert!(d0.as_millis() >= 1600 && d0.as_millis() <= 2400);
        assert!(d1.as_millis() >= 3200 && d1.as_millis() <= 4800);
        assert!(d2.as_millis() >= 6400 && d2.as_millis() <= 9600);
    }

    #[test]
    fn standard_capped_at_60s() {
        let d = delay_for(10, BackoffTier::Standard); // way past cap
        assert!(d.as_millis() <= 72_000); // 60s + 20%
    }

    #[test]
    fn heavy_tier_higher_base() {
        let d = delay_for(0, BackoffTier::Heavy);
        assert!(d.as_millis() >= 4_000 && d.as_millis() <= 6_000);
    }

    #[test]
    fn heavy_capped_at_80s() {
        let d = delay_for(15, BackoffTier::Heavy);
        assert!(d.as_millis() <= 96_000); // 80s + 20%
    }
}
