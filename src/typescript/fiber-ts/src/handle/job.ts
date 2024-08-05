import { fx_job_create, fx_process_create } from "@meshx-org/fiber-sys"
import { fx_handle_t, FX_HANDLE_INVALID, Ref, Status } from "@meshx-org/fiber-types"
import { HandleWrapper } from "./handle-wrapper"
import { Process } from "./process"
import { Handle } from "./handle"

export interface JobCreateResult {
    status: Status
    job?: Job
}

export class Job extends HandleWrapper {
    public static create(parent: Job, name: string): JobCreateResult {
        const job_handle: Ref<fx_handle_t> = new Ref(FX_HANDLE_INVALID)
        const options = 0

        const status = fx_job_create(parent.handle.raw, options, job_handle)

        if (status !== Status.OK) {
            return {
                status,
            }
        }

        return {
            status,
            job: new Job(Handle.from_raw(job_handle.value)),
        }
    }

    /// Create a new job as a child of the current job.
    ///
    /// Wraps the
    /// [fx_job_create](https://fuchsia.dev/fuchsia-src/reference/syscalls/job_create.md)
    /// syscall.
    public createChildJob(): Job {
        const parent_job_raw = this.handle.raw
        const out_ref = new Ref(FX_HANDLE_INVALID)
        const options = 0

        const status = fx_job_create(parent_job_raw, options, out_ref)

        return new Job(Handle.from_raw(out_ref.value))
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
        const parent_job_raw = this.handle.raw

        const options = 0
        const process_out = new Ref(FX_HANDLE_INVALID)
        const vmar_out = new Ref(FX_HANDLE_INVALID)

        const status = fx_process_create(parent_job_raw, name, options, process_out, vmar_out)

        return new Process(process_out.value)
    }
}
