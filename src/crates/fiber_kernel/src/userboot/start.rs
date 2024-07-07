// Copyright 2016 The Fuchsia Authors
// Copyright 2023 MeshX Contributors

use fiber_rust as fx;
use fiber_rust::HandleBased;
use fx::Handle;
use memoffset::offset_of;
use zerocopy::AsBytes;
use zerocopy::FromBytes;
use zerocopy::FromZeroes;

use crate::userboot::userboot::CHILD_HANDLE_COUNT;
use crate::userboot::userboot::PROC_SELF;

use super::message::HandleInfo;
use super::message::HandleType;
use super::userboot::fx_proc_args_t;
use super::userboot::FX_PROCARGS_PROTOCOL;
use super::userboot::FX_PROCARGS_VERSION;
use super::userboot::HANDLE_COUNT;
use super::userboot::PROCESS_ARGS_MAX_BYTES;
use super::userboot::ROOT_JOB;

const SVC_NAME_INDEX: u32 = 0;

fn handle_termination() {}

const fn handle_info_table() -> [HandleInfo; CHILD_HANDLE_COUNT] {
    let mut info: [HandleInfo; CHILD_HANDLE_COUNT] = [HandleInfo {
        htype: todo!(),
        arg: todo!(),
    }; CHILD_HANDLE_COUNT];

    info[PROC_SELF] = HandleInfo::new(HandleType::ProcessSelf, 0);
    info[ROOT_JOB] = HandleInfo::new(HandleType::DefaultJob, 0);

    info
}

// This is the processargs message the child will receive.
#[repr(C)]
#[derive(Debug, AsBytes, FromZeroes, FromBytes)]
struct ChildMessageLayout {
    header: fx_proc_args_t,
    args: [char; PROCESS_ARGS_MAX_BYTES],
    info: [HandleInfo; CHILD_HANDLE_COUNT],
    names: [char; 5], // cpp20::to_array("/svc");
}

fn create_child_message() -> ChildMessageLayout {
    let header = fx_proc_args_t {
        protocol: FX_PROCARGS_PROTOCOL,
        version: FX_PROCARGS_VERSION,
        handle_info_off: offset_of!(ChildMessageLayout, info) as u32,

        names_off: offset_of!(ChildMessageLayout, names) as u32,
        names_num: SVC_NAME_INDEX + 1,

        args_off: offset_of!(ChildMessageLayout, args) as u32,
        args_num: todo!(),

        environ_off: todo!(),
        environ_num: todo!(),
    };

    let mut child_message = ChildMessageLayout {
        header,
        args: todo!(),
        info: todo!(),
        names: todo!(),
    };

    child_message.header = header;
    child_message.info = handle_info_table();

    child_message
}

struct ChildContext {
    // ChildContext() = default;
    // ChildContext(ChildContext&&) = default;
    // ~ChildContext() { zx_handle_close_many(handles.data(), handles.size()); }

    // Process creation handles
    process: fx::Process,

    svc_client: fx::Channel,
    svc_server: fx::Channel,

    handles: Vec<Handle>,
}

fn create_child_context(name: &str, mut handles: Vec<Handle>) -> ChildContext {
    let root_job_handle = handles.remove(ROOT_JOB);
    let root_job = fx::Job::from_handle(root_job_handle);

    let status = root_job.create_child_process(name);
    if status.is_err() {
        log::error!("Failed to create child process({}).", name);
    }

    let (process, vmar) = status.unwrap();

    // FIXME: Squat on some address space before we start loading it up.
    // child.reserved_vmar = {ReserveLowAddressSpace(log, child.vmar)};

    // Create the initial thread in the new process
    // status = fx::thread::create(child.process, name.data(), name.len(), 0, &child.thread);
    // check(log, status, "Failed to create main thread for child process(%.*s).", static_cast<int>(name.length()), name.data());

    let (svc_client, svc_server) = fx::Channel::create();
    // check(log, status, "Failed to create svc channels.");

    let mut child = ChildContext {
        process,
        svc_client,
        svc_server,
        handles: Vec::with_capacity(CHILD_HANDLE_COUNT),
    };

    let mut i = 0;
    while i < handles.len() && i < HANDLE_COUNT {
        if i == PROC_SELF {
            continue;
        }

        //if (i == kVmarRootSelf) {
        //    continue;
        //}

        if handles[i] == fx::Handle::invalid() {
            continue;
        }

        // TODO: child.handles[i] = raw_duplicate_or_die(log, handles[i]);

        i += 1;
    }

    child
}

fn stash_svc() {}
fn set_child_handles() {}
fn set_stash_handle() {}

struct ProgramInfo;

impl ProgramInfo {
    fn filename(&self) -> &str {
        "test"
    }
}

struct BootFS;

fn start_child_process(
    elf_entry: &ProgramInfo,
    child_message: &ChildMessageLayout,
    child: &mut ChildContext,
    bootfs: &BootFS,
    handle_count: usize,
) {
    // let stack_size = ZIRCON_DEFAULT_STACK_SIZE;

    let (to_child, bootstrap) = fx::Channel::create();
    // check(log, status, "fx_channel_create failed for child stack");

    //let loader_svc: zx::Channel;

    // Examine the bootfs image and find the requested file in it.
    // This will handle a PT_INTERP by doing a second lookup in bootfs.
    // let entry: zx_vaddr_t = elf_load_bootfs(log, bootfs, elf_entry.root, child.process, child.vmar, elf_entry.filename(), to_child, &stack_size, &loader_svc);

    // Now load the vDSO into the child, so it has access to system calls.
    // let vdso_base: zx_vaddr_t = elf_load_vdso(log, child.vmar, *zx::unowned_vmo{child.handles[kFirstVdso]});

    // stack_size = (stack_size + zx_system_get_page_size() - 1) &  -static_cast<uint64_t>(zx_system_get_page_size());
    // zx::vmo stack_vmo;
    // status = zx::vmo::create(stack_size, 0, &stack_vmo);
    // check(log, status, "zx_vmo_create failed for child stack");

    // stack_vmo.set_property(ZX_PROP_NAME, kStackVmoName, sizeof(kStackVmoName) - 1);
    // zx_vaddr_t stack_base;
    // status = child.vmar.map(ZX_VM_PERM_READ | ZX_VM_PERM_WRITE, 0, stack_vmo, 0, stack_size, &stack_base);
    // check(log, status, "zx_vmar_map failed for child stack");

    // Allocate the stack for the child.
    // let sp: *const () = InitialStackPointer(stack_base, stack_size);
    // println!("stack [{}, {}) sp={}", stack_base as *const (), stack_base + stack_size) as *const (), sp as *const ());

    // We're done doing mappings, so clear out the reservation VMAR.
    // check(log, child.reserved_vmar.destroy(), "zx_vmar_destroy failed on reservation VMAR handle");
    // child.reserved_vmar.reset();

    // Now send the bootstrap message. This transfers away all the handles
    // we have left except the process and thread themselves.
    let status = to_child.write(child_message.as_bytes(), child.handles.as_mut_slice());
    if status.is_err() {
        log::error!("fx_channel_write to child failed");
    }

    // Start the process going.
    let status = child.process.start(0, bootstrap.into_handle());
    if status.is_err() {
        log::error!("fx_process_start failed");
    }

    //child.thread.reset();

    //loader_svc
}

fn parse_next_process_arguments() {}

fn wait_for_process_exit() {}

fn extract_handles(bootstrap: fx::Channel) -> Vec<fx::Handle> {
    // Default constructed debuglog will force check/fail to fallback to |zx_debug_write|.
    // zx::debuglog log;

    // Read the command line and the essential handles from the kernel.
    let mut buff = fx::MessageBuf::new();
    let status = bootstrap.read(&mut buff);

    if status.is_err() {
        log::error!("cannot read bootstrap message");
    }

    if buff.n_handles() != HANDLE_COUNT {
        log::error!("read {} handles instead of {}", buff.n_handles(), HANDLE_COUNT);
    }

    buff.take_handles()
}

// This is the main logic:
// 1. Read the kernel's bootstrap message.
// 2. Load up the child process from ELF file(s) on the bootfs.
// 3. Create the initial thread and allocate a stack for it.
// 4. Load up a channel with the zx_proc_args_t message for the child.
// 5. Start the child process running.
// 6. Optionally, wait for it to exit and then shut down.
fn bootstrap(channel: fx::Channel) {
    log::info!("Hello, world from user space!, {:?}", channel);

    // We pass all the same handles the kernel gives us along to the child,
    // except replacing our own process/root-VMAR handles with its, and
    // passing along the three extra handles (BOOTFS, thread-self, and a debuglog
    // handle tied to stdout).
    let mut handles = extract_handles(channel);

    // fx::debuglog log;
    // let status = fx::DebugLog::create(*fx::unowned_resource{handles[kRootResource]}, 0, &log);
    // check(log, status, "fx_debuglog_create failed: %d", status);

    // zx::vmar vmar_self{handles[kVmarRootSelf]};
    // handles[kVmarRootSelf] = sys::FX_HANDLE_INVALID;

    let proc = &handles[PROC_SELF];
    handles[PROC_SELF] = fx::Handle::invalid();

    let (svc_stash_server, svc_stash_client) = fx::Channel::create();
    // TODO: check(log, status, "Failed to create svc stash channel.");

    {
        // let borrowed_bootfs = bootfs_vmo.borrow();
        let bootfs = BootFS {
        //     vmar: vmar_self.borrow(),
        //     bootfs_vmo,
        //     vmex,
        //     log: duplicate_or_die(log),
        };

        let launch_process = |elf_entry: &ProgramInfo, svc_stash: Option<fx::Channel>| {
            let mut child = create_child_context(elf_entry.filename(), handles);
            let child_message = create_child_message();
            let handle_count = CHILD_HANDLE_COUNT - 1;

            // stash_svc(log, svc_stash_client, elf_entry.filename(), child.svc_server);
            // set_child_handles(log, *borrowed_bootfs, child);

            if svc_stash.is_some() {
                // TODO: set_stash_handle
                // set_stash_handle(log, svc_stash, child.handles);
                // handle_count += 1;
            }

            // Fill in any '+' separated arguments provided by `userboot.next`. If arguments are longer
            // than kProcessArgsMaxBytes, this function will fail process creation.
            // TODO: parse args
            // parse_next_process_arguments(
            //     log,
            //     elf_entry.next,
            //     child_message.header.args_num,
            //     child_message.args.data(),
            // );

            // Map in the bootfs so we can look for files in it.
            let loader_svc = start_child_process(elf_entry, &child_message, &mut child, &bootfs, handle_count);
            log::info!("process {} started.", elf_entry.filename());

            // Now become the loader service for as long as that's needed.
            // TODO:
            //if loader_svc.is_some() {
            // let ldsvc = LoaderService::new(duplicate_or_die(log), &bootfs, elf_entry.root);
            // ldsvc.serve(loader_svc);
            //}

            child
        };

        // if !opts.test.next.empty() {
        //     // If no boot, then hand over the stash to the test program. Test does not get the svc stash.
        //     let test_context = launch_process(opts.test);
        //     // Wait for test to finish.
        //     info.test_return_code = wait_for_process_exit(log, opts.test, test_context);
        //
        //     info.should_shutdown = opts.boot.next.empty();
        // }

        // if !opts.boot.next.empty() {
        //     let boot_context = launch_process(opts.boot, svc_stash_server);
        // }
    }

    handle_termination();
}

// This is the entry point for the whole show, the very first bit of code
// to run in user mode.
pub fn _start(arg1: fx::sys::fx_handle_t, arg2: fx::sys::fx_handle_t) {
    bootstrap(fx::Channel::from_handle(unsafe { fx::Handle::from_raw(arg1) }));
}
