mod dispatcher;
mod handle;
mod handle_table;
mod job_dispatcher;
mod process_dispatcher;
mod vmar_dispatcher;

pub use dispatcher::*;
pub use handle::*;
pub use handle_table::*;
pub use job_dispatcher::*;
pub use process_dispatcher::*;
pub use vmar_dispatcher::*;

pub struct KernelObject;
pub struct ProcessObject(pub KernelObject);
pub struct VmoObject(pub KernelObject);

impl ProcessObject {
    pub fn get_vmo(&self) -> VmoObject {
        VmoObject(KernelObject)
    }
}
