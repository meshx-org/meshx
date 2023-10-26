import {
    FX_ERR_WRONG_TYPE,
    FX_KOID_INVALID,
    fx_koid_t,
    fx_rights_t,
    fx_signals_t,
    fx_status_t,
} from "@meshx-org/fiber-types"
import { Err, Ref, Result } from "../../std"
import invariant from "tiny-invariant"
import * as koid from "../../koid"

export abstract class Dispatcher {
    private _koid: fx_koid_t
    private _handle_count: number

    // |_signals| is the set of currently active signals.
    //
    // There are several high-level operations in which the signal state is accessed.  Some of these
    // operations require holding |get_lock()| and some do not.  See the comment at |get_lock()|.
    //
    // 1. Adding, removing, or canceling an observer - These operations involve access to both
    // signals_ and observers_ and must be performed while holding get_lock().
    //
    // 2. Updating signal state - This is a composite operation consisting of two sub-operations:
    //
    //    a. Clearing signals - Because no observer may be triggered by deasserting (clearing) a
    //    signal, it is not necessary to hold |get_lock()| while clearing.  Simply clearing signals
    //    does not need to access observers_.
    //
    //    b. Raising (setting) signals and notifying matched observers - This operation must appear
    //    atomic to and cannot overlap with any of the operations in #1 above.  |get_lock()| must be
    //    held for the duration of this operation.
    //
    // Regardless of whether the operation requires holding |get_lock()| or not, access to this field
    // should use acquire/release memory ordering.  That is, use memory_order_acquire for read,
    // memory_order_release for write, and memory_order_acq_rel for read-modify-write.  To understand
    // why it's important to use acquire/release, consider the following (contrived) example:
    //
    //   RelaxedAtomic<bool> ready;
    //
    //   void T1() {
    //     // Wait for T2 to clear the signals.
    //     while (d.PollSignals() & kMask) {
    //     }
    //     // Now that we've seen there are no signals we can be confident that ready is true.
    //     ASSERT(ready.load());
    //   }
    //
    //   void T2() {
    //     ready.store(true);
    //     d.ClearSignals(kMask);
    //   }
    //
    // In the example above, T1's ASSERT may fire if PollSignals or ClearSignals were to use relaxed
    // memory order for accessing _signals.
    private _signals: fx_signals_t

    // At construction, the object is asserting |signals|.
    protected constructor(signals: fx_signals_t) {
        this._signals = signals
        this._handle_count = 0
        this._koid = koid.generate()

        // kcounter_add(dispatcher_create_count, 1);
    }

    static default_rights(): fx_rights_t {
        return 0
    }

    get_koid() {
        return this._koid
    }

    increment_handle_count(): void {
        // As this function does not return anything actionable, not even something implicit like "you
        // now have the lock", there are no correct assumptions the caller can make about orderings
        // of this increment and any other memory access. As such it can just be relaxed.
        this._handle_count += 1
    }

    // Returns true exactly when the handle count goes to zero.
    decrement_handle_count(): boolean {
        if (--this._handle_count === 1) {
            // The decrement operation above synchronizes with the fence below.  This ensures that changes
            // to the object prior to its handle count reaching 0 will be visible to the thread that
            // ultimately drops the count to 0.  This is similar to what's done in
            // |fbl::RefCountedInternal|.

            return true
        }

        return false
    }

    current_handle_count(): number {
        // Requesting the count is fundamentally racy with other users of the dispatcher. A typical
        // reference count implementation might place an acquire here for the scenario where you then
        // run an object destructor without acquiring any locks. As a handle count is not a refcount
        // and a low handle count does not imply any ownership of the dispatcher (which has its own
        // refcount), this can just be relaxed.
        return this._handle_count
    }

    // get_name() will return a null-terminated name of ZX_MAX_NAME_LEN - 1 or fewer
    // characters in |out_name|.
    // Returns ZX_ERR_WRONG_TYPE for object types that don't have ZX_PROP_NAME.
    get_name(): Result<string, fx_status_t> {
        return Err(FX_ERR_WRONG_TYPE)
    }

    // set_name() will truncate to ZX_MAX_NAME_LEN - 1 and ensure there is a
    // terminating null.
    // Returns ZX_ERR_WRONG_TYPE for object types that don't have ZX_PROP_NAME.
    set_name(name: string): fx_status_t {
        return FX_ERR_WRONG_TYPE
    }

    /**
     * Called whenever the object is bound to a new process. The |new_owner| is
     * the koid of the new process. It is only overridden for objects where a single
     * owner makes sense.
     */
    set_owner(new_owner: fx_koid_t): void {
        return
    }

    get_related_koid(): fx_koid_t {
        return FX_KOID_INVALID
    }
}

export class SoloDispatcher extends Dispatcher {
    //static default_rights(): fx_rights_t {
    //    return def_rights
    //}

    // At construction, the object is asserting |signals|.
    constructor(signals: fx_signals_t = 0) {
        super(signals)
    }

    // Related koid is overridden by subclasses, like thread and process.
    override get_related_koid(): fx_koid_t {
        return FX_KOID_INVALID
    }
}

// PeeredDispatchers have opposing endpoints to coordinate state
// with. For example, writing into one endpoint of a Channel needs to
// modify fx_signals_t state (for the readability bit) on the opposite
// side. To coordinate their state, they share a mutex, which is held
// by the PeerHolder. Both endpoints have a RefPtr back to the
// PeerHolder; no one else ever does.
// Thus creating a pair of peered objects will typically look
// something like
//     // Make the two RefPtrs for each endpoint's handle to the mutex.
//     auto holder0 = AdoptRef(new PeerHolder<Foo>(...));
//     auto holder1 = peer_holder0;
//     // Create the opposing sides.
//     auto foo0 = AdoptRef(new Foo(ktl::move(holder0, ...));
//     auto foo1 = AdoptRef(new Foo(ktl::move(holder1, ...));
//     // Initialize the opposing sides, teaching them about each other.
//     foo0->Init(&foo1);
//     foo1->Init(&foo0);
// A PeeredDispatcher object, in its |on_zero_handles| call must clear
// out its peer's |peer_| field. This is needed to avoid leaks, and to
// ensure that |user_signal| can correctly report ZX_ERR_PEER_CLOSED.
// TODO(kulakowski) We should investigate turning this into one
// allocation. This would mean PeerHolder would have two EndPoint
// members, and that PeeredDispatcher would have custom refcounting.
export class PeerHolder<T> {}

export class PeeredDispatcher<T extends Dispatcher> extends Dispatcher {
    private _holder: PeerHolder<T>
    private _peer: Ref<T> | null
    private _peer_koid: fx_koid_t

    constructor(holder: PeerHolder<T>, signals: fx_signals_t = 0) {
        super(signals)

        this._holder = holder
        this._peer = null
        this._peer_koid = FX_KOID_INVALID
    }

    public peer_koid(): fx_koid_t | null {
        return this._peer_koid
    }

    public peer(): Ref<T> | null {
        return this._peer
    }

    override get_related_koid(): fx_koid_t {
        invariant(this._peer_koid !== null)
        return this._peer_koid!
    }

    // Initialize this dispatcher's peer field.
    //
    // This method is logically part of the class constructor and must be called exactly once, during
    // initialization, prior to any other thread obtaining a reference to the object.  These
    // constraints allow for an optimization where fields are accessed without acquiring the lock,
    // hence the TA_NO_THREAD_SAFETY_ANALYSIS annotation.
    init_peer(peer: Ref<T>): void {
        //assert(!peer_);
        invariant(this._peer_koid === FX_KOID_INVALID)
        this._peer = peer
        this._peer_koid = this._peer.value.get_koid()
    }

    // is_waitable(): boolean {
    //     return PeeredDispatcher.default_rights() & FX_RIGHT_WAIT;
    // }
}
