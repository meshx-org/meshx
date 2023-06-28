use std::{any::Any, sync::RwLock};
use std::collections::VecDeque;
use std::mem::size_of;
use std::rc::Rc;

use fiber_sys as sys;
use generational_arena::{Arena, Index};
use once_cell::unsync::Lazy;

use crate::object::{
    Dispatcher, Handle, HandleOwner, ProcessDispatcher, HANDLE_GENERATION_MASK, HANDLE_GENERATION_SHIFT,
    HANDLE_INDEX_MASK, HANDLE_RESERVED_BITS, MAX_HANDLE_COUNT,
};

pub(crate) struct HandleTableArena {
    pub arena: Arena<Handle>,
}

// |index| is the literal index into the table. |old_value| is the
// |index mixed with the per-handle-lifetime state.
fn new_handle_value(index: u32, old_value: u32) -> u32 {
    debug_assert_eq!((index & !HANDLE_INDEX_MASK), 0);

    let mut old_gen = 0;

    if old_value != 0 {
        // This slot has been used before.
        debug_assert_eq!((old_value & HANDLE_INDEX_MASK), index);
        old_gen = (old_value & HANDLE_GENERATION_MASK) >> HANDLE_GENERATION_SHIFT;
    }

    let new_gen = ((old_gen + 1) << HANDLE_GENERATION_SHIFT) & HANDLE_GENERATION_MASK;

    index | new_gen
}

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

impl HandleTableArena {
    fn handle_to_index(&self, handle: *const Handle) -> u32 {
        // return handle - self.arena.base()

        return handle as u32;
    }

    // Returns a new |base_value| based on the value stored in the free
    // arena slot pointed to by |addr|. The new value will be different
    // from the last |base_value| used by this slot.
    fn get_new_base_value(&self, addr: *const ()) -> u32 {
        // Get the index of this slot within the arena.
        let handle_index = self.handle_to_index(addr as *const Handle);

        // Check the free memory for a stashed base_value.
        let v = unsafe { (*(addr as *const Handle)).base_value };

        new_handle_value(handle_index, v)
    }

    /// Allocate space for a Handle from the arena, but don't instantiate the
    /// object.  |base_value| gets the value for Handle::base_value_.  |what|
    /// says whether this is allocation or duplication, for the error message.
    fn alloc(&mut self, dispatcher: &Rc<dyn Dispatcher>, what: &str) -> Index {
        // Attempt to allocate a handle.
        let idx = self.arena.insert(Handle {
            process_id: sys::FX_KOID_INVALID.into(),
            dispatcher: todo!(),
            handle_rights: todo!(),
            base_value: todo!(),
        });

        let outstanding_handles = self.arena.len();

        //if (unlikely(addr == nullptr)) {
        //    kcounter_add(handle_count_alloc_failed, 1);
        //    printf("WARNING: Could not allocate %s handle (%zu outstanding)\n", what, outstanding_handles);
        //    return nullptr;
        //}

        // Emit a warning if too many handles have been created and we haven't recently logged
        //if (unlikely(outstanding_handles > kHighHandleCount) && handle_count_high_log_.Ready()) {
        //    printf("WARNING: High handle count: %zu / %zu handles\n", outstanding_handles,
        //            kHighHandleCount);
        //}

        dispatcher.base().increment_handle_count();

        // checking the process_id_ and dispatcher is really about trying to catch cases where this
        // Handle might somehow already be in use.
        //debug_assert!((addr).process_id().eq(&sys::FX_KOID_INVALID) == true);
        //debug_assert!((addr).dispatcher() == nullptr);

        // NOTE: we don't have a cocept to return bases here so instead we return the index
        // *base_value = GetNewBaseValue(addr);
        // return addr;

        idx
    }

    fn delete(&self, handle: *const Handle) {
        let handle = unsafe { &(*handle) };

        let dispatcher = handle.dispatcher().clone();

        let old_base_value = handle.base_value;
        let base_value = &handle.base_value;

        // There may be stale pointers to this slot and they will look at process_id. We expect
        // process_id to already have been cleared by the process dispatcher before the handle got to
        // this point.
        debug_assert!(handle.process_id() == sys::FX_KOID_INVALID);

        // TODO:
        //if (dispatcher.is_waitable()) {
        //    dispatcher.cancel(handle);
        //}

        // The destructor should not do anything interesting but call it for completeness.
        std::mem::forget(handle);

        // Make sure the base value was not altered by the destructor.
        debug_assert!(unsafe { *base_value } == old_base_value);

        let zero_handles = dispatcher.base().decrement_handle_count();
        // self.arena.remove(handle);

        // TODO: we need downcast for this
        //if (zero_handles) {
        //    dispatcher.on_zero_handles();
        //}

        // If |disp| is the last reference (which is likely) then the dispatcher object
        // gets destroyed at the exit of this function.
        //kcounter_add(handle_count_live, -1);
    }
}

pub(crate) const HANDLE_TABLE: Lazy<HandleTableArena> = Lazy::new(|| HandleTableArena {
    arena: Arena::with_capacity(size_of::<Handle>() * MAX_HANDLE_COUNT as usize),
});

#[derive(Debug)]
struct GuardedState {
    // The actual handle table.  When removing one or more handles from this list, be sure to
    // advance or invalidate any cursors that might point to the handles being removed.
    count: u32,                      // TA_GUARDED(lock_) = 0;
    handles: VecDeque<Box<dyn Any>>, //TA_GUARDED(lock_);
}

#[derive(Debug)]
pub struct HandleTable {
    guarded: RwLock<GuardedState>,

    // The containing ProcessDispatcher.
    process: *const ProcessDispatcher,
}

impl HandleTable {
    pub(super) fn new(process: *const ProcessDispatcher) -> Self {
        HandleTable {
            process,
            guarded: RwLock::new(GuardedState {
                count: 0,
                handles: VecDeque::new(),
            }),
        }
    }

    // Maps a |handle| to an integer which can be given to usermode as a
    // handle value. Uses Handle->base_value() plus additional mixing.
    pub(crate) fn map_handle_to_value(&self, handle: *const Handle) -> sys::fx_handle_t {
        unimplemented!()
    }

    pub(crate) fn map_handle_owner_to_value<T>(handle: &HandleOwner) -> sys::fx_handle_t {
        unimplemented!()
    }

    // Returns the number of outstanding handles in this handle table.
    pub(crate) fn handle_count(&self) -> u32 {
        self.guarded.read().unwrap().count
    }

    pub(crate) fn is_handle_valid(&self, handle_value: sys::fx_handle_t) -> bool {
        unimplemented!()
    }

    pub(crate) fn get_koid_for_handle(&self, handle_value: sys::fx_handle_t) -> sys::fx_koid_t {
        unimplemented!()
    }

    pub(crate) fn add_handle(&self, handle: HandleOwner) {
        //Guard<BrwLockPi, BrwLockPi::Writer> guard{&lock_};
        self.add_handle_locked(handle);
    }

    pub(crate) fn add_handle_locked(&self, handle: HandleOwner) {
        // NOTE: We need to use unsafe and raw pointer here to access the parent so we can avoid circular dependency issues.
        let koid = unsafe { (*self.process).get_koid() };
        handle.set_process_id(koid);
        
        let mut guarded = self.guarded.write().unwrap();

        guarded.handles.push_front(handle);
        guarded.count += 1;
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

        let rights = handle.rights();

        Ok(handle.dispatcher().clone())
    }
}
