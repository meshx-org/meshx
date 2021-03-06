mod dispatcher;
mod handle;
mod handle_table;
mod job_dispatcher;
mod process_dispatcher;
mod dv_dispatcher;
mod do_dispatcher;

pub(crate) use dispatcher::*;
pub(crate) use handle::*;
pub(crate) use handle_table::*;
pub(crate) use job_dispatcher::*;
pub(crate) use process_dispatcher::*;
pub(crate) use dv_dispatcher::*;
pub(crate) use do_dispatcher::*;

#[derive(Debug)]
pub struct KernelObject;

#[derive(Debug)]
pub struct ProcessObject(pub KernelObject);

#[derive(Debug)]
pub struct VmoObject(pub KernelObject);

impl ProcessObject {
    pub fn get_vmo(&self) -> VmoObject {
        VmoObject(KernelObject)
    }
}
