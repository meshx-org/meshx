use std::rc::Rc;

use fiber_sys as sys;

use crate::object::{IDispatcher, KernelHandle};

pub(crate) struct VmarDispatcher {}

impl IDispatcher for VmarDispatcher {
    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_VMAR
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

impl VmarDispatcher {
    pub(crate) fn create(flags: u32) -> Result<(KernelHandle<VmarDispatcher>, sys::fx_rights_t), sys::fx_status_t> {
        let vmar_rights: sys::fx_rights_t = VmarDispatcher::default_rights();
        //TODO: parse vmar flags

        let handle = KernelHandle {
            dispatcher: Rc::from(VmarDispatcher::new(0)),
        };

        return Ok((handle, vmar_rights));
    }

    fn new(flags: u32) -> Self {
        VmarDispatcher {}
    }
}
