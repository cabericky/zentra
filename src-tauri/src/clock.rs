use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

/// High-resolution monotonic clock wrapper using Windows QueryPerformanceCounter (QPC).
/// Provides consistent sub-microsecond timestamps across screen capture and mouse event logging.
#[derive(Debug, Clone, Copy)]
pub struct QpcClock {
    pub frequency: i64,
}

impl QpcClock {
    /// Initializes a new QpcClock by querying the system QPC frequency.
    pub fn new() -> Self {
        let mut freq = 0i64;
        unsafe {
            let _ = QueryPerformanceFrequency(&mut freq);
        }
        Self { frequency: freq }
    }

    /// Queries the current QPC tick count.
    #[inline]
    pub fn now(&self) -> i64 {
        let mut count = 0i64;
        unsafe {
            let _ = QueryPerformanceCounter(&mut count);
        }
        count
    }

    /// Computes the elapsed seconds between two QPC counts.
    #[inline]
    pub fn delta_seconds(&self, start: i64, end: i64) -> f64 {
        if self.frequency == 0 {
            return 0.0;
        }
        (end - start) as f64 / self.frequency as f64
    }

    /// Converts a QPC tick difference into Media Foundation 100-nanosecond (hns) time units.
    /// (1 second = 10,000,000 hns)
    #[inline]
    pub fn to_hns(&self, qpc_delta: i64) -> i64 {
        if self.frequency == 0 {
            return 0;
        }
        // Use 128-bit arithmetic to avoid overflow during (qpc_delta * 10_000_000)
        let delta_128 = qpc_delta as i128;
        let hns = (delta_128 * 10_000_000i128) / (self.frequency as i128);
        hns as i64
    }

    /// Converts seconds to 100-nanosecond units.
    #[inline]
    pub fn seconds_to_hns(seconds: f64) -> i64 {
        (seconds * 10_000_000.0) as i64
    }
}