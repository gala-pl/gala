//! Garbage collector for the Gala runtime.
//!
//! Manages the lifecycle of heap-allocated objects, particularly
//! quantum resources (qubits, circuits) that must be tracked
//! for safe uncomputation and cleanup.

use gala_core::int::Int;
use gala_core::bool::Bool;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

/// A unique identifier for a tracked GC allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GcId(u64);

/// The garbage collector.
///
/// Uses a simple mark-sweep approach with a root set.
/// Tracks heap allocations and periodically reclaims
/// unreachable objects.
pub struct GarbageCollector {
    next_id: Cell<u64>,
    objects: RefCell<Vec<GcObject>>,
    roots: RefCell<HashSet<GcId>>,
    stats: RefCell<GcStats>,
}

struct GcObject {
    id: GcId,
    marked: bool,
    size: usize,
    #[allow(dead_code)]
    kind: &'static str,
}

#[derive(Clone, Debug)]
pub struct GcStats {
    pub total_objects: Int,
    pub total_bytes: Int,
    pub collections_run: Int,
    pub collected_objects: Int,
    pub collected_bytes: Int,
}

impl GcStats {
    fn new() -> Self {
        GcStats {
            total_objects: 0,
            total_bytes: 0,
            collections_run: 0,
            collected_objects: 0,
            collected_bytes: 0,
        }
    }
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl GarbageCollector {
    /// Creates a new garbage collector.
    pub fn new() -> Self {
        GarbageCollector {
            next_id: Cell::new(0),
            objects: RefCell::new(Vec::new()),
            roots: RefCell::new(HashSet::new()),
            stats: RefCell::new(GcStats::new()),
        }
    }

    /// Allocates a new GC-managed object and returns its ID.
    pub fn alloc(&self, size: usize, kind: &'static str) -> GcId {
        let id = GcId(self.next_id.get());
        self.next_id.set(self.next_id.get() + 1);
        self.objects.borrow_mut().push(GcObject {
            id,
            marked: false,
            size,
            kind,
        });
        let mut stats = self.stats.borrow_mut();
        stats.total_objects += 1;
        stats.total_bytes += size as Int;
        id
    }

    /// Adds a root to the GC root set.
    pub fn add_root(&self, id: GcId) {
        self.roots.borrow_mut().insert(id);
    }

    /// Removes a root from the GC root set.
    pub fn remove_root(&self, id: GcId) {
        self.roots.borrow_mut().remove(&id);
    }

    /// Returns `true` if the given ID is a live root.
    pub fn is_root(&self, id: GcId) -> Bool {
        self.roots.borrow().contains(&id)
    }

    /// Runs a full mark-sweep collection.
    pub fn collect(&self) -> GcStats {
        // Mark phase: mark all roots
        let mut objects = self.objects.borrow_mut();
        for obj in objects.iter_mut() {
            obj.marked = false;
        }
        for root_id in self.roots.borrow().iter() {
            if let Some(obj) = objects.iter_mut().find(|o| o.id == *root_id) {
                obj.marked = true;
            }
        }

        // Sweep phase: collect unmarked objects
        let before = objects.len();
        let mut collected_bytes = 0;
        objects.retain(|obj| {
            if obj.marked {
                true
            } else {
                collected_bytes += obj.size;
                false
            }
        });
        let collected_objects = before - objects.len();

        let mut stats = self.stats.borrow_mut();
        stats.collections_run += 1;
        stats.collected_objects += collected_objects as Int;
        stats.collected_bytes += collected_bytes as Int;
        stats.total_objects = objects.len() as Int;
        stats.total_bytes = objects.iter().map(|o| o.size as Int).sum();

        stats.clone()
    }

    /// Returns current GC stats.
    pub fn stats(&self) -> GcStats {
        self.stats.borrow().clone()
    }

    /// Returns the number of live objects.
    pub fn live_count(&self) -> Int {
        self.objects.borrow().len() as Int
    }
}

/// An RAII guard that registers itself as a GC root.
pub struct GcRootGuard {
    gc: Rc<GarbageCollector>,
    id: GcId,
}

impl GcRootGuard {
    pub fn new(gc: Rc<GarbageCollector>, id: GcId) -> Self {
        gc.add_root(id);
        GcRootGuard { gc, id }
    }
}

impl Drop for GcRootGuard {
    fn drop(&mut self) {
        self.gc.remove_root(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_alloc_and_stats() {
        let gc = GarbageCollector::new();
        let _id1 = gc.alloc(64, "circuit");
        let _id2 = gc.alloc(32, "qubit");
        assert_eq!(gc.live_count(), 2);
        assert_eq!(gc.stats().total_bytes, 96);
    }

    #[test]
    fn test_gc_collect_reclaims_unreachable() {
        let gc = GarbageCollector::new();
        let _id = gc.alloc(64, "circuit");
        assert_eq!(gc.live_count(), 1);
        gc.collect();
        assert_eq!(gc.live_count(), 0);
    }

    #[test]
    fn test_gc_root_protects_object() {
        let gc = Rc::new(GarbageCollector::new());
        let id = gc.alloc(64, "circuit");
        let _guard = GcRootGuard::new(gc.clone(), id);
        gc.collect();
        assert_eq!(gc.live_count(), 1);
    }
}
