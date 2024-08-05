import {
    FX_CHANNEL_READABLE,
    FX_ERR_BAD_HANDLE,
    FX_ERR_BAD_STATE,
    FX_ERR_BUFFER_TOO_SMALL,
    FX_ERR_PEER_CLOSED,
    FX_ERR_SHOULD_WAIT,
    FX_ERR_TIMED_OUT,
    FX_KOID_INVALID,
    FX_OBJ_TYPE_CHANNEL,
    FX_OK,
    fx_koid_t,
    fx_obj_type_t,
    fx_rights_t,
    fx_signals_t,
    fx_status_t,
    fx_txid_t,
    u32,
} from "@meshx-org/fiber-types";
import { Ok, Result, Ref, Err, debug_invariant, Deque } from "../../std";
import { PeerHolder, PeeredDispatcher } from "./dispatcher";
import { KernelHandle } from "../handle";
import { MessagePacket, MessagePacketPtr } from "../message-packet";
import { DoublyLinkedListNode, DoublyLinkedList } from "@datastructures-js/linked-list";

// This value is part of the zx_channel_call contract.
const MIN_KERNEL_GENERATED_TXID = 0x80000000;

function is_kernel_generated_txid(txid: fx_txid_t): boolean {
    return txid >= MIN_KERNEL_GENERATED_TXID;
}

// MessageWaiter's state is guarded by the lock of the
// owning ChannelDispatcher, and Deliver(), Signal(), Cancel(),
// and EndWait() methods must only be called under
// that lock.
//
// MessageWaiters are embedded in ThreadDispatchers, and the channel_ pointer
// can only be manipulated by their thread (via BeginWait() or EndWait()), and
// only transitions to nullptr while holding the ChannelDispatcher's lock.
//
// See also: comments in ChannelDispatcher::Call()
class MessageWaiter extends DoublyLinkedListNode {
    #txid: fx_txid_t;
    #status: fx_status_t;
    #channel: ChannelDispatcher | null = null;
    #msg!: MessagePacket;
    #event: any;

    constructor(value?: any, prev?: DoublyLinkedListNode, next?: DoublyLinkedListNode) {
        super(value, prev, next);
        this.#txid = 0;
        this.#status = FX_ERR_BAD_STATE;
    }

    beginWait(channel: ChannelDispatcher): fx_status_t {
        if (this.#channel) return FX_ERR_BAD_STATE;

        //debug_invariant(!this.inContainer())

        this.#status = FX_ERR_TIMED_OUT;
        this.#channel = channel;
        this.#event.unsignal();
        return FX_OK;
    }

    deliver(msg: MessagePacket): void {
        debug_invariant(this.#channel);

        this.#msg = msg;
        this.#status = FX_OK;
        this.#event.signal(FX_OK);
    }

    cancel(status: fx_status_t): void {
        //DEBUG_ASSERT(!InContainer());
        debug_invariant(this.#channel);
        this.#status = status;
        this.#event.Signal(status);
    }

    get_channel(): ChannelDispatcher | null {
        return this.#channel;
    }

    get_txid(): fx_txid_t {
        return this.#txid;
    }

    set_txid(txid: fx_txid_t): void {
        this.#txid = txid;
    }

    wait(deadline: any): fx_status_t {
        if (!this.#channel) return FX_ERR_BAD_STATE;

        return this.#event.wait(deadline);
    }

    // Returns any delivered message via out and the status.
    endWait(out: Ref<MessagePacket>): fx_status_t {
        if (!this.#channel) return FX_ERR_BAD_STATE;

        out.value = this.#msg;
        this.#channel = null;
        return this.#status;
    }
}

type GuardedState = {
    waiters: DoublyLinkedList<MessageWaiter>;
    messages: Deque<MessagePacket>;
    max_message_count: number;
    txid: fx_txid_t;
    peer_has_closed: boolean;
};

export class ChannelDispatcher extends PeeredDispatcher<ChannelDispatcher> {
    #owner: fx_koid_t;
    #guarded: GuardedState;

    private constructor(peer: PeerHolder<ChannelDispatcher>) {
        super(peer);

        // this.peered_base = PeeredDispatcherBase::new(peer)
        this.#owner = FX_KOID_INVALID;
        this.#guarded = {
            waiters: new DoublyLinkedList(),
            messages: new Deque(),
            max_message_count: 0,
            txid: 0,
            peer_has_closed: false,
        };
    }

    override set_owner(new_owner: fx_koid_t): void {
        // Testing for ZX_KOID_INVALID is an optimization so we don't
        // pay the cost of grabbing the lock when the endpoint moves
        // from the process to channel; the one that we must get right
        // is from channel to new owner.
        if (new_owner == FX_KOID_INVALID) {
            return;
        }

        this.#owner = new_owner;
    }

    public static create(): Result<
        [KernelHandle<ChannelDispatcher>, KernelHandle<ChannelDispatcher>, fx_rights_t],
        fx_status_t
    > {
        const holder0 = new PeerHolder();
        const holder1 = holder0;

        const new_kernel_handle0 = new KernelHandle(new ChannelDispatcher(holder0));
        const new_kernel_handle1 = new KernelHandle(new ChannelDispatcher(holder1));

        const new_handle0 = new_kernel_handle0.dispatcher() as ChannelDispatcher;
        const new_handle1 = new_kernel_handle1.dispatcher() as ChannelDispatcher;

        new_handle0.init_peer(new_handle1);
        new_handle1.init_peer(new_handle0);

        const rights = ChannelDispatcher.default_rights();
        //let handle0 = new_handle0;
        //let handle1 = new_handle1;

        return Ok([new_kernel_handle0, new_kernel_handle1, rights]);
    }

    /// Write to the opposing endpoint's message queue. |owner| is the handle table koid of the process
    /// attempting to write to the channel, or FX_KOID_INVALID if kernel is doing it.
    public write(owner: fx_koid_t, msg: MessagePacketPtr): fx_status_t {
        // canary_.Assert();
        // Guard<CriticalMutex> guard{get_lock()};

        // Failing this test is only possible if this process has two threads racing:
        // one thread is issuing channel_write() and one thread is moving the handle
        // to another process.
        if (owner != this.#owner) {
            return FX_ERR_BAD_HANDLE;
        }

        if (!this.peer) {
            return FX_ERR_PEER_CLOSED;
        }

        const peer = this.peer!;

        if (peer.try_write_to_message_waiter(msg)) {
            return FX_OK;
        }

        peer.write_self(msg);
        console.log("wrote to peer", peer, peer.get_koid());

        return FX_OK;
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
        let previous_signals: fx_signals_t;

        const guard = this.#guarded;

        guard.messages.push_back(msg);
        // eslint-disable-next-line prefer-const
        previous_signals = this.raise_signals_locked(FX_CHANNEL_READABLE);

        const size = guard.messages.length;
        if (size > guard.max_message_count) {
            guard.max_message_count = size;
        }

        // Don't bother waking observers if FX_CHANNEL_READABLE was already active.
        if ((previous_signals & FX_CHANNEL_READABLE) === 0) {
            this.notify_observers_locked(previous_signals | FX_CHANNEL_READABLE);
        }
    }

    private raise_signals_locked(signals: fx_signals_t): fx_signals_t {
        // TODO: remove this
        return FX_CHANNEL_READABLE;
    }

    // eslint-disable-next-line @typescript-eslint/no-empty-function
    private notify_observers_locked(signals: fx_signals_t) {}

    private try_write_to_message_waiter(msg: MessagePacketPtr): boolean {
        const guard = this.#guarded;

        if (guard.waiters.isEmpty()) {
            return false;
        }

        // If the far side has "call" waiters waiting for replies, see if this message's txid matches one
        // of them.  If so, deliver it.  Note, because callers use a kernel generated txid we can skip
        // checking the list if this message's txid isn't kernel generated.
        const txid = msg.get_txid();
        if (!is_kernel_generated_txid(txid)) {
            return false;
        }

        let result = false;

        guard.waiters.forEach((waiter) => {
            // (3C) Deliver message to waiter.
            // Remove waiter from list.
            if (waiter.get_txid() == txid) {
                // TODO: self.waiters.erase(waiter);
                waiter.deliver(msg);
                result = true;
            }
        });

        return result;
    }

    // Read from this endpoint's message queue.
    // |owner| is the handle table koid of the process attempting to read from the channel.
    // |msg_size| and |msg_handle_count| are in-out parameters. As input, they specify the maximum
    // size and handle count, respectively. On ZX_OK or ZX_ERR_BUFFER_TOO_SMALL, they specify the
    // actual size and handle count of the next message. The next message is returned in |*msg| on
    // ZX_OK and also on ZX_ERR_BUFFER_TOO_SMALL when |may_discard| is set.
    read(
        owner: fx_koid_t,
        msg_size: Ref<u32>,
        msg_handle_count: Ref<u32>,
        msg: Ref<MessagePacket | null>,
        may_discard: boolean
    ): fx_status_t {
        //canary_.Assert();

        const max_size = msg_size.value;
        const max_handle_count = msg_handle_count.value;

        const guard = this.#guarded;

        if (owner !== this.#owner) {
            console.log("err, FX_ERR_BAD_HANDLE");
            return FX_ERR_BAD_HANDLE;
        }

        if (guard.messages.is_empty()) {
            console.log("err", "FX_ERR_SHOULD_WAIT", this.get_koid(), this.peer?.get_koid());
            return guard.peer_has_closed ? FX_ERR_PEER_CLOSED : FX_ERR_SHOULD_WAIT;
        }

        msg_size.value = guard.messages.front!.data_size();
        msg_handle_count.value = guard.messages.front!.num_handles();

        let status: fx_status_t = FX_OK;
        if (msg_size.value > max_size || msg_handle_count.value > max_handle_count) {
            if (!may_discard) {
                console.log("err", "FX_ERR_BUFFER_TOO_SMALL");
                return FX_ERR_BUFFER_TOO_SMALL;
            }

            status = FX_ERR_BUFFER_TOO_SMALL;
        }

        console.log(guard.messages)

        msg.value = guard.messages.pop_front();
        if (guard.messages.is_empty()) {
            this.clear_signals(FX_CHANNEL_READABLE);
        }

        console.log("dispatcher read", msg, msg_size, msg_handle_count);

        return status;
    }

    override get_type(): fx_obj_type_t {
        return FX_OBJ_TYPE_CHANNEL;
    }
}
