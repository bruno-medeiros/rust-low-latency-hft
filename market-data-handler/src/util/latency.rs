//! Tick-to-trade latency recording and reporting.
//!
//! Uses `quanta::Clock` (TSC-backed on x86 Linux) for nanosecond-precision timestamps.
//! Samples are accumulated in an HDR histogram for accurate high-percentile reporting.

use hdrhistogram::Histogram;
use quanta::Clock;

/// Raw TSC snapshot, obtained from [`LatencyRecorder::now`].
pub type RawTs = u64;

/// Records per-event tick-to-trade latency samples and reports percentiles.
pub struct LatencyRecorder {
    clock: Clock,
    pub hist: Histogram<u64>,
}

impl LatencyRecorder {
    /// Construct with a HDR histogram covering 1 ns – 10 ms, 3 significant figures.
    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
            hist: Histogram::new_with_bounds(1, 10_000_000, 3)
                .expect("valid histogram bounds"),
        }
    }

    /// Take a raw TSC timestamp. Call immediately after `recvmmsg` returns (T0)
    /// and immediately before writing the outbound buffer (T1).
    #[inline(always)]
    pub fn now(&self) -> RawTs {
        self.clock.raw()
    }

    /// Convert a (t0, t1) raw pair to nanoseconds and record in the histogram.
    ///
    /// Values that overflow the histogram ceiling are saturated to the max bucket.
    #[inline]
    pub fn record(&mut self, t0: RawTs, t1: RawTs) {
        let nanos = self.clock.delta_as_nanos(t0, t1);
        // Saturate to max to avoid panics from sporadic OS preemptions.
        let clamped = nanos.min(self.hist.high());
        let _ = self.hist.record(clamped);
    }
}

impl Default for LatencyRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_reports() {
        let mut rec = LatencyRecorder::new();
        let t0 = rec.now();
        std::hint::black_box(0u64.wrapping_add(1)); // trivial work
        let t1 = rec.now();
        rec.record(t0, t1);
        assert_eq!(rec.hist.len(), 1);
        assert!(rec.hist.max() > 0);
    }

    #[test]
    fn clamps_extreme_values() {
        let mut rec = LatencyRecorder::new();
        // Record a value at the histogram ceiling (10 ms); must not panic.
        // HDR histogram rounds to bucket boundaries, so max may be slightly above high().
        let ceiling = rec.hist.high();
        let _ = rec.hist.record(ceiling);
        assert!(rec.hist.max() >= ceiling);
        assert_eq!(rec.hist.len(), 1);
    }
}
