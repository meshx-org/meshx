use std::rc::Rc;

use fiber_sys as sys;

use crate::object::{IDispatcher, KernelHandle};

pub(crate) struct DataObjectDispatcher {}

impl IDispatcher for DataObjectDispatcher {
    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_DATAVIEW
    }

    fn get_koid(&self) -> sys::fx_koid_t {
        0
    }

    fn get_related_koid(&self) -> sys::fx_koid_t {
        0
    }

    fn default_rights() -> sys::fx_rights_t {
        sys::FX_RIGHT_NONE
    }
}

impl DataObjectDispatcher {
    pub(crate) fn create() -> Result<(KernelHandle<DataObjectDispatcher>, sys::fx_rights_t), sys::fx_status_t> {
        let rights = DataObjectDispatcher::default_rights();

        let handle = KernelHandle {
            dispatcher: Rc::new(DataObjectDispatcher::new()),
        };

        return Ok((handle, rights));
    }

    fn new() -> Self {
        DataObjectDispatcher {}
    }
}
