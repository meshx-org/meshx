import {
    fx_handle_t,
    FX_OK,
    fx_status_t,
    fx_proc_args,
    FX_HANDLE_INVALID,
    FX_RIGHT_SAME_RIGHTS,
    Ref,
    FX_PROCARGS_PROTOCOL,
    FX_PROCARGS_VERSION,
    PA_PROC_SELF,
    PA_JOB_DEFAULT,
    u32,
} from "@meshx-org/fiber-types";
import * as fx from "@meshx-org/fiber-ts";
import { processes } from "../object/kernel-processes";
import { fx_handle_close } from "@meshx-org/fiber-sys";
import { HandleWrapper } from "src/typescript/fiber-ts/src/handle/handle-wrapper";

export const PROC_SELF = 0;
export const ROOT_JOB = 1;

export const HANDLE_COUNT = 2;
export const CHILD_HANDLE_COUNT = HANDLE_COUNT + 5;

const SVC_NAME_INDEX = 0;

// Max number of bytes allowed for arguments to the userboot.next binary. This is an arbitrary
// value.
const PROCESS_ARGS_MAX_BYTES = 128;

function check(status: fx_status_t, message: string) {
    if (status !== FX_OK) throw new Error(message + ", status=" + status.toString());
}

function extract_handles(bootstrap: fx.Channel) {
    // Read the command line and the essential handles from the kernel.
    const { status, handles } = bootstrap.read(0, 0, HANDLE_COUNT);

    check(status, "cannot read bootstrap message");

    if (handles.length !== HANDLE_COUNT) {
        console.error(`read ${handles.length} handles instead of ${HANDLE_COUNT}`);
    }

    return handles.map((handle) => handle.raw);
}

function handle_info_table(): fx_handle_t[] {
    const info: fx_handle_t[] = [];
    info[PROC_SELF] = PA_HND(PA_PROC_SELF, 0);
    info[ROOT_JOB] = PA_HND(PA_JOB_DEFAULT, 0);
    return info;
}

// Handle Info entries associate a type and optional
// argument with each handle included in the process
// arguments message.
function PA_HND(type: number, arg: number) {
    return (type & 0xff) | ((arg & 0xffff) << 16);
}

// This is the processargs message the child will receive.
class ChildMessageLayout {
    handle_info: fx_handle_t[];
    argv: Ref<string[]>;
    environ = "";
    names: string[] = ["/svc"];

    constructor(handle_info: fx_handle_t[]) {
        this.handle_info = handle_info;
        this.argv = new Ref([]);
    }

    serialize() {
        const argv = this.serialize_argv();

        const data = new Uint8Array(36 + this.handle_info.length * 4 + argv.byteLength);
        const writer = new DataView(data.buffer);

        const handle_info_off = 36;
        const argv_off = handle_info_off + this.handle_info.length * 4;
        const environ_off = argv_off + argv.byteLength;
        const names_off = environ_off;

        writer.setInt32(0, FX_PROCARGS_PROTOCOL, true);
        writer.setInt32(4, FX_PROCARGS_VERSION, true);
        writer.setInt32(8, handle_info_off, true); // handle_info_off
        writer.setInt32(12, argv_off, true); // args_off
        writer.setInt32(16, this.argv.value.length, true); // args_num
        writer.setInt32(20, environ_off, true); // environ_off,
        writer.setInt32(24, 0, true); // environ_num
        writer.setInt32(28, names_off, true); // names_off
        writer.setInt32(32, SVC_NAME_INDEX + 1, true); // names_num

        for (let i = 0; i < this.handle_info.length; i++) {
            const info = this.handle_info[i];
            writer.setInt32(handle_info_off + i * 4, info, true);
        }

        data.set(argv, argv_off);

        return data;
    }

    private serialize_argv(): Uint8Array {
        const encoder = new TextEncoder();

        const merged_argv = this.argv.value.join("\0") + "\0";
        const bytes = encoder.encode(merged_argv);

        return bytes;
    }
}

class ChildContext {
    destroy(): void {
        for (const handle of this.handles) {
            fx_handle_close(handle);
        }
    }

    // Process creation handles
    process!: fx.Process;
    svc_client!: fx.Channel;
    svc_server!: fx.Channel;

    handles: fx_handle_t[] = Array(HANDLE_COUNT);
}

function create_child_message(): ChildMessageLayout {
    return new ChildMessageLayout(handle_info_table());
}

function duplicate_or_die<T extends HandleWrapper>(typed_handle: T) {
    const { status, handle: dup } = typed_handle.duplicate(FX_RIGHT_SAME_RIGHTS);
    check(status, "Failed to duplicate handle.");
    return dup;
}

function raw_duplicate_or_die(handle: fx_handle_t): fx_handle_t {
    const dup = new Ref(FX_HANDLE_INVALID);
    const status = self.fiber.sys_handle_duplicate(handle, FX_RIGHT_SAME_RIGHTS, dup);
    check(status, "Failed to duplicate handle.");
    return dup.value;
}

function create_child_context(name: string, handles: fx_handle_t[]): ChildContext {
    const child = new ChildContext();

    const root_job = new fx.Job(fx.Handle.from_raw(handles[ROOT_JOB]));

    const process = fx.Process.create(root_job, name);
    check(process.status, `Failed to create child process(${name}).`);
    child.process = process.handle!;

    const pair = fx.ChannelPair.create();
    check(pair.status, "Failed to create svc channels.");
    child.svc_client = pair.handle!.first;
    child.svc_server = pair.handle!.second;

    // Copy all resources that are not explicitly duplicated in SetChildHandles.
    for (let i = 0; i < handles.length && i < HANDLE_COUNT; ++i) {
        if (i == PROC_SELF) {
            continue;
        }

        if (handles[i] == FX_HANDLE_INVALID) {
            continue;
        }

        child.handles[i] = raw_duplicate_or_die(handles[i]);
    }

    return child;
}

function set_child_handles(child: ChildContext) {
    //child.handles[kBootfsVmo] = DuplicateOrDie(bootfs_vmo).release();
    //child.handles[kDebugLog] = DuplicateOrDie().release();
    child.handles[PROC_SELF] = duplicate_or_die(child.process).raw;
    //child.handles[kVmarRootSelf] = DuplicateOrDie(child.vmar).release();
    //child.handles[kThreadSelf] = DuplicateOrDie(child.thread).release();
    //child.handles[kSvcStub] = child.svc_client.release();
}

interface ProgramInfo {
    // `prefix.root`: the BOOTFS directory under which userboot will find its
    // child program and the libraries accessible to its loader service
    root: string;

    entry: string;

    // `prefix.next`: The root-relative child program path, with optional '+' separated
    // arguments to pass to the child program.
    next: string;
}

function filename(info: ProgramInfo) {
    return info.next.substring(0, info.next.indexOf("+"));
}

function parse_next_process_arguments(next: string, argv: { value: string[] }): void {
    // Extra byte for null terminator (not needed in JavaScript/TypeScript but kept for conceptual parity).
    const requiredSize = next.length + 1;
    if (requiredSize > PROCESS_ARGS_MAX_BYTES) {
        console.error(
            `required ${requiredSize} bytes for process arguments, but only ${PROCESS_ARGS_MAX_BYTES} are available`
        );
        return;
    }

    const args = next.split("+");
    argv.value = args;

    // At a minimum, child processes will be passed a single argument containing the binary name.
    //argc.value++

    // Split the `next` string by '+' and populate the `argv` array.
    //const args = next.split("+")
    ////argv.push(...args)
    //argc.value += args.length - 1 // Increment argc by the number of new arguments.
}

async function start_child_process(program_info: ProgramInfo, child_message: ChildMessageLayout, child: ChildContext) {
    const stack_size = 0;

    // eslint-disable-next-line prefer-const
    let { status, handle } = fx.ChannelPair.create();
    check(status, "fx_channel_create failed for child stack");
    const { first: to_child, second: bootstrap } = handle!;

    let loader_svc: fx.Channel;

    // Now load the vDSO into the child, so it has access to system calls.
    //const vdso_base = elf_load_vdso(log, child.vmar, zx.unowned_vmo(child.handles[kFirstVdso]))

    //let stack_vmo: Vmo
    //status = Vmo.create(stack_size, 0, stack_vmo)
    //check(status, "zx_vmo_create failed for child stack")
    //stack_vmo.set_property(ZX_PROP_NAME, kStackVmoName, sizeof(kStackVmoName) - 1)

    //let stack_base
    //status = child.vmar.map(FX_VM_PERM_READ | FX_VM_PERM_WRITE, 0, stack_vmo, 0, stack_size, stack_base)
    //check(status, "zx_vmar_map failed for child stack")

    // Allocate the stack for the child.
    //const sp = 0
    //console.log(log, "stack [%p, %p) sp=%p", stack_base, stack_base + stack_size, sp)

    // We're done doing mappings, so clear out the reservation VMAR.
    //check(child.reserved_vmar.destroy(), "zx_vmar_destroy failed on reservation VMAR handle")
    //child.reserved_vmar.reset()

    console.log("child_message", child_message);

    // Now send the bootstrap message.  This transfers away all the handles
    // we have left except the process and thread themselves.
    status = to_child.write(0, child_message.serialize(), child.handles);
    check(status, "fx_channel_write to child failed");

    // Start the process going.
    status = child.process.start(program_info.entry, bootstrap.handle);
    check(status, "fx_process_start failed");
    //child.thread.reset()
}

async function bootstrap(channel: fx.Channel): Promise<u32> {
    // We pass all the same handles the kernel gives us along to the child,
    // except replacing our own process/root-VMAR handles with its, and
    // passing along the three extra handles (BOOTFS, thread-self, and a debuglog
    // handle tied to stdout).
    const handles = extract_handles(channel);

    // These channels will speak `fuchsia.boot.Userboot` protocol.

    const { status, handle } = fx.ChannelPair.create();
    check(status, "Failed to create fuchsia.boot.Userboot channel.");
    const { first: userboot_server, second: userboot_client } = handle!;

    // Parse CMDLINE items to determine the set of runtime options.
    // const opts: Options = get_options_from_fxbi(vmar_self, fxbi)

    const info: TerminationInfo = {
        should_shutdown: false,
    };

    const launch_process = (entry: ProgramInfo): ChildContext => {
        console.log(handles);
        const child_message: ChildMessageLayout = create_child_message();
        const child: ChildContext = create_child_context(filename(entry), handles);
        const handle_count: number = CHILD_HANDLE_COUNT - 1;
        // stash_svc(log, svc_stash_client, elf_entry.filename(), child.svc_server)
        set_child_handles(child);

        //if (svc_stash) {
        //    SetStashHandle(log, svc_stash, child.handles)
        //    handle_count++
        //}

        // Fill in any '+' separated arguments provided by `userboot.next`. If arguments are longer
        // than kProcessArgsMaxBytes, this function will fail process creation.
        parse_next_process_arguments(entry.next, child_message.argv);

        // Map in the bootfs so we can look for files in it.
        start_child_process(entry, child_message, child);
        console.log(`process ${filename(entry)} started.`);

        // Now become the loader service for as long as that's needed.
        //if (loader_svc) {
        //let ldsvc: LoaderService = (DuplicateOrDie(log), bootfs, elf_entry.root);
        // ldsvc.Serve(std::move(loader_svc));
        //}

        return child;
    };

    const boot_context = launch_process({
        root: "/",
        entry: "_fiber_start",
        next: "component_manager+--boot",
    });

    handle_termination(info);

    return 0;
}

type TerminationInfo = {
    // Depending on test mode and result, this might be the return code of boot or test elf.
    // std::optional<int64_t> test_return_code;

    // Whether we should continue or shutdown.
    should_shutdown: boolean;
};

function handle_termination(info: TerminationInfo): void {
    if (!info.should_shutdown) {
        console.log("finished!");
        self.fiber.sys_process_exit(0n);
    }

    //console.log("Process exited.  Executing poweroff")
    //fx_system_powerctl(info.power.get(), FX_SYSTEM_POWERCTL_SHUTDOWN, null)
    //console.log("still here after poweroff!")
}

processes.set("userboot", bootstrap);
