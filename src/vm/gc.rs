use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::env::{HeapObject, GcTrace, ObjClass, ObjClosure, ObjFunction, ObjFunctionBuilder, ObjKind, ObjRef, Value};

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

    /// Generic allocation hook. Tracks size and wraps the object in `ObjRef<T>`.
    pub fn allocate<T>(&self, obj: T) -> ObjRef<T> {
        self.track_alloc(std::mem::size_of::<T>());
        ObjRef::new(obj)
    }

    /// Allocate a compiled function from a builder.
    pub fn allocate_function(&self, builder: ObjFunctionBuilder) -> ObjRef<ObjFunction> {
        let func = builder.build();
        self.track_alloc(std::mem::size_of::<ObjFunction>() + func.chunk.code().len());
        ObjRef::new(func)
    }

    /// Allocate a closure bound to a compiled function.
    pub fn allocate_closure(
        &self,
        function: ObjRef<ObjFunction>,
        upvalues: Vec<Arc<Mutex<Value>>>,
    ) -> ObjRef<ObjClosure> {
        self.track_alloc(std::mem::size_of::<ObjClosure>());
        ObjRef::new(ObjClosure::new(function, upvalues))
    }

    /// Allocate a class object.
    pub fn allocate_class(&self, name: impl Into<String>) -> ObjRef<ObjClass> {
        let class = ObjClass::new(name);
        self.track_alloc(std::mem::size_of::<ObjClass>());
        ObjRef::new(class)
    }

    /// Mark a root object for the future tracing collector.
    pub fn mark_root(&self, _obj: &dyn HeapObject) {
        // Stub: future GC will gray the object and enqueue it.
    }

    /// Trace an object that implements the GC trace contract.
    pub fn trace_object(&self, _obj: &dyn GcTrace) {
        // Stub: future GC will call trace() on the object.
    }

    /// Write barrier hook for generational / incremental collectors.
    pub fn write_barrier(&self, _parent: &dyn HeapObject, _child: &dyn HeapObject) {
        // Stub: generational / incremental GC barrier hook.
    }

    /// Return a debug heap snapshot (stub).
    pub fn heap_snapshot(&self) -> HeapSnapshot {
        HeapSnapshot { objects: Vec::new() }
    }

    /// Sweep unreachable objects.
    pub fn sweep(&self) {
        // Stub: future GC will reclaim white objects.
    }

    /// Trigger a collection cycle if the allocation threshold is exceeded.
    pub fn collect_if_needed(&self) {
        if self.should_collect() {
            self.sweep();
            self.update_threshold_after_sweep(0);
        }
    }
}

/// Debug heap snapshot: a list of (kind, size) pairs.
#[derive(Debug, Clone)]
pub struct HeapSnapshot {
    pub objects: Vec<(ObjKind, usize)>,
}
