import { fx_process_start, fx_process_create } from "@meshx-org/fiber-sys"
import { FX_INVALID_HANDLE, Ref, fx_status_t } from "@meshx-org/fiber-types"
import { HandleWrapper } from "./handle-wrapper"
import { Handle } from "./handle"
import { Job } from "./job"

export class Process extends HandleWrapper {
      constructor(raw: number) {
        super(raw)
    }

    public static create(parent: Job, name: string) {
        const process = new Ref(FX_INVALID_HANDLE)
        const vmar = new Ref(FX_INVALID_HANDLE)

        const status = fx_process_create(parent.raw, name, 0, process, vmar)

        return new Process(process.value)
    }

    /// Similar to `Thread::start`, but is used to start the first thread in a process.
    ///
    /// Wraps the
    /// [zx_process_start](https://fuchsia.dev/fuchsia-src/reference/syscalls/process_start.md)
    /// syscall.
    public start(entry: bigint, arg1: Handle): fx_status_t {
        const process_raw = this.raw
        const arg1_raw = arg1.raw

        return fx_process_start(process_raw, entry, arg1_raw)
    }
}
