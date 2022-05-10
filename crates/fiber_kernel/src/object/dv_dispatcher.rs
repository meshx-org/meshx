use std::rc::Rc;

use fiber_sys as sys;

use crate::object::{BaseDispatcher, Dispatcher, KernelHandle, TypedDispatcher};

pub(crate) struct DataViewDispatcher {
    base: BaseDispatcher,
}

impl Dispatcher for DataViewDispatcher {
    fn get_koid(&self) -> sys::fx_koid_t {
        self.base.get_koid()
    }

    fn get_related_koid(&self) -> sys::fx_koid_t {
        0
    }

    fn base(&self) -> &BaseDispatcher {
        &self.base
    }
}

impl TypedDispatcher for DataViewDispatcher {
    fn default_rights() -> sys::fx_rights_t {
        sys::FX_RIGHT_NONE
    }

    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_DATAVIEW
    }
}

impl DataViewDispatcher {
    fn new() -> Self {
        DataViewDispatcher {
            base: BaseDispatcher::new(),
        }
    }

    pub(crate) fn create(
        dv: Rc<&[u8]>,
    ) -> Result<(KernelHandle<DataViewDispatcher>, sys::fx_rights_t), sys::fx_status_t> {
        let rights = DataViewDispatcher::default_rights();

        let handle = KernelHandle {
            dispatcher: Rc::new(DataViewDispatcher::new()),
        };

        return Ok((handle, rights));
    }
}
