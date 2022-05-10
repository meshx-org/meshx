use fiber_sys as sys;
use std::sync::atomic;

const KOID_GENERATOR: atomic::AtomicU64 = atomic::AtomicU64::new(0);

// Generates unique 64bit ids for kernel objects.
pub fn generate() -> sys::fx_koid_t {
    return KOID_GENERATOR.fetch_add(1, atomic::Ordering::Relaxed);
}
