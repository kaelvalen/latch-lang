use std::sync::atomic::{AtomicUsize, Ordering};

/// Garbage Collector state and memory allocation metrics manager.
pub struct GcState {
    pub bytes_allocated: AtomicUsize,
    pub next_gc_threshold: AtomicUsize,
}

impl GcState {
    pub fn new() -> Self {
        GcState {
            bytes_allocated: AtomicUsize::new(0),
            next_gc_threshold: AtomicUsize::new(1024 * 1024), // 1 MB initial threshold
        }
    }

    pub fn track_alloc(&self, size: usize) {
        self.bytes_allocated.fetch_add(size, Ordering::Relaxed);
    }

    pub fn track_free(&self, size: usize) {
        self.bytes_allocated.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn should_collect(&self) -> bool {
        self.bytes_allocated.load(Ordering::Relaxed) >= self.next_gc_threshold.load(Ordering::Relaxed)
    }

    pub fn update_threshold_after_sweep(&self, live_bytes: usize) {
        let next = (live_bytes * 2).max(1024 * 1024);
        self.next_gc_threshold.store(next, Ordering::Relaxed);
    }
}
