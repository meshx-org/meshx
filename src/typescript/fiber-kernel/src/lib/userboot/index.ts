import invariant from "tiny-invariant"
import { Handle, HandleOwner } from "../object/handle"
import { FX_KOID_INVALID, FX_OK } from "@meshx-org/fiber-types"
import { ProcessDispatcher } from "../object/process-dispatcher"
import { ChannelDispatcher } from "../object/channel-dispatcher"
import { HANDLE_COUNT, ROOT_JOB, PROC_SELF, _start } from "./userboot"
import { MessagePacket } from "../object/message-packet"
import { Kernel } from ".."
import { JobDispatcher } from "../object/job-dispatcher"

function get_job_handle(kernel: Kernel): HandleOwner {
    return Handle.dup(kernel.get_root_job_handle().handle, JobDispatcher.default_rights())
}

// KCOUNTER(timeline_userboot, "boot.timeline.userboot")
// KCOUNTER(init_time, "init.userboot.time.msec")
export function userboot_init(kernel: Kernel) {
    // Prepare the bootstrap message packet. This allocates space for its
    // handles, which we'll fill in as we create things.
    const msg_result = MessagePacket.create(null, 0, HANDLE_COUNT)
    if (!msg_result.ok) throw new Error("panic")
    const msg = msg_result.value

    invariant(msg.num_handles() === HANDLE_COUNT)

    // Create the process.
    // let vmar_handle:  KernelHandle<VmAddressRegionDispatcher> ;
    const result = ProcessDispatcher.create(kernel.get_root_job_dispatcher(), "userboot", 0)
    if (!result.ok) throw new Error("panic")
    const [process_handle, process_rights] = result.value

    // It needs its own process and root VMAR handles.

    const proc_handle_owner = Handle.make(process_handle, process_rights)
    const process = proc_handle_owner.dispatcher() as ProcessDispatcher

    // let vmar = vmar_handle.dispatcher();
    // let vmar_handle_owner = Handle::make( vmar_handle, vmar_rights);

    const handles = msg.handles()

    handles[PROC_SELF] = proc_handle_owner.handle
    // handles[userboot::VMAR_ROOT_SELF] = vmar_handle_owner.release();

    // It gets the root job handles.
    handles[ROOT_JOB] = get_job_handle(kernel).handle
    invariant(handles[ROOT_JOB] !== null)

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
    const result2 = ChannelDispatcher.create()
    if (!result2.ok) throw new Error("panic")
    const [user_handle, channel_handle, channel_rights] = result2.value

    console.debug(user_handle, channel_handle)

    const channel_dispatcher = channel_handle.dispatcher() as ChannelDispatcher

    // Transfer it in.
    const status1 = channel_dispatcher.write(FX_KOID_INVALID, msg)
    invariant(status1 === FX_OK)

    // Inject the user-side channel handle into the process.
    const user_handle_owner = Handle.make(user_handle, channel_rights)
    const hv = process.handle_table().map_handle_to_value(user_handle_owner)
    process.handle_table().add_handle(user_handle_owner)

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
    kernel.start_root_job_observer()

    console.info(`userboot: entrypoint=${_start.name}`)

    // Start the process.
    const arg1 = hv
    const status2 = process.start(_start, arg1, 0)
    invariant(status2 === FX_OK)

    // TODO: counters
    // timeline_userboot.set(current_ticks());
    // init_time.add(current_time() / 1000000LL);
}
