// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2016 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#![allow(non_camel_case_types)]

pub type fx_clock_t = u32;

pub type fx_handle_t = u32;
pub type fx_handle_op_t = u32;
pub type fx_koid_t = u64;
pub type fx_obj_type_t = u32;
pub type fx_object_info_topic_t = u32;
pub type fx_info_maps_type_t = u32;
pub type fx_off_t = u64;
pub type fx_paddr_t = usize;
pub type fx_vaddr_t = usize;
pub type fx_rights_t = u32;
pub type fx_signals_t = u32;
pub type fx_policy_t = u32;
pub type fx_ssize_t = isize;
pub type fx_status_t = i32;
pub type fx_txid_t = u32;
pub type fx_vm_option_t = u32;

/// Absolute time in nanoseconds (generally with respect to the monotonic clock)
pub type fx_time_t = i64;
/// A duration in nanoseconds
pub type fx_duration_t = i64;
/// A duration in hardware ticks
pub type fx_ticks_t = i64;

// Ports V2
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum fx_packet_type_t {
    FX_PKT_TYPE_USER = 0,
    FX_PKT_TYPE_SIGNAL_ONE = 1,
    FX_PKT_TYPE_SIGNAL_REP = 2,
}

impl Default for fx_packet_type_t {
    fn default() -> Self {
        fx_packet_type_t::FX_PKT_TYPE_USER
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fx_packet_signal_t {
    pub trigger: fx_signals_t,
    pub observed: fx_signals_t,
    pub count: u64,
}

// Actually a union of different integer types, but this should be good enough.
pub type fx_packet_user_t = [u8; 32];

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct fx_port_packet_t {
    pub key: u64,
    pub packet_type: fx_packet_type_t,
    pub status: i32,
    pub union: [u8; 32],
}

pub const FX_WAIT_ASYNC_TIMESTAMP: u32 = 1;
pub const FX_WAIT_ASYNC_EDGE: u32 = 2;

// object property constants
pub const FX_MAX_NAME_LEN: usize = 32;

// channel write size constants
pub const FX_CHANNEL_MAX_MSG_HANDLES: u32 = 64;
pub const FX_CHANNEL_MAX_MSG_BYTES: u32 = 65536;

// Task response codes if a process is externally killed
pub const FX_TASK_RETCODE_SYSCALL_KILL: i64 = -1024;
pub const FX_TASK_RETCODE_OOM_KILL: i64 = -1025;
pub const FX_TASK_RETCODE_POLICY_KILL: i64 = -1026;
pub const FX_TASK_RETCODE_VDSO_KILL: i64 = -1027;
pub const FX_TASK_RETCODE_EXCEPTION_KILL: i64 = -1028;

pub const FX_HANDLE_FIXED_BITS_MASK: u32 = 0x80000000;

macro_rules! multiconst {
    ($typename:ident, [$($(#[$attr:meta])* $rawname:ident = $value:expr;)*]) => {
        $(
            $(#[$attr])*
            pub const $rawname: $typename = $value;
        )*
    }
}

multiconst!(fx_handle_t, [
    FX_HANDLE_INVALID = 0;
]);

multiconst!(fx_handle_op_t, [
    FX_HANDLE_OP_MOVE = 0;
    FX_HANDLE_OP_DUPLICATE = 1;
]);

multiconst!(fx_koid_t, [
    FX_KOID_INVALID = 0;
    FX_KOID_KERNEL = 1;
    FX_KOID_FIRST = 1024;
]);

multiconst!(fx_time_t, [
    FX_TIME_INFINITE = ::std::i64::MAX;
    FX_TIME_INFINITE_PAST = ::std::i64::MIN;
]);

multiconst!(fx_rights_t, [
    FX_RIGHT_NONE           = 0;
    FX_RIGHT_DUPLICATE      = 1 << 0;
    FX_RIGHT_TRANSFER       = 1 << 1;
    FX_RIGHT_READ           = 1 << 2;
    FX_RIGHT_WRITE          = 1 << 3;
    FX_RIGHT_EXECUTE        = 1 << 4;
    FX_RIGHT_MAP            = 1 << 5;
    FX_RIGHT_GET_PROPERTY   = 1 << 6;
    FX_RIGHT_SET_PROPERTY   = 1 << 7;
    FX_RIGHT_ENUMERATE      = 1 << 8;
    FX_RIGHT_DESTROY        = 1 << 9;
    FX_RIGHT_SET_POLICY     = 1 << 10;
    FX_RIGHT_GET_POLICY     = 1 << 11;
    FX_RIGHT_SIGNAL         = 1 << 12;
    FX_RIGHT_SIGNAL_PEER    = 1 << 13;
    FX_RIGHT_WAIT           = 1 << 14;
    FX_RIGHT_INSPECT        = 1 << 15;
    FX_RIGHT_MANAGE_JOB     = 1 << 16;
    FX_RIGHT_MANAGE_PROCESS = 1 << 17;
    FX_RIGHT_MANAGE_THREAD  = 1 << 18;
    FX_RIGHT_APPLY_PROFILE  = 1 << 19;
    FX_RIGHT_MANAGE_SOCKET  = 1 << 20;
    FX_RIGHT_SAME_RIGHTS    = 1 << 31;

    // Convenient names for commonly grouped rights.
    FX_RIGHTS_BASIC = FX_RIGHT_TRANSFER | FX_RIGHT_DUPLICATE | FX_RIGHT_WAIT | FX_RIGHT_INSPECT;
    FX_RIGHTS_IO = FX_RIGHT_READ | FX_RIGHT_WRITE;
    FX_RIGHTS_PROPERTY = FX_RIGHT_GET_PROPERTY | FX_RIGHT_SET_PROPERTY;
    FX_DEFAULT_CHANNEL_RIGHTS = (FX_RIGHTS_BASIC & (!FX_RIGHT_DUPLICATE)) | FX_RIGHTS_IO | FX_RIGHT_SIGNAL | FX_RIGHT_SIGNAL_PEER;
    FX_DEFAULT_PROCESS_RIGHTS    = FX_RIGHTS_BASIC | FX_RIGHTS_IO | FX_RIGHTS_PROPERTY | FX_RIGHT_ENUMERATE | FX_RIGHT_DESTROY | FX_RIGHT_SIGNAL | FX_RIGHT_MANAGE_PROCESS | FX_RIGHT_MANAGE_THREAD;
    FX_DEFAULT_VMO_RIGHTS  = FX_RIGHTS_BASIC | FX_RIGHTS_IO | FX_RIGHTS_PROPERTY | FX_RIGHT_MAP | FX_RIGHT_SIGNAL;
]);

multiconst!(u32, [
    FX_VMO_RESIZABLE = 1 << 1;
    FX_VMO_DISCARDABLE = 1 << 2;
    FX_VMO_TRAP_DIRTY = 1 << 3;
]);

multiconst!(fx_status_t, [
    FX_OK                         = 0;
    FX_ERR_INTERNAL               = -1;
    FX_ERR_NOT_SUPPORTED          = -2;
    FX_ERR_NO_RESOURCES           = -3;
    FX_ERR_NO_MEMORY              = -4;
    FX_ERR_INTERRUPTED_RETRY      = -6;
    FX_ERR_INVALID_ARGS           = -10;
    FX_ERR_BAD_HANDLE             = -11;
    FX_ERR_WRONG_TYPE             = -12;
    FX_ERR_BAD_SYSCALL            = -13;
    FX_ERR_OUT_OF_RANGE           = -14;
    FX_ERR_BUFFER_TOO_SMALL       = -15;
    FX_ERR_BAD_STATE              = -20;
    FX_ERR_TIMED_OUT              = -21;
    FX_ERR_SHOULD_WAIT            = -22;
    FX_ERR_CANCELED               = -23;
    FX_ERR_PEER_CLOSED            = -24;
    FX_ERR_NOT_FOUND              = -25;
    FX_ERR_ALREADY_EXISTS         = -26;
    FX_ERR_ALREADY_BOUND          = -27;
    FX_ERR_UNAVAILABLE            = -28;
    FX_ERR_ACCESS_DENIED          = -30;
    FX_ERR_IO                     = -40;
    FX_ERR_IO_REFUSED             = -41;
    FX_ERR_IO_DATA_INTEGRITY      = -42;
    FX_ERR_IO_DATA_LOSS           = -43;
    FX_ERR_IO_NOT_PRESENT         = -44;
    FX_ERR_IO_OVERRUN             = -45;
    FX_ERR_IO_MISSED_DEADLINE     = -46;
    FX_ERR_IO_INVALID             = -47;
    FX_ERR_BAD_PATH               = -50;
    FX_ERR_NOT_DIR                = -51;
    FX_ERR_NOT_FILE               = -52;
    FX_ERR_FILE_BIG               = -53;
    FX_ERR_NO_SPACE               = -54;
    FX_ERR_NOT_EMPTY              = -55;
    FX_ERR_STOP                   = -60;
    FX_ERR_NEXT                   = -61;
    FX_ERR_ASYNC                  = -62;
    FX_ERR_PROTOCOL_NOT_SUPPORTED = -70;
    FX_ERR_ADDRESS_UNREACHABLE    = -71;
    FX_ERR_ADDRESS_IN_USE         = -72;
    FX_ERR_NOT_CONNECTED          = -73;
]);

multiconst!(fx_signals_t, [
    FX_SIGNAL_NONE              = 0;
    FX_OBJECT_SIGNAL_ALL        = 0x00ffffff;
    FX_USER_SIGNAL_ALL          = 0xff000000;
    FX_OBJECT_SIGNAL_0          = 1 << 0;
    FX_OBJECT_SIGNAL_1          = 1 << 1;
    FX_OBJECT_SIGNAL_2          = 1 << 2;
    FX_OBJECT_SIGNAL_3          = 1 << 3;
    FX_OBJECT_SIGNAL_4          = 1 << 4;
    FX_OBJECT_SIGNAL_5          = 1 << 5;
    FX_OBJECT_SIGNAL_6          = 1 << 6;
    FX_OBJECT_SIGNAL_7          = 1 << 7;
    FX_OBJECT_SIGNAL_8          = 1 << 8;
    FX_OBJECT_SIGNAL_9          = 1 << 9;
    FX_OBJECT_SIGNAL_10         = 1 << 10;
    FX_OBJECT_SIGNAL_11         = 1 << 11;
    FX_OBJECT_SIGNAL_12         = 1 << 12;
    FX_OBJECT_SIGNAL_13         = 1 << 13;
    FX_OBJECT_SIGNAL_14         = 1 << 14;
    FX_OBJECT_SIGNAL_15         = 1 << 15;
    FX_OBJECT_SIGNAL_16         = 1 << 16;
    FX_OBJECT_SIGNAL_17         = 1 << 17;
    FX_OBJECT_SIGNAL_18         = 1 << 18;
    FX_OBJECT_SIGNAL_19         = 1 << 19;
    FX_OBJECT_SIGNAL_20         = 1 << 20;
    FX_OBJECT_SIGNAL_21         = 1 << 21;
    FX_OBJECT_SIGNAL_22         = 1 << 22;
    FX_OBJECT_HANDLE_CLOSED     = 1 << 23;
    FX_USER_SIGNAL_0            = 1 << 24;
    FX_USER_SIGNAL_1            = 1 << 25;
    FX_USER_SIGNAL_2            = 1 << 26;
    FX_USER_SIGNAL_3            = 1 << 27;
    FX_USER_SIGNAL_4            = 1 << 28;
    FX_USER_SIGNAL_5            = 1 << 29;
    FX_USER_SIGNAL_6            = 1 << 30;
    FX_USER_SIGNAL_7            = 1 << 31;
    FX_OBJECT_READABLE          = FX_OBJECT_SIGNAL_0;
    FX_OBJECT_WRITABLE          = FX_OBJECT_SIGNAL_1;
    FX_OBJECT_PEER_CLOSED       = FX_OBJECT_SIGNAL_2;
    // Cancelation (handle was closed while waiting with it)
    FX_SIGNAL_HANDLE_CLOSED     = FX_OBJECT_HANDLE_CLOSED;
    // Event
    FX_EVENT_SIGNALED           = FX_OBJECT_SIGNAL_3;
    // EventPair
    FX_EVENTPAIR_SIGNALED       = FX_OBJECT_SIGNAL_3;
    FX_EVENTPAIR_CLOSED         = FX_OBJECT_SIGNAL_2;
    // Task signals (process, thread, job)
    FX_TASK_TERMINATED          = FX_OBJECT_SIGNAL_3;
    // Channel
    FX_CHANNEL_READABLE         = FX_OBJECT_SIGNAL_0;
    FX_CHANNEL_WRITABLE         = FX_OBJECT_SIGNAL_1;
    FX_CHANNEL_PEER_CLOSED      = FX_OBJECT_SIGNAL_2;
    // Clock
    FX_CLOCK_STARTED            = FX_OBJECT_SIGNAL_4;
    // Socket
    FX_SOCKET_READABLE            = FX_OBJECT_READABLE;
    FX_SOCKET_WRITABLE            = FX_OBJECT_WRITABLE;
    FX_SOCKET_PEER_CLOSED         = FX_OBJECT_PEER_CLOSED;
    FX_SOCKET_PEER_WRITE_DISABLED = FX_OBJECT_SIGNAL_4;
    FX_SOCKET_WRITE_DISABLED      = FX_OBJECT_SIGNAL_5;
    FX_SOCKET_READ_THRESHOLD      = FX_OBJECT_SIGNAL_10;
    FX_SOCKET_WRITE_THRESHOLD     = FX_OBJECT_SIGNAL_11;
    // Job
    FX_JOB_TERMINATED           = FX_OBJECT_SIGNAL_3;
    FX_JOB_NO_JOBS              = FX_OBJECT_SIGNAL_4;
    FX_JOB_NO_PROCESSES         = FX_OBJECT_SIGNAL_5;
    // Process
    FX_PROCESS_TERMINATED       = FX_OBJECT_SIGNAL_3;
    // Thread
    FX_THREAD_TERMINATED        = FX_OBJECT_SIGNAL_3;
    FX_THREAD_RUNNING           = FX_OBJECT_SIGNAL_4;
    FX_THREAD_SUSPENDED         = FX_OBJECT_SIGNAL_5;
    // Log
    FX_LOG_READABLE             = FX_OBJECT_READABLE;
    FX_LOG_WRITABLE             = FX_OBJECT_WRITABLE;
    // Timer
    FX_TIMER_SIGNALED           = FX_OBJECT_SIGNAL_3;
    // Vmo
    FX_VMO_ZERO_CHILDREN        = FX_OBJECT_SIGNAL_3;
]);

multiconst!(fx_obj_type_t, [
    FX_OBJ_TYPE_NONE                = 0;
    FX_OBJ_TYPE_PROCESS             = 1;
    FX_OBJ_TYPE_VMO                 = 3;
    FX_OBJ_TYPE_CHANNEL             = 4;
    FX_OBJ_TYPE_EVENT               = 5;
    FX_OBJ_TYPE_PORT                = 6;
    FX_OBJ_TYPE_JOB                 = 17;
    FX_OBJ_TYPE_VMAR                = 18;
]);

// System ABI commits to having no more than 64 object types.
//
// See zx_info_process_handle_stats_t for an example of a binary interface that
// depends on having an upper bound for the number of object types.
pub const FX_OBJ_TYPE_UPPER_BOUND: usize = 64;

multiconst!(fx_object_info_topic_t, [
    FX_INFO_NONE                       = 0;
    FX_INFO_HANDLE_VALID               = 1;
    FX_INFO_HANDLE_BASIC               = 2;  // fx_info_handle_basic_t[1]
]);

multiconst!(fx_policy_t, [
    // policy options
    FX_JOB_POLICY_RELATIVE = 0;
    FX_JOB_POLICY_ABSOLUTE = 1;

    // policy topic
    FX_JOB_POLICY_BASIC = 0;
    FX_JOB_POLICY_TIMER_SLACK = 1;

    // policy conditions
    FX_POLICY_BAD_HANDLE            = 0;
    FX_POLICY_WRONG_OBJECT          = 1;
    FX_POLICY_NEW_ANY               = 3;
    FX_POLICY_NEW_VMO               = 4;
    FX_POLICY_NEW_CHANNEL           = 5;
    FX_POLICY_NEW_TIMER             = 11;
    FX_POLICY_NEW_PROCESS           = 12;

    // policy actions
    FX_POLICY_ACTION_ALLOW           = 0;
    FX_POLICY_ACTION_DENY            = 1;
    FX_POLICY_ACTION_ALLOW_EXCEPTION = 2;
    FX_POLICY_ACTION_DENY_EXCEPTION  = 3;
    FX_POLICY_ACTION_KILL            = 4;

    // timer slack default modes
    FX_TIMER_SLACK_CENTER = 0;
    FX_TIMER_SLACK_EARLY  = 1;
    FX_TIMER_SLACK_LATE   = 2;
]);

multiconst!(u32, [
    // critical options
    FX_JOB_CRITICAL_PROCESS_RETCODE_NONZERO = 1 << 0;
]);

// TODO: add an alias for this type in the C headers.
multiconst!(fx_vm_option_t, [
    FX_VM_PERM_READ             = 1 << 0;
    FX_VM_PERM_WRITE            = 1 << 1;
    FX_VM_PERM_EXECUTE          = 1 << 2;
    FX_VM_COMPACT               = 1 << 3;
    FX_VM_SPECIFIC              = 1 << 4;
    FX_VM_SPECIFIC_OVERWRITE    = 1 << 5;
    FX_VM_CAN_MAP_SPECIFIC      = 1 << 6;
    FX_VM_CAN_MAP_READ          = 1 << 7;
    FX_VM_CAN_MAP_WRITE         = 1 << 8;
    FX_VM_CAN_MAP_EXECUTE       = 1 << 9;
    FX_VM_MAP_RANGE             = 1 << 10;
    FX_VM_REQUIRE_NON_RESIZABLE = 1 << 11;
    FX_VM_ALLOW_FAULTS          = 1 << 12;
    FX_VM_OFFSET_IS_UPPER_LIMIT = 1 << 13;
]);

// Don't need struct_decl_macro for this, the wrapper is different.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct fx_info_handle_basic_t {
    pub koid: fx_koid_t,
    pub rights: fx_rights_t,
    pub type_: fx_obj_type_t,
    pub related_koid: fx_koid_t,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct fx_handle_info_t {
    pub handle: fx_handle_t,
    pub ty: fx_obj_type_t,
    pub rights: fx_rights_t,
    pub unused: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct zx_channel_call_args_t {
    pub wr_bytes: *const u8,
    pub wr_handles: *const fx_handle_t,
    pub rd_bytes: *mut u8,
    pub rd_handles: *mut fx_handle_t,
    pub wr_num_bytes: u32,
    pub wr_num_handles: u32,
    pub rd_num_bytes: u32,
    pub rd_num_handles: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct fx_channel_call_etc_args_t {
    pub wr_bytes: *const u8,
    pub wr_handles: *mut fx_handle_disposition_t,
    pub rd_bytes: *mut u8,
    pub rd_handles: *mut fx_handle_info_t,
    pub wr_num_bytes: u32,
    pub wr_num_handles: u32,
    pub rd_num_bytes: u32,
    pub rd_num_handles: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct fx_handle_disposition_t {
    pub operation: fx_handle_op_t,
    pub handle: fx_handle_t,
    pub type_: fx_obj_type_t,
    pub rights: fx_rights_t,
    pub result: fx_status_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct fx_policy_basic {
    pub condition: u32,
    pub policy: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct fx_policy_timer_slack {
    pub min_slack: fx_duration_t,
    pub default_mode: u32,
}
