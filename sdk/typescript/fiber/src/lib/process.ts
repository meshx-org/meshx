import { fx_process_create, fx_process_start } from '@meshx-org/fiber-sys'
import { fx_status_t } from '@meshx-org/fiber-types'
import { HandleWrapper } from './handleWrapper'
import { Handle } from './handle'

export class Process extends HandleWrapper {
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    public static create(): void {}

    /// Similar to `Thread::start`, but is used to start the first thread in a process.
    ///
    /// Wraps the
    /// [zx_process_start](https://fuchsia.dev/fuchsia-src/reference/syscalls/process_start.md)
    /// syscall.
    public start(entry: number, arg1: Handle): fx_status_t {
        const process_raw = this.handle!.raw
        const arg1_raw = arg1.raw

        const status = fx_process_start(process_raw, entry, arg1_raw)

        return 0
    }
}
