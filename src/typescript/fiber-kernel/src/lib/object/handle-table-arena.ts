import invariant from "tiny-invariant"
import { Ref } from "../std"
import { Dispatcher } from "./dispatchers/dispatcher"
import { HANDLE_GENERATION_MASK, HANDLE_GENERATION_SHIFT, HANDLE_INDEX_MASK, HIGH_HANDLE_COUNT, Handle } from "./handle"
import { FX_KOID_INVALID, u32 } from "@meshx-org/fiber-types"

class Ptr<T> {
    index!: number
    data: T | null = null
}

class Arena<T> {
    #free: Ptr<T>[] = []
    #count = 0

    alloc(): Ptr<T> {
        this.#count += 1
        const ptr = new Ptr<T>()

        const new_index = this.#free.push(ptr)
        ptr.index = new_index - 1

        return ptr
    }

    diagnostic_count() {
        return this.#count
    }

    free(value: T): void {
        //this.#free.find(v => v === value) = new Ptr<T>();
        //this.#count--;
    }

    get(addr: number): Ptr<T> {
        return this.#free[addr]
    }
}

// |index| is the literal index into the table. |old_value| is the
// |index mixed with the per-handle-lifetime state.
function new_handle_value(index: u32, old_value: u32): u32 {
    invariant((index & ~HANDLE_INDEX_MASK) == 0)

    let old_gen = 0
    if (old_value != 0) {
        // This slot has been used before.
        invariant((old_value & HANDLE_INDEX_MASK) == index)
        old_gen = (old_value & HANDLE_GENERATION_MASK) >> HANDLE_GENERATION_SHIFT
    }

    const new_gen = ((old_gen + 1) << HANDLE_GENERATION_SHIFT) & HANDLE_GENERATION_MASK
    return index | new_gen
}

export class HandleTableArena {
    public _arena = new Arena<Handle>()

    private get_new_base_value(addr: Ptr<Handle>) {
        // Get the index of this slot within the arena.
        const handle_index = addr.index

        // Check the free memory for a stashed base_value.
        const v = 0 // addr.data.index_self();

        return new_handle_value(handle_index, v)
    }

    // Allocate space for a Handle from the arena, but don't instantiate the
    // object.  |base_value| gets the value for Handle::base_value_.  |what|
    // says whether this is allocation or duplication, for the error message.
    public alloc(dispatcher: Dispatcher, what: string, base_value: Ref<number>): Ptr<Handle> | null {
        // Attempt to allocate a handle.
        const addr = this._arena.alloc()

        const outstanding_handles = this._arena.diagnostic_count()

        if (addr === null) {
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
        invariant(addr.data === null, "handle value already in use")
        //invariant(addr.data && addr.data.handle_table_id() === FX_KOID_INVALID);
        //invariant(addr.data && addr.data.dispatcher === null);

        base_value.value = this.get_new_base_value(addr)

        return addr
    }

    delete(handle: Handle): void {
        const dispatcher = handle.dispatcher()
        const old_self_index = handle.index_self()
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
        // delete handle

        // Make sure the base value was not altered by the destructor.
        // assert(base_value == old_base_value);

        this._arena.free(handle)

        const zero_handles = dispatcher.decrement_handle_count()

        // TODO:
        if (zero_handles) {
            dispatcher.on_zero_handles()
        }

        // If |disp| is the last reference (which is likely) then the dispatcher object
        // gets destroyed at the exit of this function.
        // kcounter_add(handle_count_live, -1);
    }
}

export const gHandleTableArena = new HandleTableArena()
