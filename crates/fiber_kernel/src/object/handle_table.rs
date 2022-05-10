use std::any::Any;
use std::collections::VecDeque;
use std::rc::Rc;

use fiber_sys as sys;
use once_cell::sync::Lazy;
use static_assertions::const_assert;

use crate::object::{Dispatcher, Handle, HandleOwner, ProcessDispatcher, HANDLE_RESERVED_BITS};

pub struct Arena;

impl Arena {
    pub fn base(&self) -> *const () {
        std::ptr::null()
    }

    pub fn committed(&self, node: *const ()) -> bool {
        false
    }
}

pub struct HandleTableArena {
    pub arena: Arena,
}

pub static HANDLE_TABLE: Lazy<HandleTableArena> = Lazy::new(|| HandleTableArena { arena: Arena });

const HANDLE_MUST_BE_ONE_MASK: u32 = (0x1 << HANDLE_RESERVED_BITS) - 1;
//const_assert!(HANDLE_MUST_BE_ONE_MASK == sys::FX_HANDLE_FIXED_BITS_MASK); // kHandleMustBeOneMask must match ZX_HANDLE_FIXED_BITS_MASK!

fn map_value_to_handle(value: sys::fx_handle_t, mixer: u32) -> *const Handle {
    // Validate that the "must be one" bits are actually one.
    if (value & HANDLE_MUST_BE_ONE_MASK) != HANDLE_MUST_BE_ONE_MASK {
        return std::ptr::null();
    }

    let handle_id = ((value as u32) ^ mixer) >> HANDLE_RESERVED_BITS;
    return Handle::from_u32(handle_id);
}

struct LockedState {}

#[derive(Debug)]
pub struct HandleTable {
    // The actual handle table.  When removing one or more handles from this list, be sure to
    // advance or invalidate any cursors that might point to the handles being removed.
    count: u32,                      // TA_GUARDED(lock_) = 0;
    handles: VecDeque<Box<dyn Any>>, //TA_GUARDED(lock_);

    // The containing ProcessDispatcher.
    process: *const ProcessDispatcher,
}

impl HandleTable {
    pub(super) fn new(process: *const ProcessDispatcher) -> Self {
        HandleTable {
            count: 0,
            handles: VecDeque::new(),
            process,
        }
    }

    // Maps a |handle| to an integer which can be given to usermode as a
    // handle value. Uses Handle->base_value() plus additional mixing.
    fn map_handle_to_value<T>(handle: *const Handle) -> sys::fx_handle_t {
        unimplemented!()
    }

    fn map_handle_owner_to_value<T>(handle: &HandleOwner) -> sys::fx_handle_t {
        unimplemented!()
    }

    // Returns the number of outstanding handles in this handle table.
    fn handle_count(&self) -> u32 {
        self.count
    }

    pub fn is_handle_valid(&self, handle_value: sys::fx_handle_t) -> bool {
        unimplemented!()
    }

    pub fn get_koid_for_handle(&self, handle_value: sys::fx_handle_t) -> sys::fx_koid_t {
        unimplemented!()
    }

    fn add_handle(&mut self, handle: HandleOwner) {
        //Guard<BrwLockPi, BrwLockPi::Writer> guard{&lock_};
        self.add_handle_locked(handle);
    }

    fn add_handle_locked(&mut self, handle: HandleOwner) {
        // NOTE: We need to use unsafe and raw pointer here to access the parent so we can avoid circular dependency issues.
        let koid = unsafe { (*self.process).get_koid() };
        handle.set_process_id(koid);
        self.handles.push_front(handle);
        self.count += 1;
    }

    /*fn get_handle_locked<T>(&self, handle_value: sys::fx_handle_t, skip_policy: bool) -> *const Handle<T> {
        let handle = Handle::from(handle_id);
        if (handle && handle.process_id() == self.process.get_koid()) {
            return handle;
        }

        if (likely(!skip_policy)) {
            // Handle lookup failed.  We potentially generate an exception or kill the process,
            // depending on the job policy. Note that we don't use the return value from
            // EnforceBasicPolicy() here: ZX_POL_ACTION_ALLOW and ZX_POL_ACTION_DENY are equivalent for
            // ZX_POL_BAD_HANDLE.
            let result = self.process.enforce_basic_policy(sys::FX_POLICY_BAD_HANDLE);
        }

        return nullptr;
    }*/

    fn get_handle_locked(&self, handle_value: sys::fx_handle_t, skip_policy: bool) -> *const Handle {
        let handle = map_value_to_handle(handle_value, 0);

        unsafe {
            if !handle.is_null() && (*handle).process_id() == (*self.process).get_koid() {
                handle
            } else {
                std::ptr::null()
            }
        }
    }

    // Get the dispatcher corresponding to this handle value.
    pub fn get_dispatcher<T: 'static>(&self, handle_value: sys::fx_handle_t) -> Result<Rc<T>, sys::fx_status_t> {
        self.get_dispatcher_with_rights(handle_value, sys::FX_RIGHT_NONE)
    }

    /// Get the dispatcher and the rights corresponding to this handle value.
    pub fn get_dispatcher_with_rights<T: 'static>(
        &self,
        handle_value: sys::fx_handle_t,
        rights: sys::fx_rights_t,
    ) -> Result<Rc<T>, sys::fx_status_t> {
        let generic_dispatcher = self.get_dispatcher_internal(handle_value, rights)?;

        let dispatcher = generic_dispatcher.downcast::<T>();

        if dispatcher.is_err() {
            return Err(sys::FX_ERR_WRONG_TYPE);
        }

        return Ok(dispatcher.unwrap());
    }

    fn get_dispatcher_internal(
        &self,
        handle_value: sys::fx_handle_t,
        rights: sys::fx_rights_t,
    ) -> Result<Rc<dyn Any>, sys::fx_status_t> {
        //let dispatcher: Rc<dyn Any> = Rc::from(JobDispatcher::new(0, None, JobPolicy));

        //Guard<BrwLockPi, BrwLockPi::Reader> guard{&lock_};
        let handle: *const Handle = self.get_handle_locked(handle_value, false);

        if handle == std::ptr::null() {
            return Err(sys::FX_ERR_BAD_HANDLE);
        }

        let handle = unsafe { &*handle };

        let mut rights = handle.rights();

        Ok(handle.dispatcher().clone())
    }
}
