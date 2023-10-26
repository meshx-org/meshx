import { fx_handle_t } from "@meshx-org/fiber-types"
import { Channel, Handle, Process, Job } from "@meshx-org/fiber-ts"

export const PROC_SELF = 0
export const ROOT_JOB = 1

export const HANDLE_COUNT = 3
export const CHILD_HANDLE_COUNT = HANDLE_COUNT + 5

interface fx_proc_args_t {
    // Protocol and version identifiers to allow for
    // different process start message protocols and
    // versioning of the same.
    protocol: number
    version: number

    // Offset from start of message to handle info
    // array, which contains one u32 per handle
    // passed along with the message.
    handle_info_off: number

    // Offset from start of message to arguments and
    // count of arguments.  Arguments are provided as
    // a set of null-terminated utf-8 strings, one
    // after the other.
    args_off: number
    args_num: number

    // Offset from start of message to environment strings and count of
    // them. Environment entries are provided as a set of null-terminated
    // UTF-8 strings, one after the other.  Canonically each string has
    // the form "NAME=VALUE", but nothing enforces this.
    environ_off: number
    environ_num: number

    // Offset from start of message to name strings and count of them.
    // These strings are packed similar to the argument strings,
    // but are referenced by PA_NS_* handle table entries and used
    // to set up namespaces.
    //
    // Specifically: In a handle table entry with PA_HND_TYPE(info)
    // of PA_NS_*, PA_HND_ARG(info) is an index into this name table.
    names_off: number
    names_num: number
}

// This is the processargs message the child will receive.
class ChildMessageLayout {
    header!: fx_proc_args_t
    args!: string[]
    info!: number[]
    names: string[] = ["/svc"]
}

/*function extract_handles(bootstrap: Channel) {
    // Read the command line and the essential handles from the kernel.
    const handles: fx_handle_t[] = []
    let actual_handles

    let status = bootstrap.read(0, nullptr, handles.data(), 0, handles.size(), nullptr, actual_handles)

    check(log, status, "cannot read bootstrap message")

    if (actual_handles != HANDLE_COUNT) {
        console.error("read %u handles instead of %u", actual_handles, HANDLE_COUNT)
    }

    return handles
}*/

class DebugLog {}

class ChildContext {
    destructor() {
        // fx_handle_close_many(this.handles)
    }

    // Process creation handles
    process!: Process
    svc_client!: Channel
    svc_server!: Channel
    handles: fx_handle_t[] = []
}

/*function create_child_context(log: DebugLog, name: string, handles: fx_handle_t[]): ChildContext {
    const child = new ChildContext()

    let status = Process.create(Job.from_raw(handles[ROOT_JOB]), name, 0, child.process, child.vmar)
    check(log, status, "Failed to create child process(%.*s).", name)

    status = Channel.create(0, child.svc_client, child.svc_server)
    check(log, status, "Failed to create svc channels.")

    // Copy all resources that are not explicitly duplicated in SetChildHandles.
    for (let i = 0; i < handles.length && i < HANDLE_COUNT; ++i) {
        if (i == PROC_SELF) {
            continue
        }
        if (handles[i] == FX_HANDLE_INVALID) {
            continue
        }

        child.handles[i] = RawDuplicateOrDie(log, handles[i])
    }

    return child
}*/

function bootstrap(channel: Channel) {
    // const handles = extract_handles(channel)

    /*const launch_process = (elf_entry, svc_stash): ChildContext => {
        const child_message: ChildMessageLayout = CreateChildMessage()
        const child: ChildContext = create_child_context(log, elf_entry.filename(), handles)
        const handle_count: number = CHILD_HANDLE_COUNT - 1
        // StashSvc(log, svc_stash_client, elf_entry.filename(), child.svc_server)
        SetChildHandles(log, borrowed_bootfs, child)

        if (svc_stash) {
            SetStashHandle(log, svc_stash, child.handles)
            handle_count++
        }

        // Fill in any '+' separated arguments provided by `userboot.next`. If arguments are longer
        // than kProcessArgsMaxBytes, this function will fail process creation.
        ParseNextProcessArguments(log, elf_entry.next, child_message.header.args_num, child_message.args.data())

        // Map in the bootfs so we can look for files in it.
        const loader_svc: Channel = StartChildProcess(log, elf_entry, child_message, child, bootfs, handle_count)
        console.log("process %.*s started.", elf_entry.filename().size() as number, elf_entry.filename().data())

        // Now become the loader service for as long as that's needed.
        if (loader_svc) {
            //let ldsvc: LoaderService = (DuplicateOrDie(log), bootfs, elf_entry.root);
            // ldsvc.Serve(std::move(loader_svc));
        }

        return child
    }*/
}

export function _start(arg1: fx_handle_t) {
    bootstrap(Channel.from_handle(Handle.from_raw(arg1)))
}
