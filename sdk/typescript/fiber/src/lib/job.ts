import { fx_job_create, fx_process_create } from "@meshx-org/fiber-sys"
import {
    fx_handle_t,
    FX_INVALID_HANDLE,
    Ref,
    Status,
} from "@meshx-org/fiber-types"
import { Handle } from "./handle"
import { HandleWrapper } from "./handleWrapper"
import { Process } from "./process"

export class Job extends HandleWrapper {
    public static create(parent: Job, name: string): Job {
        const job_handle: Ref<fx_handle_t> = new Ref(FX_INVALID_HANDLE)
        const options = 0

        const status = fx_job_create(parent.handle!.raw, options, job_handle)

        if (status !== Status.OK) {
            return new Job(Handle.invalid())
        }

        return new Job(new Handle(job_handle.value))
    }

    /// Create a new job as a child of the current job.
    ///
    /// Wraps the
    /// [fx_job_create](https://fuchsia.dev/fuchsia-src/reference/syscalls/job_create.md)
    /// syscall.
    public createChildJob(): Job {
        const parent_job_raw = this.handle!.raw
        const out = new Ref(FX_INVALID_HANDLE)
        const options = 0

        const status = fx_job_create(parent_job_raw, options, out)

        return new Job(new Handle(out.value))
    }

    /// Create a new process as a child of the current job.
    ///
    /// On success, returns a handle to the new process and a handle to the
    /// root of the new process's address space.
    ///
    /// Wraps the
    /// [fx_process_create](https://fuchsia.dev/fuchsia-src/reference/syscalls/process_create.md)
    /// syscall.
    public createChildProcess(name: string): Process {
        const parent_job_raw = this.handle!.raw

        const enc = new TextEncoder()
        const name_size = name.length

        const options = 0
        const process_out = new Ref(FX_INVALID_HANDLE)
        const vmar_out = new Ref(FX_INVALID_HANDLE)

        const status = fx_process_create(
            parent_job_raw,
            enc.encode(name),
            name_size,
            options,
            process_out,
            vmar_out
        )

        return new Process(new Handle(process_out.value))
    }
}
