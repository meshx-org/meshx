import { fx_handle_t, fx_koid_t } from "@meshx-org/fiber-types"
import { ProcessDispatcher } from "./dispatchers/process-dispatcher"
import { HANDLE_RESERVED_BITS } from "./handle"
import { generate } from "../koid"
import invariant from "tiny-invariant"
import { HandleOwner } from './handle-owner'

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
