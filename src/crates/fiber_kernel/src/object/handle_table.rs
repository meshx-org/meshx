use std::collections::VecDeque;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::RwLock;
use std::sync::{Arc, Weak};

use fiber_sys as sys;
use generational_arena::{Arena, Index};
use once_cell::unsync::Lazy;
use rand::Rng;

use super::GenericDispatcher;
use crate::koid::generate;
use crate::object::{
    Dispatcher, Handle, HandleOwner, ProcessDispatcher, HANDLE_GENERATION_MASK, HANDLE_GENERATION_SHIFT,
    HANDLE_INDEX_MASK, HANDLE_RESERVED_BITS, MAX_HANDLE_COUNT,
};

pub(crate) struct HandleTableArena {
    pub arena: Arena<Arc<Handle>>,
}

impl HandleTableArena {
    pub(crate) fn handle_to_index(&self, handle: *const Handle) -> u32 {
        // return handle - self.arena.base()

        return handle as u32;
    }

    // Returns a new |base_value| based on the value stored in the free
    // arena slot pointed to by |addr|. The new value will be different
    // from the last |base_value| used by this slot.
    pub(crate) fn get_new_base_value(&self, addr: *const ()) -> u32 {
        // Get the index of this slot within the arena.
        let handle_index = self.handle_to_index(addr as *const Handle);

        // Check the free memory for a stashed base_value.
        let v = unsafe { (*(addr as *const Handle)).base_value };

        new_handle_value(handle_index, v)
    }

    /// Allocate space for a Handle from the arena, but don't instantiate the
    /// object.  |base_value| gets the value for Handle::base_value_.  |what|
    /// says whether this is allocation or duplication, for the error message.
    pub(crate) fn alloc(&mut self, dispatcher: &Arc<dyn Dispatcher>, what: &str) -> Index {
        // Attempt to allocate a handle.
        let idx = self.arena.insert(Arc::new(Handle {
            handle_table_id: sys::FX_KOID_INVALID.into(),
            dispatcher: todo!(),
            handle_rights: todo!(),
            base_value: todo!(),
        }));

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

    pub(crate) fn delete(&self, handle: *const Handle) {
        let handle = unsafe { &(*handle) };

        let dispatcher = handle.dispatcher();

        let old_base_value = handle.base_value;
        let base_value = &handle.base_value;

        // There may be stale pointers to this slot and they will look at process_id. We expect
        // process_id to already have been cleared by the process dispatcher before the handle got to
        // this point.
        debug_assert!(handle.handle_table_id() == sys::FX_KOID_INVALID);

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

pub(crate) const HANDLE_TABLE_ARENA: Lazy<HandleTableArena> = Lazy::new(|| HandleTableArena {
    arena: Arena::with_capacity(size_of::<Handle>() * MAX_HANDLE_COUNT as usize),
});

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

fn map_handle_to_value(handle: &Handle, mixer: u32) -> sys::fx_handle_t {
    // Ensure that the last two bits of the result is not zero, and make sure we
    // don't lose any base_value bits when shifting.
    let base_value_must_be_zero_mask =
        HANDLE_MUST_BE_ONE_MASK << ((std::mem::size_of_val(&handle.base_value()) as u32 * 8) - HANDLE_RESERVED_BITS);

    debug_assert!((mixer & HANDLE_MUST_BE_ONE_MASK) == 0);
    debug_assert!((handle.base_value() & base_value_must_be_zero_mask) == 0);

    let handle_id = (handle.base_value() << HANDLE_RESERVED_BITS) | HANDLE_MUST_BE_ONE_MASK;

    mixer ^ handle_id
}

fn map_value_to_handle(value: sys::fx_handle_t, mixer: u32) -> Option<Weak<Handle>> {
    // Validate that the "must be one" bits are actually one.
    if (value & HANDLE_MUST_BE_ONE_MASK) != HANDLE_MUST_BE_ONE_MASK {
        return None;
    }

    let handle_id = ((value as u32) ^ mixer) >> HANDLE_RESERVED_BITS;

    Handle::from_u32(handle_id)
}

#[derive(Debug)]
struct GuardedState {
    // The actual handle table.  When removing one or more handles from this list, be sure to
    // advance or invalidate any cursors that might point to the handles being removed.
    count: u32,
    handles: VecDeque<Weak<Handle>>,
}

#[derive(Debug)]
pub(crate) struct HandleTable {
    guarded: RwLock<GuardedState>,

    // Normalized parent process koid.
    process_koid: sys::fx_koid_t,

    // The koid of this handle table. Used to check whether or not a handle belongs to this handle
    // table (and thus that it belongs to a process associated with this handle table).
    koid: sys::fx_koid_t,

    // Each handle table provides pseudorandom userspace handle
    // values. This is the per-handle-table pseudorandom state.
    random_value: u32, //  = 0;
}

impl HandleTable {
    pub(super) fn new(process: &ProcessDispatcher) -> Self {
        // Generate handle XOR mask with top bit and bottom two bits cleared
        let mut prng = rand::thread_rng();
        let secret: u32 = prng.gen();

        // Handle values must always have the low kHandleReservedBits set.  Do not
        // ever attempt to toggle these bits using the random_value_ xor mask.
        let random_value = secret << HANDLE_RESERVED_BITS;

        HandleTable {
            koid: generate(),
            random_value,
            process_koid: process.get_koid(),
            guarded: RwLock::new(GuardedState {
                count: 0,
                handles: VecDeque::new(),
            }),
        }
    }

    // Maps a |handle| to an integer which can be given to usermode as a
    // handle value. Uses Handle->base_value() plus additional mixing.
    pub(crate) fn map_handle_to_value(&self, handle: &Handle) -> sys::fx_handle_t {
        return map_handle_to_value(handle, self.random_value);
    }

    pub(crate) fn map_handle_owner_to_value<T>(&self, handle: &HandleOwner) -> sys::fx_handle_t {
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
        handle.set_process_id(self.process_koid);

        let mut guarded = self.guarded.write().unwrap();

        guarded.handles.push_front(Arc::downgrade(&handle));
        guarded.count += 1;
    }

    // Maps a handle value into a Handle as long we can verify that
    // it belongs to this handle table.
    pub(crate) fn get_handle_locked(
        &self,
        caller: &ProcessDispatcher,
        handle_value: sys::fx_handle_t,
    ) -> Option<Arc<Handle>> {
        let handle = map_value_to_handle(handle_value, self.random_value);

        if handle.is_none() {
            return None;
        }

        let handle = handle.unwrap().upgrade().unwrap();

        if handle.handle_table_id() != self.koid {
            return None;
        }

        // TODO: enforce policy
        //if caller {
        // Handle lookup failed.  We potentially generate an exception or kill the process,
        // depending on the job policy. Note that we don't use the return value from
        // EnforceBasicPolicy() here: ZX_POL_ACTION_ALLOW and ZX_POL_ACTION_DENY are equivalent for
        // ZX_POL_BAD_HANDLE.
        // let result = caller.enforce_basic_policy(sys::FX_POL_BAD_HANDLE);
        //}

        Some(handle)
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

    // Get the dispatcher corresponding to this handle value.
    pub fn get_dispatcher(
        &self,
        caller: &ProcessDispatcher,
        handle_value: sys::fx_handle_t,
    ) -> Result<GenericDispatcher, sys::fx_status_t> {
        self.get_dispatcher_with_rights(caller, handle_value, sys::FX_RIGHT_NONE)
    }

    /// Get the dispatcher and the rights corresponding to this handle value.
    pub fn get_dispatcher_with_rights(
        &self,
        caller: &ProcessDispatcher,
        handle_value: sys::fx_handle_t,
        rights: sys::fx_rights_t,
    ) -> Result<GenericDispatcher, sys::fx_status_t> {
        let generic_dispatcher = self.get_dispatcher_internal(caller, handle_value, rights)?;

        return Ok(generic_dispatcher);
    }

    fn get_dispatcher_internal(
        &self,
        caller: &ProcessDispatcher,
        handle_value: sys::fx_handle_t,
        rights: sys::fx_rights_t,
    ) -> Result<GenericDispatcher, sys::fx_status_t> {
        //let dispatcher: Rc<dyn Any> = Rc::from(JobDispatcher::new(0, None, JobPolicy));

        let handle = self.get_handle_locked(caller, handle_value);

        if handle.is_none() {
            return Err(sys::FX_ERR_BAD_HANDLE);
        }

        let handle = handle.unwrap();

        let rights = handle.rights();

        Ok(handle.dispatcher())
    }
}
