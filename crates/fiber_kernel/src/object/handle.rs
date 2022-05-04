// Copyright 2022 MeshX Contributors. All rights reserved.
// Copyright 2016 The Fuchsia Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use std::rc::Rc;
use std::sync::atomic::AtomicU64;

use crate::object::IDispatcher;

use super::dispatcher::Dispatcher;
use fiber_sys::{fx_koid_t, fx_rights_t};

// HandleOwner wraps a Handle in a Box that has single
// ownership of the Handle and deletes it whenever it falls out of scope.
pub(crate) type HandleOwner<T> = Box<Handle<T>>;

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
pub(crate) struct Handle<T> {
    // process_id_ is atomic because threads from different processes can
    // access it concurrently, while holding different instances of
    // handle_table_lock_.
    process_id: AtomicU64,
    dispatcher: Rc<T>,
    handle_rights: fx_rights_t,
}

pub(crate) trait H {}
impl<T> H for Handle<T> {}

impl<T> Handle<T> {
    // Handle should never be created by anything other than Make or Dup.
    pub fn new_from_dispatcher(dispatcher: Rc<T>, rights: fx_rights_t) -> HandleOwner<T> {
        Box::from(Handle {
            process_id: AtomicU64::new(0),
            handle_rights: rights,
            dispatcher,
        })
    }

    pub fn new(handle: KernelHandle<T>, rights: fx_rights_t) -> HandleOwner<T> {
        Box::from(Handle {
            process_id: AtomicU64::new(0),
            handle_rights: rights,
            dispatcher: handle.dispatcher().clone(),
        })
    }

    //pub fn dup(source: *const Handle<T>, rights: fx_rights_t) -> HandleOwner<T> {
    //    Box::from(Handle {
    //        process_id: AtomicU64::new(0),
    //        handle_rights: rights,
    //    })
    //}

    /// Returns the Dispatcher to which this instance points.
    pub fn dispatcher(&self) -> &Rc<T> {
        return &self.dispatcher;
    }

    /// Returns the process that owns this instance. Used to guarantee
    /// that one process may not access a handle owned by a different process.
    pub fn process_id(&self) -> fx_koid_t {
        self.process_id.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Sets the value returned by process_id().
    pub fn set_process_id(&self, pid: fx_koid_t) {}

    /// Returns the |rights| parameter that was provided when this instance
    /// was created.
    pub fn rights(&self) -> fx_rights_t {
        self.handle_rights
    }

    /// To be called once during bring up.
    pub fn init() {}

    // Maps an integer obtained by Handle::base_value() back to a Handle.
    //pub fn from_u32(value: u32) -> Handle<T> {
    //    Handle {
    //        process_id: AtomicU64::new(0),
    //        dispatcher: Rc::new(Dispatcher::new()),
    //        handle_rights: 0,
    //    }
    //}

    /// Get the number of outstanding handles for a given dispatcher.
    pub fn count(dp: Rc<Dispatcher>) -> u32 {
        0
    }

    /* Private */
}
