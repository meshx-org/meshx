// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2016 The Fuchsia Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use std::any::Any;
use std::arch::asm;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;

use crate::object::{BaseDispatcher, Dispatcher, HANDLE_TABLE};

use fiber_sys::{fx_koid_t, fx_rights_t};
use static_assertions::const_assert;

// HandleOwner wraps a Handle in a Box that has single
// ownership of the Handle and deletes it whenever it falls out of scope.
pub(crate) type HandleOwner = Box<Handle>;

/// A minimal wrapper around a Dispatcher which is owned by the kernel.
///
/// Intended usage when creating new a Dispatcher object is:
///   1. Create a KernelHandle on the stack (cannot fail)
///   2. Move the RefPtr<Dispatcher> into the KernelHandle (cannot fail)
///   3. When ready to give the handle to a process, upgrade the KernelHandle
///      to a full HandleOwner via UpgradeToHandleOwner() or
///      user_out_handle::make() (can fail)
///
/// This sequence ensures that the Dispatcher's on_zero_handles() method is
/// called even if errors occur during or before HandleOwner creation, which
/// is necessary to break circular references for some Dispatcher types.
///
/// This class is thread-unsafe and must be externally synchronized if used
/// across multiple threads.
pub(crate) struct KernelHandle<T> {
    pub(super) dispatcher: Rc<T>,
}

impl<T> KernelHandle<T> {
    pub fn dispatcher(&self) -> &Rc<T> {
        &self.dispatcher
    }
}

/// A Handle is how a specific process refers to a specific Dispatcher.
pub(crate) struct Handle {
    // process_id_ is atomic because threads from different processes can
    // access it concurrently, while holding different instances of
    // handle_table_lock_.
    pub(super) process_id: AtomicU64,
    pub(super) dispatcher: Rc<dyn Dispatcher>,
    pub(super) handle_rights: fx_rights_t,
    pub(super) base_value: u32,
}

fn index_to_handle(index: usize) -> *const u32 {
    &0
}

fn handle_value_to_index(value: u32) -> u32 {
    value & HANDLE_INDEX_MASK
}

/// (x == y) ? a : b
#[inline]
const fn conditional_select_spec_eq(x: usize, y: usize, a: usize, b: usize) -> usize {
    if x == y {
        a
    } else {
        b
    }
}

impl Handle {
    // Called only by Make.
    fn new(dispatcher: Rc<dyn Dispatcher>, rights: fx_rights_t, base_value: u32) -> Self {
        Handle {
            process_id: AtomicU64::new(0),
            handle_rights: rights,
            dispatcher,
            base_value: 0,
        }
    }

    // Called only by Dup.
    fn new_from_raw(rhs: *const Handle, rights: fx_rights_t, base_value: u32) -> Self {
        let dispatcher = unsafe { (*rhs).dispatcher().clone() };

        Handle {
            process_id: AtomicU64::new(0),
            handle_rights: rights,
            dispatcher,
            base_value: 0,
        }
    }

    /// Maps an integer obtained by Handle::base_value() back to a Handle.
    pub(super) fn from_u32(value: u32) -> *const Self {
        let index = handle_value_to_index(value);
        let handle_addr = index_to_handle(index as usize);

        let handle_addr = handle_addr as *const Self;

        // if !HANDLE_TABLE.arena.committed(handle_addr as *const ()) {
        //     return std::ptr::null();
        //  }

        // let handle_addr = gHandleTableArena.arena.Confine(handle_addr);

        let handle = unsafe { &*handle_addr };
        let handle_addr = unsafe { handle_addr as usize };

        conditional_select_spec_eq(handle.base_value() as usize, value as usize, handle_addr, 0) as *const Handle
    }

    // Handle should never be created by anything other than Make or Dup.
    pub(crate) fn make_from_dispatcher<T: 'static + Dispatcher>(dispatcher: Rc<T>, rights: fx_rights_t) -> HandleOwner {
        Box::new(Handle::new(dispatcher, rights, 0))
    }

    pub(crate) fn make<T: 'static + Dispatcher>(handle: KernelHandle<T>, rights: fx_rights_t) -> HandleOwner {
        Box::new(Handle::new(handle.dispatcher().clone(), rights, 0))
    }

    pub(crate) fn dup<T: 'static>(source: *const Handle, rights: fx_rights_t) -> HandleOwner {
        Box::new(Handle::new_from_raw(source, rights, 0))
    }

    /// Returns a value that can be decoded by Handle::FromU32() to derive a
    /// pointer to this instance.  ProcessDispatcher will XOR this with its
    /// |handle_rand_| to create the zx_handle_t value that user space sees.
    pub(crate) fn base_value(&self) -> u32 {
        return self.base_value;
    }

    // Get the number of outstanding handles for a given dispatcher.
    pub(crate) fn count(dispatcher: Rc<dyn Dispatcher>) -> u32 {
        dispatcher.base().current_handle_count()
    }

    /// To be called once during bring up.
    pub(crate) fn init() {
        //gHandleTableArena.arena_.Init("handles", kMaxHandleCount);
    }

    /// Returns the Dispatcher to which this instance points.
    pub(crate) fn dispatcher(&self) -> &Rc<dyn Dispatcher> {
        return &self.dispatcher;
    }

    /// Returns the process that owns this instance. Used to guarantee
    /// that one process may not access a handle owned by a different process.
    pub(crate) fn process_id(&self) -> fx_koid_t {
        self.process_id.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Sets the value returned by process_id().
    pub(crate) fn set_process_id(&self, pid: fx_koid_t) {}

    /// Returns the |rights| parameter that was provided when this instance
    /// was created.
    pub(crate) fn rights(&self) -> fx_rights_t {
        self.handle_rights
    }
}

// Compute floor(log2(|val|)), or 0 if |val| is 0
const fn bit_width(mut x: i64) -> i64 {
    let mut i;
    let mut j;
    let mut k;
    let l;
    let m;
    x = x | (x >> 1);
    x = x | (x >> 2);
    x = x | (x >> 4);
    x = x | (x >> 8);
    x = x | (x >> 16);

    // i = 0x55555555
    i = 0x55 | (0x55 << 8);
    i = i | (i << 16);

    // j = 0x33333333
    j = 0x33 | (0x33 << 8);
    j = j | (j << 16);

    // k = 0x0f0f0f0f
    k = 0x0f | (0x0f << 8);
    k = k | (k << 16);

    // l = 0x00ff00ff
    l = 0xff | (0xff << 16);

    // m = 0x0000ffff
    m = 0xff | (0xff << 8);

    x = (x & i) + ((x >> 1) & i);
    x = (x & j) + ((x >> 2) & j);
    x = (x & k) + ((x >> 4) & k);
    x = (x & l) + ((x >> 8) & l);
    x = (x & m) + ((x >> 16) & m);
    x = x + !0;
    return x;
}

const fn log2_floor(val: u32) -> u32 {
    return if val == 0 { 0 } else { (bit_width(val as i64)) as u32 };
}

const fn log2_uint_floor(val: u32) -> u32 {
    log2_floor(val)
}

// The number of outstanding (live) handles in the arena.
pub(super) const MAX_HANDLE_COUNT: u32 = 256 * 1024;

// Warning level: When the number of handles exceeds this value, we start to emit
// warnings to the kernel's debug log.
pub(super) const HIGH_HANDLE_COUNT: u32 = (MAX_HANDLE_COUNT * 7) / 8;

// Masks for building a Handle's base_value, which ProcessDispatcher
// uses to create zx_handle_t values.
//
// base_value bit fields:
//   [31..(32 - kHandleReservedBits)]                     : Must be zero
//   [(31 - kHandleReservedBits)..kHandleGenerationShift] : Generation number
//                                                          Masked by kHandleGenerationMask
//   [kHandleGenerationShift-1..0]                        : Index into handle_arena
//                                                          Masked by kHandleIndexMask
pub(super) const HANDLE_RESERVED_BITS: u32 = 2;
pub(super) const HANDLE_INDEX_MASK: u32 = MAX_HANDLE_COUNT - 1;
pub(super) const HANDLE_RESERVED_BITS_MASK: u32 = ((1 << HANDLE_RESERVED_BITS) - 1) << (32 - HANDLE_RESERVED_BITS);
pub(super) const HANDLE_GENERATION_MASK: u32 = !HANDLE_INDEX_MASK & !HANDLE_RESERVED_BITS_MASK;
pub(super) const HANDLE_GENERATION_SHIFT: u32 = log2_uint_floor(MAX_HANDLE_COUNT);

const_assert!((HANDLE_INDEX_MASK & MAX_HANDLE_COUNT) == 0); //kMaxHandleCount must be a power of 2
const_assert!(((3 << (HANDLE_GENERATION_SHIFT - 1)) & HANDLE_GENERATION_MASK) == 1 << HANDLE_GENERATION_SHIFT); //Shift is wrong
const_assert!((HANDLE_GENERATION_MASK >> HANDLE_GENERATION_SHIFT) >= 255); // Not enough room for a useful generation count
const_assert!((HANDLE_RESERVED_BITS_MASK & HANDLE_GENERATION_MASK) == 0); // Handle Mask Overlap!
const_assert!((HANDLE_RESERVED_BITS_MASK & HANDLE_INDEX_MASK) == 0); // Handle Mask Overlap!
const_assert!((HANDLE_GENERATION_MASK & HANDLE_INDEX_MASK) == 0); // Handle Mask Overlap!
const_assert!((HANDLE_RESERVED_BITS_MASK | HANDLE_GENERATION_MASK | HANDLE_INDEX_MASK) == 0xffffffff); // Handle masks do not cover all bits!
