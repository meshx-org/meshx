pub static FX_PROCARGS_PROTOCOL: u32 = 0x4150585du32;// MXPA
pub static FX_PROCARGS_VERSION: u32 = 0x0001000u32;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct fx_proc_args_t {
    // Protocol and version identifiers to allow for
    // different process start message protocols and
    // versioning of the same.
    pub protocol: u32,
    pub version: u32,

    // Offset from start of message to handle info
    // array, which contains one u32 per handle
    // passed along with the message.
    pub handle_info_off: u32,

    // Offset from start of message to arguments and
    // count of arguments.  Arguments are provided as
    // a set of null-terminated utf-8 strings, one
    // after the other.
    pub args_off: u32,
    pub args_num: u32,

    // Offset from start of message to environment strings and count of
    // them. Environment entries are provided as a set of null-terminated
    // UTF-8 strings, one after the other.  Canonically each string has
    // the form "NAME=VALUE", but nothing enforces this.
    pub environ_off: u32,
    pub environ_num: u32,

    // Offset from start of message to name strings and count of them.
    // These strings are packed similar to the argument strings,
    // but are referenced by PA_NS_* handle table entries and used
    // to set up namespaces.
    //
    // Specifically: In a handle table entry with PA_HND_TYPE(info)
    // of PA_NS_*, PA_HND_ARG(info) is an index into this name table.
    pub names_off: u32,
    pub names_num: u32,
}

pub const PA_PROC_SELF: u32 = 0x01;

// Handle to a job object which can be used to make child processes.
//
// The job can be the same as the one used to create this process or it can
// be different.
pub const PA_JOB_DEFAULT: u32 = 0x03;