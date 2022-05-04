use std::any::Any;
use std::rc::Rc;

use fiber_sys as sys;

use crate::object::{dispatcher, Handle, HandleOwner, IDispatcher, JobDispatcher, JobPolicy, ProcessDispatcher, H};

#[derive(Debug, Clone, Copy)]
pub struct HandleTable {
    // The actual handle table.  When removing one or more handles from this list, be sure to
    // advance or invalidate any cursors that might point to the handles being removed.
    count: u32, // TA_GUARDED(lock_) = 0;
                //handles: Vec<Box<dyn H>>, //TA_GUARDED(lock_);

                // The containing ProcessDispatcher.
                // process: RefCell<ProcessDispatcher>,
}

impl HandleTable {
    pub(super) fn new() -> Self {
        HandleTable {
            count: 0,
            //handles: vec![],
        }
    }

    // Maps a |handle| to an integer which can be given to usermode as a
    // handle value. Uses Handle->base_value() plus additional mixing.
    fn map_handle_to_value<T>(handle: *const Handle<T>) -> sys::fx_handle_t {
        unimplemented!()
    }

    fn map_handle_owner_to_value<T>(handle: &HandleOwner<T>) -> sys::fx_handle_t {
        unimplemented!()
    }

    // Returns the number of outstanding handles in this handle table.
    fn handle_count() -> u32 {
        unimplemented!()
    }

    pub fn is_handle_valid(handle_value: sys::fx_handle_t) -> bool {
        unimplemented!()
    }
    pub fn get_koid_for_handle(handle_value: sys::fx_handle_t) -> sys::fx_koid_t {
        unimplemented!()
    }

    // Get the dispatcher corresponding to this handle value.
    //pub fn get_dispatcher<T>(&self, handle_value: sys::fx_handle_t) -> (sys::fx_status_t, Option<Rc<T>>) {
    //    self.get_dispatcher_with_rights(handle_value, 0)
    //}

    /// Get the dispatcher and the rights corresponding to this handle value.
    pub fn get_dispatcher_with_rights<T: 'static>(
        &self,
        handle_value: sys::fx_handle_t,
        rights: sys::fx_rights_t,
    ) -> (sys::fx_status_t, Option<Rc<T>>) {
        let (status, generic_dispatcher) = self.get_dispatcher_internal(handle_value, rights);

        if status != sys::FX_OK {
            return (status, None);
        }

        let dispatcher = generic_dispatcher.downcast::<T>().ok();

        if dispatcher.is_none() {
            return (sys::FX_ERR_WRONG_TYPE, None);
        }

        return (sys::FX_OK, dispatcher);
    }

    fn get_dispatcher_internal(
        &self,
        handle_value: sys::fx_handle_t,
        rights: sys::fx_rights_t,
    ) -> (sys::fx_status_t, Rc<dyn Any>) {
        let dispatcher: Rc<dyn Any> = Rc::from(JobDispatcher::new(0, None, JobPolicy));
        (sys::FX_OK, dispatcher)
    }
}
