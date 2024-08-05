import { u32 } from "./types"

export const FX_PROCARGS_PROTOCOL = 0x4150585d // MXPA
export const FX_PROCARGS_VERSION = 0x0001000

// This is a protocol for passing state to a new process
// via a message in a channel.
export interface fx_proc_args {
    // Protocol and version identifiers to allow for
    // different process start message protocols and
    // versioning of the same.
    protocol: u32
    version: u32

    // Offset from start of message to handle info
    // array, which contains one uint32_t per handle
    // passed along with the message.
    handle_info_off: u32

    // Offset from start of message to arguments and
    // count of arguments.  Arguments are provided as
    // a set of null-terminated utf-8 strings, one
    // after the other.
    args_off: u32
    args_num: u32

    // Offset from start of message to environment strings and count of
    // them.  Environment entries are provided as a set of null-terminated
    // UTF-8 strings, one after the other.  Canonically each string has
    // the form "NAME=VALUE", but nothing enforces this.
    environ_off: u32
    environ_num: u32

    // Offset from start of message to name strings and count of them.
    // These strings are packed similar to the argument strings,
    // but are referenced by PA_NS_* handle table entries and used
    // to set up namespaces.
    //
    // Specifically: In a handle table entry with PA_HND_TYPE(info)
    // of PA_NS_*, PA_HND_ARG(info) is an index into this name table.
    names_off: u32
    names_num: u32
}

// Handle to our own process.
export const PA_PROC_SELF = 0x01

// Handle to a job object which can be used to make child processes.
//
// The job can be the same as the one used to create this process or it can
// be different.
export const PA_JOB_DEFAULT = 0x03

// --- Namespace Handles ---

// A handle which will handle OPEN requests relative
// to a particular path which is specified by the
// nametable entry referred to by the "arg" field
export const PA_NS_DIR = 0x20

// --- Various ---

// Handle types for one-off use and prototyping
export const PA_USER0 = 0xF0
export const PA_USER1 = 0xF1
export const PA_USER2 = 0xF2
