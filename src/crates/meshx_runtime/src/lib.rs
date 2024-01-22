// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Copyright 2024 MeshX Contributors. All rights reserved.

//! Type-safe bindings for MeshX-specific `libc` functionality.
//!
//! This crate is a minimal extension on top of the `fiber_rust` crate,
//! which provides bindings to the Fiber kernel's syscalls, but does not
//! depend on functionality from `libc`.

use {
    fiber_rust::{
        sys::{fx_handle_t, fx_status_t, FX_HANDLE_INVALID}, // handle type (primitive, non-owning)
        Handle,
        HandleBased,
        Job,
        Process,
        Rights,
        Status,
        Time,
        Unowned,
    },
    std::convert::TryFrom,
};

#[allow(missing_docs)]
extern "C" {
    pub fn dl_clone_loader_service(out: *mut fx_handle_t) -> fx_status_t;
    pub fn fx_take_startup_handle(hnd_info: u32) -> fx_handle_t;
    pub fn fx_thread_self() -> fx_handle_t;
    pub fn fx_process_self() -> fx_handle_t;
    pub fn thrd_get_fx_process() -> fx_handle_t;
    pub fn fx_vmar_root_self() -> fx_handle_t;
    pub fn fx_job_default() -> fx_handle_t;
    pub fn fx_utc_reference_get() -> fx_handle_t;
    pub fn fx_utc_reference_swap(new_handle: fx_handle_t, prev_handle: *mut fx_handle_t) -> fx_status_t;
}

/// Handle types as defined by the processargs protocol.
///
/// See [//zircon/system/public/zircon/processargs.h][processargs.h] for canonical definitions.
///
/// Short descriptions of each handle type are given, but more complete documentation may be found
/// in the [processargs.h] header.
///
/// [processargs.h]: https://fuchsia.googlesource.com/fuchsia/+/HEAD/zircon/system/public/zircon/processargs.h
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HandleType {
    /// Handle to our own process.
    ///
    /// Equivalent to PA_PROC_SELF.
    ProcessSelf = 0x01,
    /// Handle to the initial thread of our own process.
    ///
    /// Equivalent to PA_THREAD_SELF.
    ThreadSelf = 0x02,
    /// Handle to a job object which can be used to make child processes.
    ///
    /// The job can be the same as the one used to create this process or it can
    /// be different.
    ///
    /// Equivalent to PA_JOB_DEFAULT.
    DefaultJob = 0x03,
    /// Handle to the root of our address space.
    ///
    /// Equivalent to PA_VMAR_ROOT.
    RootVmar = 0x04,
    /// Handle to the VMAR used to load the initial program image.
    ///
    /// Equivalent to PA_VMAR_LOADED.
    LoadedVmar = 0x05,
    /// Service for loading shared libraries.
    ///
    /// See `fuchsia.ldsvc.Loader` for the interface definition.
    ///
    /// Equivalent to PA_LDSVC_LOADER.
    LdsvcLoader = 0x10,
    /// Handle to the VMO containing the vDSO ELF image.
    ///
    /// Equivalent to PA_VMO_VDSO.
    VdsoVmo = 0x11,
    /// Handle to the VMO used to map the initial thread's stack.
    ///
    /// Equivalent to PA_VMO_STACK.
    StackVmo = 0x13,
    /// Handle to the VMO for the main executable file.
    ///
    /// Equivalent to PA_VMO_EXECUTABLE.
    ExecutableVmo = 0x14,
    /// Used by kernel and userboot during startup.
    ///
    /// Equivalent to PA_VMO_BOOTDATA.
    BootdataVmo = 0x1A,
    /// Used by kernel and userboot during startup.
    ///
    /// Equivalent to PA_VMO_BOOTFS.
    BootfsVmo = 0x1B,
    /// Used by the kernel to export debug information as a file in bootfs.
    ///
    /// Equivalent to PA_VMO_KERNEL_FILE.
    KernelFileVmo = 0x1C,
    /// A Handle to a component's process' configuration VMO.
    ///
    /// Equivalent to PA_VMO_COMPONENT_CONFIG.
    ComponentConfigVmo = 0x1D,
    /// A handle to a fuchsia.io.Directory service to be used as a directory in the process's
    /// namespace. Corresponds to a path in the processargs bootstrap message's namespace table
    /// based on the argument of a HandleInfo of this type.
    ///
    /// Equivalent to PA_NS_DIR.
    NamespaceDirectory = 0x20,
    /// A handle which will be used as a file descriptor.
    ///
    /// Equivalent to PA_FD.
    FileDescriptor = 0x30,
    /// A Handle to a channel on which the process may serve the
    /// the |fuchsia.process.Lifecycle| protocol.
    ///
    /// Equivalent to PA_LIFECYCLE.
    Lifecycle = 0x3A,
    /// Server endpoint for handling connections to a process's outgoing directory.
    ///
    /// Equivalent to PA_DIRECTORY_REQUEST.
    DirectoryRequest = 0x3B,
    /// A Handle to a resource object. Used by devcoordinator and devhosts.
    ///
    /// Equivalent to PA_RESOURCE.
    Resource = 0x3F,
    /// A Handle to a clock object representing UTC.  Used by runtimes to gain
    /// access to UTC time.
    ///
    /// Equivalent to PA_CLOCK_UTC.
    ClockUtc = 0x40,
    /// A Handle to an MMIO resource object.
    ///
    /// Equivalent to PA_MMIO_RESOURCE.
    MmioResource = 0x50,
    /// A Handle to an IRQ resource object.
    ///
    /// Equivalent to PA_IRQ_RESOURCE.
    IrqResource = 0x51,
    /// A Handle to an IO Port resource object.
    ///
    /// Equivalent to PA_IOPORT_RESOURCE.
    IoportResource = 0x52,
    /// A Handle to an SMC resource object.
    ///
    /// Equivalent to PA_SMC_RESOURCE.
    SmcResource = 0x53,
    /// A Handle to the System resource object.
    ///
    /// Equivalent to PA_SYSTEM_RESOURCE.
    SystemResource = 0x54,
    /// A handle type with user-defined meaning.
    ///
    /// Equivalent to PA_USER0.
    User0 = 0xF0,
    /// A handle type with user-defined meaning.
    ///
    /// Equivalent to PA_USER1.
    User1 = 0xF1,
    /// A handle type with user-defined meaning.
    ///
    /// Equivalent to PA_USER2.
    User2 = 0xF2,
}

/// Metadata information for a handle in a processargs message. Contains a handle type and an
/// unsigned 16-bit value, whose meaning is handle type dependent.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HandleInfo {
    htype: HandleType,
    arg: u16,
}

impl HandleInfo {
    /// Create a handle info struct from a handle type and an argument.
    ///
    /// For example, a `HandleInfo::new(HandleType::FileDescriptor, 32)` identifies
    /// the respective handle as file descriptor 32.
    ///
    /// Corresponds to PA_HND in processargs.h.
    pub const fn new(htype: HandleType, arg: u16) -> Self {
        HandleInfo { htype, arg }
    }

    /// Returns the handle type for this handle info struct.
    #[inline(always)]
    pub fn handle_type(&self) -> HandleType {
        self.htype
    }

    /// Returns the argument for this handle info struct.
    #[inline(always)]
    pub fn arg(&self) -> u16 {
        self.arg
    }

    /// Convert the handle info into a raw u32 value for FFI purposes.
    pub const fn as_raw(&self) -> u32 {
        ((self.htype as u32) & 0xFF) | (self.arg as u32) << 16
    }
}

/// An implementation of the From trait to create a [HandleInfo] from a [HandleType] with an argument
/// of 0.
impl From<HandleType> for HandleInfo {
    fn from(ty: HandleType) -> Self {
        Self::new(ty, 0)
    }
}

/// Removes the handle of type `HandleType` from the list of handles received at startup.
///
/// This function will return `Some` at-most once per handle type.
/// This function will return `None` if the requested type was not received at
/// startup or if the handle with the provided type was already taken.
pub fn take_startup_handle(info: HandleInfo) -> Option<Handle> {
    unsafe {
        let raw = fx_take_startup_handle(info.as_raw());
        if raw == FX_HANDLE_INVALID {
            None
        } else {
            Some(Handle::from_raw(raw))
        }
    }
}

/// Get a reference to the handle of the current process.
pub fn process_self() -> Unowned<'static, Process> {
    unsafe {
        // zx_process_self() doesn't work correctly in jobs where multiple processes share
        // the portion of their address space for global variables. Use thrd_get_zx_process() to
        // return the correct value in that context. See https://fxbug.dev/133807 for background.
        let handle = thrd_get_fx_process();
        Unowned::from_raw_handle(handle)
    }
}

/// Get a reference to the default `Job` provided to the process on startup.
///
/// This typically refers to the `Job` that is the immediate parent of the current
/// process.
///
/// If the current process was launched as a Fuchsia Component, this `Job`
/// will begin with no child processes other than the current process.
pub fn job_default() -> Unowned<'static, Job> {
    unsafe {
        let handle = fx_job_default();
        Unowned::from_raw_handle(handle)
    }
}
