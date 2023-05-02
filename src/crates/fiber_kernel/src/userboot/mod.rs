mod userboot;

use fiber_rust::sys;

use crate::object::{
    get_root_job_dispatcher, get_root_job_handle, start_root_job_observer, ChannelDispatcher, Handle, HandleOwner,
    JobDispatcher, MessagePacket, ProcessDispatcher, TypedDispatcher,
};

fn get_job_handle() -> HandleOwner {
    Handle::dup(get_root_job_handle(), JobDispatcher::default_rights())
}

// KCOUNTER(timeline_userboot, "boot.timeline.userboot")
// KCOUNTER(init_time, "init.userboot.time.msec")

fn userboot_init() {
    // Prepare the bootstrap message packet. This allocates space for its
    // handles, which we'll fill in as we create things.
    let result = MessagePacket::create(std::ptr::null(), 0, userboot::HANDLE_COUNT as u16);
    assert!(result.is_ok());
    let msg = result.unwrap();

    let handles = msg.mutable_handles();
    debug_assert!(msg.num_handles() == userboot::HANDLE_COUNT as u16);

    // Create the process.
    // let vmar_handle:  KernelHandle<VmAddressRegionDispatcher> ;
    let result = ProcessDispatcher::create(get_root_job_dispatcher(), "userboot".to_owned(), 0);
    assert!(result.is_ok());
    let (process_handle, process_rights) = result.unwrap();

    // It needs its own process and root VMAR handles.
    let process = process_handle.dispatcher();
    let proc_handle_owner = Handle::make(process_handle, process_rights);
    // let vmar = vmar_handle.dispatcher();
    // let vmar_handle_owner = Handle::make( vmar_handle, vmar_rights);

    handles[userboot::PROC_SELF] = *proc_handle_owner; // TODO: release
                                                       // handles[userboot::VMAR_ROOT_SELF] = vmar_handle_owner.release();

    // It gets the root job handles.
    handles[userboot::ROOT_JOB] = *get_job_handle(); // TODO: release
    assert!(handles.get(userboot::ROOT_JOB).is_some());

    // TODO: revisit this
    // It also gets many VMOs for VDSOs and other things.
    // constexpr int kVariants = static_cast<int>(userboot::VdsoVariant::COUNT);
    // KernelHandle<VmObjectDispatcher> vdso_kernel_handles[kVariants];
    // const VDso* vdso = VDso::Create(vdso_kernel_handles);
    // for (int i = 0; i < kVariants; ++i) {
    //     handles[userboot::kFirstVdso + i] =
    //         Handle::Make(ktl::move(vdso_kernel_handles[i]), vdso->vmo_rights()).release();
    //     ASSERT(handles[userboot::kFirstVdso + i]);
    // }
    // DEBUG_ASSERT(handles[userboot::kFirstVdso + 1]->dispatcher() == vdso->vmo());
    // if (gBootOptions->always_use_next_vdso) {
    //     std::swap(handles[userboot::kFirstVdso], handles[userboot::kFirstVdso + 1]);
    // }
    // bootstrap_vmos(handles);

    // Make the channel that will hold the message.

    let result = ChannelDispatcher::create();
    assert!(result.is_ok());
    let (user_handle, kernel_handle, channel_rights) = result.unwrap();

    // Transfer it in.
    let status = kernel_handle.dispatcher().write(sys::FX_KOID_INVALID, msg);
    assert!(status == sys::FX_OK);

    // Inject the user-side channel handle into the process.
    let user_handle_owner = Handle::make(user_handle, channel_rights);
    let hv = process.handle_table().map_handle_to_value(&*user_handle_owner);
    process.handle_table().add_handle(user_handle_owner);

    // TODO: do we even need threads?
    // Create the user thread.
    //let thread: Rc<ThreadDispatcher>;
    //{
    //    let thread_handle: KernelHandle<ThreadDispatcher> ;
    //    let thread_rights;
    //    let status = ThreadDispatcher::Create(process, 0, "userboot", &thread_handle, &thread_rights);
    //    assert!(status == ZX_OK);
    //     status = thread_handle.dispatcher().initialize();
    //    assert!(status == ZX_OK);
    //    thread = thread_handle.dispatcher();
    //}
    //assert!(thread);

    // TODO: revisit this
    // Map in the userboot image along with the vDSO.
    // KernelHandle<VmObjectDispatcher> userboot_vmo_kernel_handle;
    // UserbootImage userboot(vdso, &userboot_vmo_kernel_handle);
    // let vdso_base = 0;
    // let entry = 0;
    // let status = userboot.Map(vmar, &vdso_base, &entry);
    // ASSERT(status == ZX_OK);

    // Create a root job observer, restarting the system if the root job becomes childless.
    start_root_job_observer();

    // FIXME: dprintf(SPEW, "userboot: %-23s @ %#" PRIxPTR "\n", "entry point", entry);

    // Start the process's initial thread.
    // let arg1 = hv as usize;
    // status = thread.start(
    //     ThreadDispatcher::EntryState {
    //         entry,
    //         sp,
    //         arg1,
    //         vdso_base,
    //     },
    //     /* ensure_initial_thread= */ true,
    // );
    // assert!(status == sys::FX_OK);

    // TODO: counters
    // timeline_userboot.set(current_ticks());
    // init_time.add(current_time() / 1000000LL);
}
