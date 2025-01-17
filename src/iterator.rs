use super::Backoff;
use fastrand::Rng;
use std::{iter, time};

/// An exponential backoff iterator.
#[derive(Debug, Clone)]
pub struct Iter<'b> {
    inner: &'b Backoff,
    rng: Rng,
    retry_count: u32,
}

impl<'b> Iter<'b> {
    pub(crate) fn new(inner: &'b Backoff) -> Self {
        Self::with_count(inner, 0)
    }

    pub(crate) fn with_count(inner: &'b Backoff, retry_count: u32) -> Self {
        Self {
            inner,
            retry_count,
            rng: Rng::new(),
        }
    }
}

impl<'b> iter::Iterator for Iter<'b> {
    type Item = time::Duration;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check whether we've exceeded the number of retries.
        // We use `saturating_add` to prevent overflowing on `int::MAX + 1`.
        if self.retry_count == self.inner.retries.saturating_add(1) {
            return None;
        }

        // Create exponential duration.
        let exponent = self.inner.factor.saturating_pow(self.retry_count);
        let duration = self.inner.min.saturating_mul(exponent);

        self.retry_count += 1;

        // Apply jitter. Uses multiples of 100 to prevent relying on floats.
        let jitter_factor = (self.inner.jitter * 100f32) as u32;
        let random = self.rng.u32(0..jitter_factor * 2);
        let mut duration = duration.saturating_mul(100);
        if random < jitter_factor {
            let jitter = duration.saturating_mul(random) / 100;
            duration = duration.saturating_sub(jitter);
        } else {
            let jitter = duration.saturating_mul(random / 2) / 100;
            duration = duration.saturating_add(jitter);
        };
        duration /= 100;

        // Make sure it doesn't exceed upper / lower bounds.
        if let Some(max) = self.inner.max {
            duration = duration.min(max);
        }

        duration = duration.max(self.inner.min);

        Some(duration)
    }
}
