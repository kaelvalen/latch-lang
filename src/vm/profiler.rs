use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::env::ObjKind;

/// Zero-cost runtime profiling metrics hook for Latch Virtual Machine.
pub struct VmProfiler {
    pub instruction_count: AtomicU64,
    pub call_count: AtomicU64,
    pub alloc_count: AtomicU64,
    pub opcode_histogram: [AtomicU64; 256],
    /// Per-object-kind allocation profile: (count, total_bytes).
    allocations: Mutex<HashMap<ObjKind, (usize, usize)>>,
}

impl Default for VmProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl VmProfiler {
    pub fn new() -> Self {
        VmProfiler {
            instruction_count: AtomicU64::new(0),
            call_count: AtomicU64::new(0),
            alloc_count: AtomicU64::new(0),
            opcode_histogram: std::array::from_fn(|_| AtomicU64::new(0)),
            allocations: Mutex::new(HashMap::new()),
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

    /// Record a heap allocation of `size` bytes for the given object kind.
    pub fn record_allocation(&self, kind: ObjKind, size: usize) {
        if let Ok(mut map) = self.allocations.lock() {
            let entry = map.entry(kind).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += size;
        }
    }

    /// Return a per-object-kind allocation summary: (kind, count, total_bytes).
    pub fn allocation_summary(&self) -> Vec<(ObjKind, usize, usize)> {
        let map = self.allocations.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        map.iter().map(|(kind, (count, bytes))| (*kind, *count, *bytes)).collect()
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
