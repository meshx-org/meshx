// Copyright 2023 MeshX Contributors
// Copyright 2016 The Fuchsia Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

pub(crate) const FX_PROCARGS_PROTOCOL: u32 = 0x4150585d; // MXPA
pub(crate) const FX_PROCARGS_VERSION: u32 = 0x0001000;

#[derive(Debug, Default)]
pub(crate) struct fx_proc_args_t {
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

pub(crate) const PROC_SELF: usize = 0;
pub(crate) const ROOT_JOB: usize = 1;

pub(crate) const HANDLE_COUNT: usize = 3;
pub(crate) const CHILD_HANDLE_COUNT: usize = HANDLE_COUNT + 5;

/*fn extract_handles(channel: fx::Channel) -> [sys::fx_handle_t; HANDLE_COUNT] {
    return [0; HANDLE_COUNT];
}

#[repr(u32)]
enum Enum {
    // These describe userboot itself.
    ProcSelf,
    // NOTE: no address kVmarRootSelf,

    // Essential job and resource handles.
    RootJob,
    RootResource,

    FirstInstrumentationData,
    HandleCount = FirstInstrumentationData + InstrumentationData::vmo_count(),
}

// This is the processargs message the child will receive. The command
// line block the kernel sends us is formatted exactly to become the
// environ strings for the child message, so we read it directly into
// the same buffer.
struct ChildMessageLayout {
    header: fx_proc_args_t,
    info: [u32; kChildHandleCount],
    cmdline: [char; kCmdlineMax],
}

// This is the main logic:
// 1. Read the kernel's bootstrap message.
// 2. Load up the child process from ELF file(s) on the bootfs.
// 3. Create the initial thread and allocate a stack for it.
// 4. Load up a channel with the zx_proc_args_t message for the child.
// 5. Start the child process running.
// 6. Optionally, wait for it to exit and then shut down.
fn bootstrap(channel: fx::Channel) {
    // We pass all the same handles the kernel gives us along to the child,
    // except replacing our own process/root-VMAR handles with its, and
    // passing along the three extra handles (BOOTFS, thread-self, and a debuglog
    // handle tied to stdout).
    let handles = extract_handles(channel);

    let child_message = ChildMessageLayout::default();

    // Process the kernel command line, which gives us options and also
    // becomes the environment strings for our child.
    let (opts, environ_num) = parse_options(child_message.cmdline, cmdline_len, &o);
    child_message.pargs.environ_num = environ_num;

    // Fill in the child message header.
    child_message.pargs.protocol = ZX_PROCARGS_PROTOCOL;
    child_message.pargs.version = ZX_PROCARGS_VERSION;
    child_message.pargs.handle_info_off = offsetof(child_message_layout, info);
    child_message.pargs.environ_off = offsetof(child_message_layout, cmdline);

    // Fill in the handle info table.
    child_message.info[kBootfsVmo] = PA_HND(PA_VMO_BOOTFS, 0);
    child_message.info[kProcSelf] = PA_HND(PA_PROC_SELF, 0);
    child_message.info[kRootJob] = PA_HND(PA_JOB_DEFAULT, 0);
    child_message.info[kVmarRootSelf] = PA_HND(PA_VMAR_ROOT, 0);

    // Hang on to our own process handle. If we closed it, our process
    // would be killed. Exiting will clean it up.
    let proc_self = handles[kProcSelf];
    handles[kProcSelf] = sys::FX_HANDLE_INVALID;

    // Make the channel for the bootstrap message.
    let (to_child, child_start_handle) = fx::Channel::create();
    // TODO: check(log.get(), status, "zx_channel_create failed");

    let root_job = unsafe { fx::Handle::from_raw(handles[kRootJob]) };
    let root_job = fx::Job::from_handle(root_job);

    let filename = o.value[OPTION_FILENAME];

    // Create the process itself.
    let result = root_job.create_child_process(filename);
    let (proc, vmar) = result.expect("fx_process_create failed");

    // Duplicate the child's process handle to pass to it.
    let status = fx::sys::fx_handle_duplicate(proc.get(), ZX_RIGHT_SAME_RIGHTS, &handles[kProcSelf]);
    // TODO: check(log.get(), status,  "zx_handle_duplicate failed on child process handle");

    // Now send the bootstrap message. This transfers away all the handles
    // we have left except the process and thread themselves.
    let status = to_child.write(&child_message, handles.data());
    // check(log.get(), status, "zx_channel_write to child failed");
    to_child.reset();

    // Start the process going.
    let status = proc.start(entry, sp, child_start_handle, vdso_base);
    // TODO: check(log.get(), status, "zx_process_start failed");
    thread.reset();

    // TODO wait for the child to exit, if requested.

    // Now we've accomplished our purpose in life, and we can die happy.
    proc.reset();

    println!("finished!");
    fx::Process::exit(0);
}
*/
