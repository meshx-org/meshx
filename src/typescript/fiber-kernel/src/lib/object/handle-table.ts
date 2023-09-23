import { FX_KOID_INVALID, fx_handle_t, fx_koid_t } from "@meshx-org/fiber-types"
import { ProcessDispatcher } from "./process-dispatcher"
import { HANDLE_RESERVED_BITS, HIGH_HANDLE_COUNT, Handle, HandleOwner } from "./handle"
import { generate } from "../koid"
import invariant from "tiny-invariant"
import { Dispatcher } from "./dispatcher"
import { Ref } from "../std"

class Ptr<T> {
    data: T | null

    constructor() {
        this.data = null
    }
}

class Arena<T> {
    private _free: Ptr<T>[] = []
    private _count = 0

    alloc(): Ptr<T> {
        this._count += 1
        const ptr = new Ptr<T>()
        this._free.push(ptr)
        return ptr
    }

    diagnostic_count() {
        return this._count
    }
}

export class HandleTableArena {
    private _arena = new Arena<Handle>()

    private get_new_base_value(addr: Ptr<Handle>) {
        return 12
    }

    public alloc(dispatcher: Dispatcher, what: string, base_value: Ref<number>): Ptr<Handle> | null {
        // Attempt to allocate a handle.
        const addr = this._arena.alloc()

        const outstanding_handles = this._arena.diagnostic_count()

        if (addr == null) {
            // kcounter_add(handle_count_alloc_failed, 1);
            console.warn(`WARNING: Could not allocate ${what} handle (${outstanding_handles} outstanding)`)
            return null
        }

        // Emit a warning if too many handles have been created and we haven't recently logged
        if (outstanding_handles > HIGH_HANDLE_COUNT) {
            console.warn("WARNING: High handle count: %zu / %zu handles\n", outstanding_handles, HIGH_HANDLE_COUNT)
        }

        dispatcher.increment_handle_count()
        // TODO

        // checking the handle_table_id_ and dispatcher_ is really about trying to catch cases where this
        // Handle might somehow already be in use.
        
        // assert(addr.data?._handle_table_id == FX_KOID_INVALID)
        // assert(addr.data?._dispatcher == null)

        // base_value.value = this.get_new_base_value(addr)

        return addr
    }

    delete(handle: Handle): void {
        const dispatcher = handle._dispatcher
        const old_base_value = handle._base_value
        // const base_value = &handle.base_value_;
        // There may be stale pointers to this slot and they will look at handle_table_id. We expect
        // handle_table_id to already have been cleared by the process dispatcher before the handle got to
        // this point.
        invariant(handle.handle_table_id() == FX_KOID_INVALID)

        // TODO:
        //if (dispatcher.is_waitable()) {
        //    dispatcher.cancel(handle);
        //}

        // The destructor should not do anything interesting but call it for completeness.
        handle.destructor()

        // Make sure the base value was not altered by the destructor.
        // assert(base_value == old_base_value);

        const zero_handles = dispatcher.decrement_handle_count()
        // TODO: this._arena.free(handle)

        // TODO:
        if (zero_handles) {
            //    dispatcher.on_zero_handles()
        }

        // If |disp| is the last reference (which is likely) then the dispatcher object
        // gets destroyed at the exit of this function.
        // kcounter_add(handle_count_live, -1);
    }
}

export class HandleTable {
    /** Normalized parent process koid. */
    private _process_koid: fx_koid_t

    /**
     * The koid of this handle table. Used to check whether or not a handle belongs to this handle
     * table (and thus that it belongs to a process associated with this handle table).
     */
    private _koid: fx_koid_t

    // Each handle table provides pseudorandom userspace handle
    // values. This is the per-handle-table pseudorandom state.
    private _random_value = 0

    // The actual handle table. When removing one or more handles from this list, be sure to
    // advance or invalidate any cursors that might point to the handles being removed.
    private _count: number
    private _handles: HandleOwner[]

    constructor(process: ProcessDispatcher) {
        // Generate handle XOR mask with top bit and bottom two bits cleared
        const secret: number = Math.random() * 100

        // Handle values must always have the low kHandleReservedBits set.  Do not
        // ever attempt to toggle these bits using the random_value_ xor mask.
        const random_value = secret << HANDLE_RESERVED_BITS

        console.log("random value", random_value)

        this._koid = generate()
        this._random_value = random_value
        this._process_koid = process.get_koid()
        this._count = 0
        this._handles = []
    }

    add_handle(handle: HandleOwner) {
        this.add_handle_locked(handle)
    }

    private add_handle_locked(handle: HandleOwner) {
        handle.set_handle_table_id(this._process_koid)

        this._handles.unshift(handle)
        this._count += 1
    }

    /** Maps a |handle| to an integer which can be given to usermode as a
     *  handle value. Uses Handle->base_value() plus additional mixing.
     */
    public map_handle_to_value(handle: HandleOwner): fx_handle_t {
        return map_handle_to_value(handle, this._random_value)
    }
}

const HANDLE_MUST_BE_ONE_MASK = (0x1 << HANDLE_RESERVED_BITS) - 1
// assert(HANDLE_MUST_BE_ONE_MASK == FX_HANDLE_FIXED_BITS_MASK); // kHandleMustBeOneMask must match ZX_HANDLE_FIXED_BITS_MASK!

function map_handle_to_value(handle: HandleOwner, mixer: number): fx_handle_t {
    // Ensure that the last two bits of the result is not zero, and make sure we
    // don't lose any base_value bits when shifting.
    const base_value_must_be_zero_mask = HANDLE_MUST_BE_ONE_MASK << (4 * 8 - HANDLE_RESERVED_BITS)

    invariant((mixer & HANDLE_MUST_BE_ONE_MASK) === 0)
    invariant((handle.base_value() & base_value_must_be_zero_mask) === 0)

    const handle_id = (handle.base_value() << HANDLE_RESERVED_BITS) | HANDLE_MUST_BE_ONE_MASK

    return mixer ^ handle_id
}
