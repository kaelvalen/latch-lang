use std::sync::atomic::{AtomicU64, Ordering};

/// Zero-cost runtime profiling metrics hook for Latch Virtual Machine.
pub struct VmProfiler {
    pub instruction_count: AtomicU64,
    pub call_count: AtomicU64,
    pub alloc_count: AtomicU64,
    pub opcode_histogram: [AtomicU64; 256],
}

impl VmProfiler {
    pub fn new() -> Self {
        // Construct atomic array
        const INIT: AtomicU64 = AtomicU64::new(0);
        VmProfiler {
            instruction_count: AtomicU64::new(0),
            call_count: AtomicU64::new(0),
            alloc_count: AtomicU64::new(0),
            opcode_histogram: [INIT; 256],
        }
    }

    #[inline(always)]
    pub fn record_instruction(&self, opcode: u8) {
        self.instruction_count.fetch_add(1, Ordering::Relaxed);
        self.opcode_histogram[opcode as usize].fetch_add(1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn record_call(&self) {
        self.call_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn record_alloc(&self) {
        self.alloc_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn print_summary(&self) {
        let total_ops = self.instruction_count.load(Ordering::Relaxed);
        let total_calls = self.call_count.load(Ordering::Relaxed);
        let total_allocs = self.alloc_count.load(Ordering::Relaxed);

        println!("── Latch VM Profiler Summary ──────────────────────────");
        println!("Total Instructions Executed : {total_ops}");
        println!("Total Function Invocations  : {total_calls}");
        println!("Total Heap Allocations      : {total_allocs}");
        println!("───────────────────────────────────────────────────────");
    }
}
