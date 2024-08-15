import {} from "@meshx-org/fiber-sys";
import { FX_HANDLE_INVALID, Ref, Status, fx_status_t } from "@meshx-org/fiber-types";
import { HandleWrapper } from "./handle-wrapper";
import { Handle } from "./handle";
import { Job } from "./job";
import { CreateResult } from "./types";

export class Process extends HandleWrapper {
    constructor(raw: number) {
        super(Handle.from_raw(raw));
    }

    public static create(parent: Job, name: string): CreateResult<Process> {
        const process = new Ref(FX_HANDLE_INVALID);

        const status = self.fiber.sys_process_create(parent.handle?.raw, name, 0, process);

        return {
            status,
            handle: status == Status.OK ? new Process(process.value) : undefined,
        };
    }

    /// Similar to `Thread::start`, but is used to start the first thread in a process.
    ///
    /// Wraps the
    /// [fx_process_start](https://fuchsia.dev/fuchsia-src/reference/syscalls/process_start.md)
    /// syscall.
    public start(entry: string, arg1: Handle): fx_status_t {
        const process_raw = this.handle.raw;
        const arg1_raw = arg1.raw;

        return self.fiber.sys_process_start(process_raw, entry, arg1_raw, 0);
    }
}
