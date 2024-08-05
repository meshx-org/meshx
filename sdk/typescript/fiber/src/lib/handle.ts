import { fx_handle_t, fx_signals_t, FX_HANDLE_INVALID } from "@meshx-org/fiber-types"
import { fx_handle_close, fx_handle_duplicate } from "@meshx-org/fiber-sys"
import { AsyncWaitCallback, HandleWaiter } from "./handle_waiter"

export class Handle {
    private $handle: fx_handle_t = FX_HANDLE_INVALID
    private waiters: HandleWaiter[] = []

    constructor(handle: fx_handle_t) {
        this.$handle = handle
    }

    public static invalid(): Handle {
        return new Handle(FX_HANDLE_INVALID)
    }

    public get raw(): fx_handle_t {
        return this.$handle
    }

    public get isValid(): boolean {
        return this.$handle !== FX_HANDLE_INVALID
    }

    public close(): void {
        const status = fx_handle_close(this.$handle)
    }

    public async duplicate(): Promise<Handle> {
        const { status, handle: raw } = fx_handle_duplicate(this.$handle)

        return new Handle(raw!)
    }

    // TODO: Implement
    public async replace(): Promise<Handle> {
        throw new Error("Not implemented")
    }

    public asyncWait(signals: fx_signals_t, callback: AsyncWaitCallback): HandleWaiter {
        const waiter = new HandleWaiter(this, signals, callback)
        this.waiters.push(waiter)
        return waiter
    }

    public releaseWaiter(waiter: HandleWaiter) {
        this.waiters = this.waiters.filter((current) => current !== waiter)
    }
}
