use std::sync::atomic;

type zx_koid_t = u32;

// Generates unique 64bit ids for kernel objects.
struct KernelObjectId {
    koid_generator: atomic::AtomicU32,
}

impl KernelObjectId {
    pub fn gen(&self) -> zx_koid_t {
        return self.koid_generator.fetch_add(1, atomic::Ordering::Relaxed);
    }
}
