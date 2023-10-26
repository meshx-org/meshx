import invariant from 'tiny-invariant'
import { Ref } from "../std"
import { Dispatcher } from "./dispatchers/dispatcher"
import { HIGH_HANDLE_COUNT, Handle } from "./handle"
import { FX_KOID_INVALID } from '@meshx-org/fiber-types'

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

export const gHandleTableArena = new HandleTableArena()