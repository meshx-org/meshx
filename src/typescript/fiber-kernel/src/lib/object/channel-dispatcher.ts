import {
    FX_CHANNEL_READABLE,
    FX_ERR_BAD_HANDLE,
    FX_ERR_PEER_CLOSED,
    FX_KOID_INVALID,
    FX_OK,
    fx_koid_t,
    fx_rights_t,
    fx_signals_t,
    fx_status_t,
    fx_txid_t,
} from "@meshx-org/fiber-types"
import { Ok, Result, Ref, Err } from "../std"
import { PeerHolder, PeeredDispatcher } from "./dispatcher"
import { KernelHandle } from "./handle"
import { MessagePacketPtr } from "./message-packet"

// This value is part of the zx_channel_call contract.
const MIN_KERNEL_GENERATED_TXID = 0x80000000

function is_kernel_generated_txid(txid: fx_txid_t): boolean {
    return txid >= MIN_KERNEL_GENERATED_TXID
}

type GuardedState = {
    waiters: any[]
    messages: any[]
    max_message_count: number
    txid: fx_txid_t
    peer_has_closed: boolean
}

export class ChannelDispatcher extends PeeredDispatcher<ChannelDispatcher> {
    private _owner: fx_koid_t
    private _guarded: GuardedState

    private constructor(peer: PeerHolder<ChannelDispatcher>) {
        super(peer)

        // this.peered_base = PeeredDispatcherBase::new(peer)
        this._owner = FX_KOID_INVALID
        this._guarded = {
            waiters: [],
            messages: [],
            max_message_count: 0,
            txid: 0,
            peer_has_closed: false,
        }
    }

    public static create(): Result<
        [KernelHandle<ChannelDispatcher>, KernelHandle<ChannelDispatcher>, fx_rights_t],
        fx_status_t
    > {
        const holder0 = new PeerHolder()
        const holder1 = holder0

        const new_kernel_handle0 = new KernelHandle(new ChannelDispatcher(holder0))
        const new_kernel_handle1 = new KernelHandle(new ChannelDispatcher(holder1))

        const new_handle0 = new_kernel_handle0.dispatcher() as ChannelDispatcher
        const new_handle1 = new_kernel_handle1.dispatcher() as ChannelDispatcher

        new_handle0.init_peer(new Ref(new_handle1))
        new_handle1.init_peer(new Ref(new_handle0))

        const rights = ChannelDispatcher.default_rights()
        //let handle0 = new_handle0;
        //let handle1 = new_handle1;

        return Ok([new_kernel_handle0, new_kernel_handle1, rights])
    }

    /// Write to the opposing endpoint's message queue. |owner| is the handle table koid of the process
    /// attempting to write to the channel, or FX_KOID_INVALID if kernel is doing it.
    public write(owner: fx_koid_t, msg: MessagePacketPtr): fx_status_t {
        // canary_.Assert();
        // Guard<CriticalMutex> guard{get_lock()};

        // Failing this test is only possible if this process has two threads racing:
        // one thread is issuing channel_write() and one thread is moving the handle
        // to another process.
        if (owner != this._owner) {
            return FX_ERR_BAD_HANDLE
        }

        if (!this.peer()) {
            return FX_ERR_PEER_CLOSED
        }

        const peer = this.peer()!.value

        if (peer.try_write_to_message_waiter(msg)) {
            return FX_OK
        }

        peer.write_self(msg)

        return FX_OK
    }

    private write_self(msg: MessagePacketPtr) {
        //canary_.Assert();

        // Once we've acquired the channel_lock_ we're going to make a copy of the previously active
        // signals and raise the READABLE signal before dropping the lock.  After we've dropped the lock,
        // we'll notify observers using the previously active signals plus READABLE.
        //
        // There are several things to note about this sequence:
        //
        // 1. We must hold channel_lock_ while updating the stored signals (RaiseSignalsLocked) to
        // synchronize with thread adding, removing, or canceling observers otherwise we may create a
        // spurious READABLE signal (see NoSpuriousReadableSignalWhenRacing test).
        //
        // 2. We must release the channel_lock_ before notifying observers to ensure that Read can execute
        // concurrently with NotifyObserversLocked, which is a potentially long running call.
        //
        // 3. We can skip the call to NotifyObserversLocked if the previously active signals contained
        // READABLE (because there can't be any observers still waiting for READABLE if that signal is
        // already active).
        let previous_signals: fx_signals_t

        const lock = this._guarded

        lock.messages.push(msg)
        // eslint-disable-next-line prefer-const
        previous_signals = this.raise_signals_locked(FX_CHANNEL_READABLE)

        const size = lock.messages.length
        if (size > lock.max_message_count) {
            lock.max_message_count = size
        }

        // Don't bother waking observers if FX_CHANNEL_READABLE was already active.
        if ((previous_signals & FX_CHANNEL_READABLE) === 0) {
            this.notify_observers_locked(previous_signals | FX_CHANNEL_READABLE)
        }
    }

    private raise_signals_locked(signals: fx_signals_t): fx_signals_t {
        // TODO: remove this
        return FX_CHANNEL_READABLE
    }

    // eslint-disable-next-line @typescript-eslint/no-empty-function
    private notify_observers_locked(signals: fx_signals_t) {}

    private try_write_to_message_waiter(msg: MessagePacketPtr): Result<null, MessagePacketPtr> {
        const lock = this._guarded

        if (lock.waiters.length === 0) {
            return Err(msg)
        }

        // If the far side has "call" waiters waiting for replies, see if this message's txid matches one
        // of them.  If so, deliver it.  Note, because callers use a kernel generated txid we can skip
        // checking the list if this message's txid isn't kernel generated.
        const txid = msg.get_txid()
        if (!is_kernel_generated_txid(txid)) {
            return Err(msg)
        }

        for (const waiter of lock.waiters) {
            // (3C) Deliver message to waiter.
            // Remove waiter from list.
            if (waiter.get_txid() == txid) {
                // TODO: self.waiters.erase(waiter);
                waiter.deliver(msg)
                return Ok(null)
            }
        }

        return Err(msg)
    }
}
