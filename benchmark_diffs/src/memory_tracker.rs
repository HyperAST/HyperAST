use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MemoryTracker;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static NET_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK_NET_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static MARK_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static MARK_ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for MemoryTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        // Update currently allocated memory
        let new_allocated = ALLOCATED.fetch_add(size, Ordering::SeqCst) + size;
        
        // Update peak memory (highest watermark)
        let mut peak = PEAK_ALLOCATED.load(Ordering::SeqCst);
        while new_allocated > peak {
            match PEAK_ALLOCATED.compare_exchange(peak, new_allocated, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(current) => peak = current,
            }
        }
        
        // Update net allocated since last mark
        let new_net_allocated = NET_ALLOCATED.fetch_add(size, Ordering::SeqCst) + size;
        
        // Update peak net memory 
        let mut peak_net = PEAK_NET_ALLOCATED.load(Ordering::SeqCst);
        while new_net_allocated > peak_net {
            match PEAK_NET_ALLOCATED.compare_exchange(peak_net, new_net_allocated, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(current) => peak_net = current,
            }
        }
        
        // Count allocation operations
        ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        ALLOCATED.fetch_sub(size, Ordering::SeqCst);
        NET_ALLOCATED.fetch_sub(size, Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

/// Get current allocated memory in bytes
pub fn get_allocated() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}

/// Get peak allocated memory in bytes since last reset
pub fn get_peak_allocated() -> usize {
    PEAK_ALLOCATED.load(Ordering::SeqCst)
}

/// Get count of allocation operations performed
pub fn get_allocation_count() -> usize {
    ALLOCATION_COUNT.load(Ordering::SeqCst)
}

/// Get net allocated memory since last mark
pub fn get_net_allocated() -> usize {
    NET_ALLOCATED.load(Ordering::SeqCst)
}

/// Get peak net allocated memory since last mark
pub fn get_peak_net_allocated() -> usize {
    PEAK_NET_ALLOCATED.load(Ordering::SeqCst)
}

/// Get allocations since last mark
pub fn get_allocations_since_mark() -> usize {
    ALLOCATION_COUNT.load(Ordering::SeqCst) - MARK_ALLOCATION_COUNT.load(Ordering::SeqCst)
}

/// Reset the peak memory counter to the current allocated memory
pub fn reset_peak() {
    PEAK_ALLOCATED.store(ALLOCATED.load(Ordering::SeqCst), Ordering::SeqCst);
}

/// Reset the allocation counter to zero
pub fn reset_allocation_count() {
    ALLOCATION_COUNT.store(0, Ordering::SeqCst);
}

/// Reset all counters
pub fn reset_all() {
    reset_peak();
    reset_allocation_count();
    PEAK_NET_ALLOCATED.store(0, Ordering::SeqCst);
    NET_ALLOCATED.store(0, Ordering::SeqCst);
    MARK_ALLOCATED.store(0, Ordering::SeqCst);
    MARK_ALLOCATION_COUNT.store(0, Ordering::SeqCst);
}

/// Mark the current memory state for differential measurements
pub fn mark() {
    MARK_ALLOCATED.store(ALLOCATED.load(Ordering::SeqCst), Ordering::SeqCst);
    MARK_ALLOCATION_COUNT.store(ALLOCATION_COUNT.load(Ordering::SeqCst), Ordering::SeqCst);
    NET_ALLOCATED.store(0, Ordering::SeqCst);
    PEAK_NET_ALLOCATED.store(0, Ordering::SeqCst);
}

/// Get the net memory change since the last mark
pub fn get_memory_since_mark() -> isize {
    let current = ALLOCATED.load(Ordering::SeqCst);
    let marked = MARK_ALLOCATED.load(Ordering::SeqCst);
    current as isize - marked as isize
}

/// Struct that resets peak on drop, for scoped measurements
pub struct ScopedPeakMeasurement;

impl ScopedPeakMeasurement {
    /// Create a new scoped measurement, resetting the peak
    pub fn new() -> Self {
        reset_peak();
        Self
    }
    
    /// Get the current peak measurement without dropping
    pub fn current_peak(&self) -> usize {
        get_peak_allocated()
    }
}

impl Drop for ScopedPeakMeasurement {
    fn drop(&mut self) {
        // Optional: could log the peak here if desired
    }
}