/*static zx_status_t Create(fbl::RefPtr<VmObject> vmo,
                          fbl::RefPtr<ContentSizeManager> content_size_manager,
                          InitialMutability initial_mutability,
                          KernelHandle<VmObjectDispatcher>* handle, zx_rights_t* rights) {
  return Create(ktl::move(vmo), ktl::move(content_size_manager), ZX_KOID_INVALID,
                initial_mutability, handle, rights);
}*/

use std::{any::Any, sync::Arc};

use super::{BaseDispatcher, Dispatcher, KernelHandle, TypedDispatcher};
use fiber_sys as sys;

#[derive(Debug)]
pub(crate) struct VMODispatcher {
    base: BaseDispatcher,
}

impl Dispatcher for VMODispatcher {
    fn get_koid(&self) -> sys::fx_koid_t {
        self.base.get_koid()
    }

    fn get_related_koid(&self) -> sys::fx_koid_t {
        0
    }

    fn base(&self) -> &super::BaseDispatcher {
        &self.base
    }
}

impl TypedDispatcher for VMODispatcher {
    fn default_rights() -> sys::fx_rights_t {
        sys::FX_DEFAULT_VMO_RIGHTS
    }

    fn get_type() -> sys::fx_obj_type_t {
        sys::FX_OBJ_TYPE_VMO
    }
}

impl VMODispatcher {
    pub fn create() -> (sys::fx_status_t, Option<KernelHandle<VMODispatcher>>, sys::fx_rights_t) {
        let new_handle = KernelHandle::new(super::GenericDispatcher::VMODispatcher(VMODispatcher::new().into()));

        (sys::FX_OK, Some(new_handle), VMODispatcher::default_rights())
    }

    pub fn new() -> Arc<VMODispatcher> {
        Arc::new(VMODispatcher {
            base: BaseDispatcher::new(sys::FX_VMO_ZERO_CHILDREN),
        })
    }
}
