use std::rc::Rc;

use fiber_sys as sys;

use crate::object::{IDispatcher, KernelHandle};

pub(crate) struct DataViewDispatcher {}

impl IDispatcher for DataViewDispatcher {
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

impl DataViewDispatcher {
    pub(crate) fn create(
        dv: Rc<&[u8]>,
    ) -> Result<(KernelHandle<DataViewDispatcher>, sys::fx_rights_t), sys::fx_status_t> {
        let rights = DataViewDispatcher::default_rights();

        let handle = KernelHandle {
            dispatcher: Rc::new(DataViewDispatcher::new()),
        };

        return Ok((handle, rights));
    }

    fn new() -> Self {
        DataViewDispatcher {}
    }
}
