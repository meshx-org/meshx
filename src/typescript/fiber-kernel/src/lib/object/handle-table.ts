import {
    FX_ERR_BAD_HANDLE,
    FX_HANDLE_FIXED_BITS_MASK,
    fx_handle_t,
    FX_KOID_INVALID,
    fx_koid_t,
    fx_rights_t,
    fx_status_t,
} from "@meshx-org/fiber-types"
import { ProcessDispatcher } from "./dispatchers/process-dispatcher"
import { Handle, HANDLE_RESERVED_BITS } from "./handle"
import { generate } from "../koid"
import invariant from "tiny-invariant"
import { HandleOwner } from "./handle-owner"
import { debug_invariant, Err, Ok, Result } from "../std"
import { Dispatcher } from "./dispatchers"

const kHandleMustBeOneMask = (0x1 << HANDLE_RESERVED_BITS) - 1
invariant(
    kHandleMustBeOneMask === FX_HANDLE_FIXED_BITS_MASK,
    "kHandleMustBeOneMask must match ZX_HANDLE_FIXED_BITS_MASK!"
)

function map_value_to_handle(value: number, mixer: number): Handle | null {
    // Validate that the "must be one" bits are actually one.
    if ((value & kHandleMustBeOneMask) != kHandleMustBeOneMask) {
        return null
    }

    const handle_id = (value ^ mixer) >> HANDLE_RESERVED_BITS

    const handle = Handle.from_u32(handle_id)
    return handle ? handle.data : null
}

export class HandleTable {
    /** Normalized parent process koid. */
    #process_koid: fx_koid_t

    /**
     * The koid of this handle table. Used to check whether or not a handle belongs to this handle
     * table (and thus that it belongs to a process associated with this handle table).
     */
    #koid: fx_koid_t

    // Each handle table provides pseudorandom userspace handle
    // values. This is the per-handle-table pseudorandom state.
    #random_value = 0

    // The actual handle table. When removing one or more handles from this list, be sure to
    // advance or invalidate any cursors that might point to the handles being removed.
    #count: number
    #handles: Handle[]

    constructor(process: ProcessDispatcher) {
        const array = new Uint32Array(1)
        crypto.getRandomValues(array)

        // Generate handle XOR mask with top bit and bottom two bits cleared
        const secret: number = array[0]

        // Handle values must always have the low kHandleReservedBits set.  Do not
        // ever attempt to toggle these bits using the random_value_ xor mask.
        const random_value = secret << HANDLE_RESERVED_BITS

        this.#koid = generate()
        this.#random_value = random_value
        this.#process_koid = process.get_koid()
        this.#count = 0
        this.#handles = []
    }

    get_koid() {
        return this.#koid
    }

    add_handle(handle: HandleOwner) {
        this.add_handle_locked(handle)
    }

    add_handle_locked(handle: HandleOwner) {
        handle.set_handle_table_id(this.#koid)

        this.#handles.unshift(handle.handle)
        this.#count += 1
    }

    remove_handle_locked(handle: Handle): HandleOwner {
        debug_invariant(this.#count > 0)
        handle.set_handle_table_id(FX_KOID_INVALID)
        // Make sure we don't leave any dangling cursors.
        //for (let cursor of this.#cursors) {
        // If it points to |handle|, skip over it.
        //    cursor.AdvanceIf(handle);
        // }
        this.#handles = this.#handles.filter((h) => h !== handle)
        this.#count--
        return new HandleOwner(handle)
    }

    remove_handle_locked_wcaller(caller: ProcessDispatcher, handle_value: fx_handle_t) {
        const handle = this.get_handle_locked(caller, handle_value);
        if (!handle) return null;
        return this.remove_handle_locked(handle);
    }

    remove_handle(caller: ProcessDispatcher, handle_value: fx_handle_t) {
        return this.remove_handle_locked_wcaller(caller, handle_value)
    }

    /** Maps a |handle| to an integer which can be given to usermode as a
     *  handle value. Uses Handle->base_value() plus additional mixing.
     */
    public map_handleowner_to_value(handle: HandleOwner): fx_handle_t {
        return map_handle_to_value(handle.handle, this.#random_value)
    }
    public map_handle_to_value(handle: Handle): fx_handle_t {
        return map_handle_to_value(handle, this.#random_value)
    }

    // Maps a handle value into a Handle as long we can verify that
    // it belongs to this handle table.
    get_handle_locked(caller: ProcessDispatcher, handle_value: fx_handle_t): Handle | null {
        const handle = map_value_to_handle(handle_value, this.#random_value)

        if (handle && handle.handle_table_id() == this.#koid) {
            return handle
        }

        // TODO: enforce policy
        if (caller) {
            // Handle lookup failed.  We potentially generate an exception or kill the process,
            // depending on the job policy. Note that we don't use the return value from
            // EnforceBasicPolicy() here: ZX_POL_ACTION_ALLOW and ZX_POL_ACTION_DENY are equivalent for
            // ZX_POL_BAD_HANDLE.
            // let result = caller.enforce_basic_policy(sys::FX_POL_BAD_HANDLE);
        }

        return null
    }

    /// Get the dispatcher and the rights corresponding to this handle value.
    get_dispatcher_with_rights<T extends Dispatcher>(
        caller: ProcessDispatcher,
        handle_value: fx_handle_t,
        rights: fx_rights_t
    ): Result<T, fx_status_t> {
        const result = this.get_dispatcher_internal(caller, handle_value, rights)

        if (result.is_err()) {
            Err(result)
        }

        return Ok(result.unwrap() as T)
    }

    get_dispatcher_internal(
        caller: ProcessDispatcher,
        handle_value: fx_handle_t,
        handle_rights: fx_rights_t
    ): Result<Dispatcher, fx_status_t> {
        //let dispatcher: Rc<dyn Any> = Rc::from(JobDispatcher::new(0, None, JobPolicy));

        const handle = this.get_handle_locked(caller, handle_value)

        if (handle === null) {
            console.log("get_dispatcher_internal", handle)
            return Err(FX_ERR_BAD_HANDLE)
        }

        const rights = handle.rights()

        return Ok(handle.dispatcher())
    }
}

const HANDLE_MUST_BE_ONE_MASK = (0x1 << HANDLE_RESERVED_BITS) - 1
// assert(HANDLE_MUST_BE_ONE_MASK == FX_HANDLE_FIXED_BITS_MASK); // kHandleMustBeOneMask must match ZX_HANDLE_FIXED_BITS_MASK!

export function map_handle_to_value(handle: Handle, mixer: number): fx_handle_t {
    // Ensure that the last two bits of the result is not zero, and make sure we
    // don't lose any base_value bits when shifting.
    const base_value_must_be_zero_mask = HANDLE_MUST_BE_ONE_MASK << (4 * 8 - HANDLE_RESERVED_BITS)

    invariant((mixer & HANDLE_MUST_BE_ONE_MASK) === 0)
    invariant((handle.index_self() & base_value_must_be_zero_mask) === 0)

    const handle_id = (handle.index_self() << HANDLE_RESERVED_BITS) | HANDLE_MUST_BE_ONE_MASK

    return mixer ^ handle_id
}
