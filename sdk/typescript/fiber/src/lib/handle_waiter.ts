/* eslint-disable @typescript-eslint/no-empty-function */

import { fx_port_wait, fx_object_wait_async, fx_port_create } from "@meshx-org/fiber-sys"
import {
    u64,
    FX_INVALID_HANDLE,
    fx_port_packet_t,
    fx_signals_t,
    FX_TIME_INFINITE,
    Ref,
    fx_handle_t,
    Status,
} from "@meshx-org/fiber-types"
import { Handle } from "./handle"

export type AsyncWaitCallback = (status: number, pending: number) => void

function checked_add_u64(current: u64, increment: u64, expect: string): u64 {
    const newValue = current + increment

    if (newValue == 9_223_372_036_854_775_807n) {
        throw new RangeError(expect)
    }

    return newValue
}

interface PacketReceiver {
    /** Receive a packet when one arrives. */
    receivePacket(packet: fx_port_packet_t): void
}

class PacketReceiverMap<T> {
    private next_key = 0n
    public mapping: Map<u64, T> = new Map<u64, T>()

    public get(key: u64): T | null {
        const value = this.mapping.get(key)
        return value ? value : null
    }

    public insert(val: T): u64 {
        const key = this.next_key
        this.next_key = checked_add_u64(this.next_key, 1n, "ran out of keys")
        this.mapping.set(key, val)
        return key
    }

    public remove(key: u64) {
        this.mapping.delete(key)
    }

    public contains(key: u64): boolean {
        return this.mapping.has(key)
    }
}

interface ReceiverRegistration<T> {
    executor: Executor
    key: u64
    receiver: T
}

class Executor {
    public port: Ref<fx_handle_t> = new Ref(FX_INVALID_HANDLE)
    private receivers: PacketReceiverMap<PacketReceiver>

    constructor() {
        this.receivers = new PacketReceiverMap()

        fx_port_create(this.port)

        const onPacket = (packet: fx_port_packet_t) => {
            packet.key
            packet.packet_type
            packet.union
            packet.status
        }

        if (!this.port.value) throw new Error("failed to init signaling port")
        fx_port_wait(this.port.value, FX_TIME_INFINITE, onPacket)
    }

    public deliver_packet(key: u64, packet: fx_port_packet_t) {
        const receiver = this.receivers.get(key)
        if (!receiver) throw new Error("unknown key")

        receiver.receivePacket(packet)
    }

    /// Registers a `PacketReceiver` with the executor and returns a registration.
    /// The `PacketReceiver` will be deregistered when the `Registration` is dropped.
    public register_receiver<T extends PacketReceiver>(receiver: T): ReceiverRegistration<T> {
        const key = this.receivers.insert(receiver) as u64
        return { executor: this, key, receiver }
    }
}

globalThis.executor = new Executor()

declare global {
    // eslint-disable-next-line no-var
    var executor: Executor
}

export class HandleWaiter implements PacketReceiver {
    private handle: Handle | null = null
    private onRecieve: AsyncWaitCallback

    constructor(handle: Handle, signals: fx_signals_t, callback: AsyncWaitCallback) {
        this.handle = handle

        const registration = globalThis.executor.register_receiver(this)

        const status = fx_object_wait_async(
            this.handle.raw,
            registration.executor.port.value,
            registration.key,
            signals,
            0
        )

        this.onRecieve = callback
    }

    /** @internal */
    receivePacket(packet: fx_port_packet_t): void {
        this.onRecieve(packet.status, 0)
    }

    public cancel(): void {
        if (this.handle) {
            // Cancel the wait.
            // TODO: this.wait.cancel()

            // Release this object from the handle and clear `handle`.
            this.handle.releaseWaiter(this)
            this.handle = null
        }
    }
}
