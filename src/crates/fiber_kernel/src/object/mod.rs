mod channel_dispatcher;
mod dispatcher;
mod handle;
mod handle_table;
mod job_dispatcher;
mod message_packet;
mod process_dispatcher;
mod root_job_observer;
mod signal_observer;
mod vmo_dispatcher;
mod port_dispatcher;

use std::{ops::Deref, sync::Arc};

pub(crate) use dispatcher::*;
pub(crate) use handle::*;
pub(crate) use handle_table::*;
pub(crate) use message_packet::*;
pub(crate) use job_dispatcher::*;
pub(crate) use process_dispatcher::*;
pub(crate) use vmo_dispatcher::*;
pub(crate) use channel_dispatcher::*;
pub(crate) use port_dispatcher::*;
pub(crate) use root_job_observer::*;
pub(crate) use signal_observer::*;

#[derive(Debug, Clone)]
pub(crate) enum GenericDispatcher {
    ProcessDispatcher(Arc<ProcessDispatcher>),
    ChannelDispatcher(Arc<ChannelDispatcher>),
    JobDispatcher(Arc<JobDispatcher>),
    PortDispatcher(Arc<PortDispatcher>),
    VMODispatcher(Arc<VMODispatcher>),
}

impl Deref for GenericDispatcher {
    type Target = dyn Dispatcher;

    fn deref(&self) -> &Self::Target {
        match self {
            GenericDispatcher::ProcessDispatcher(dispatcher) => dispatcher.as_ref(),
            GenericDispatcher::ChannelDispatcher(dispatcher) => dispatcher.as_ref(),
            GenericDispatcher::JobDispatcher(dispatcher) => dispatcher.as_ref(),
            GenericDispatcher::PortDispatcher(dispatcher) => dispatcher.as_ref(),
            GenericDispatcher::VMODispatcher(dispatcher) => dispatcher.as_ref(),
        }
    }
}

impl GenericDispatcher {
    pub(crate) fn as_job_dispatcher(&self) -> Option<Arc<JobDispatcher>> {
        match self {
            GenericDispatcher::JobDispatcher(dispatcher) => Some(dispatcher.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_process_dispatcher(&self) -> Option<Arc<ProcessDispatcher>> {
        match self {
            GenericDispatcher::ProcessDispatcher(dispatcher) => Some(dispatcher.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_channel_dispatcher(&self) -> Option<Arc<ChannelDispatcher>> {
        match self {
            GenericDispatcher::ChannelDispatcher(dispatcher) => Some(dispatcher.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_port_dispatcher(&self) -> Option<Arc<PortDispatcher>> {
        match self {
            GenericDispatcher::PortDispatcher(dispatcher) => Some(dispatcher.clone()),
            _ => None,
        }
    }
}

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
