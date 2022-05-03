use std::rc::Rc;

use fiber_sys as sys;

use crate::object::JobDispatcher;

pub struct HandleTable;

impl HandleTable {
    pub(crate) fn get_dispatcher_with_rights(
        &self,
        handle: sys::fx_handle_t,
        rights: sys::fx_rights_t,
    ) -> (sys::fx_status_t, Rc<JobDispatcher>) {
        (sys::FX_OK, unsafe { Rc::from_raw(&JobDispatcher {} as *const JobDispatcher) })
    }
}
