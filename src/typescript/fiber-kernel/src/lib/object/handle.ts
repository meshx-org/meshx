import { fx_koid_t, fx_rights_t } from "@meshx-org/fiber-types"
import { Dispatcher } from "./dispatchers/dispatcher"
import invariant from "tiny-invariant"
import { Ref } from "../std"
import { gHandleTableArena } from "./handle-table-arena"
import { HandleOwner } from './handle-owner'

/**
 * A minimal wrapper around a Dispatcher which is owned by the kernel.
 *
 * Intended usage when creating new a Dispatcher object is:
 *  1. Create a KernelHandle on the stack (cannot fail)
 *  2. Move the RefPtr<Dispatcher> into the KernelHandle (cannot fail)
 *  3. When ready to give the handle to a process, upgrade the KernelHandle
 *     to a full HandleOwner via UpgradeToHandleOwner() or
 *     user_out_handle::make() (can fail)
 *
 * This sequence ensures that the Dispatcher's on_zero_handles() method is
 * called even if errors occur during or before HandleOwner creation, which
 * is necessary to break circular references for some Dispatcher types.
 *
 * This class is thread-unsafe and must be externally synchronized if used
 * across multiple threads.
 */
export class KernelHandle<T> {
    private _dispatcher: Dispatcher | null

    constructor(dispather: Dispatcher) {
        this._dispatcher = dispather
    }

    public dispatcher(): Dispatcher | null {
        return this._dispatcher
    }

    public release(): Dispatcher {
        const dispatcher = this._dispatcher!
        this._dispatcher = null
        return dispatcher
    }
}

/** A Handle is how a specific process refers to a specific Dispatcher. */
export class Handle {
    public _handle_table_id: bigint
    public _dispatcher: Dispatcher
    public _handle_rights: fx_rights_t
    public _base_value: number

    /** Called only by Make. */
    private constructor(dispatcher: Dispatcher, rights: fx_rights_t, base_value: number) {
        this._handle_table_id = 0n
        this._handle_rights = rights
        this._dispatcher = dispatcher
        this._base_value = base_value
    }

    // Called only by Dup.
    private static create_dup(rhs: Handle, rights: fx_rights_t, base_value: number): Handle {
        const dispatcher = rhs.dispatcher()
        return new Handle(dispatcher, rights, base_value)
    }

    // eslint-disable-next-line @typescript-eslint/no-empty-function
    public destructor(): void {}

    public static make(kernel_handle: KernelHandle<Dispatcher>, rights: fx_rights_t): HandleOwner {
        const base_value = new Ref(0)
        const addr = gHandleTableArena.alloc(kernel_handle.dispatcher()!, "new", base_value)

        if (!addr) {
            throw "alloc failed"
        }

        //kcounter_add(handle_count_made, 1);
        //kcounter_add(handle_count_live, 1);
        return new HandleOwner(new Handle(kernel_handle.release(), rights, base_value.value))

        // return new Handle(handle.dispatcher(), rights, 0) // TODO: figure out how to do Arc here
    }

    static dup(source: Handle, rights: fx_rights_t): HandleOwner {
        const base_value = new Ref(0)
        const addr = gHandleTableArena.alloc(source.dispatcher(), "duplicate", base_value)

        if (!addr) {
            throw "alloc failed"
        }

        //kcounter_add(handle_count_duped, 1);
        //kcounter_add(handle_count_live, 1);
        return new HandleOwner(Handle.create_dup(source, rights, base_value.value))
    }

    /**
     *  Returns a value that can be decoded by Handle::FromU32() to derive a
     *  pointer to this instance. ProcessDispatcher will XOR this with its
     *  handle_rand_ to create the fx_handle_t value that user space sees.
     */
    public base_value(): number {
        return this._base_value
    }

    /** Returns the Dispatcher to which this instance points. */
    public dispatcher(): Dispatcher {
        return this._dispatcher
    }

    /** Sets the value returned by handle_table_id(). */
    public set_handle_table_id(pid: fx_koid_t): void {
        this._handle_table_id = pid
        this._dispatcher.set_owner(pid)
    }

    /** Returns the handle table that owns this instance. Used to guarantee
     *  that a process may only access handles in its own handle table.
     */
    public handle_table_id(): fx_koid_t {
        return this._handle_table_id
    }
}

// Compute floor(log2(|val|)), or 0 if |val| is 0
function bit_width(x: number): number {
    let i
    let j
    let k

    x = x | (x >> 1)
    x = x | (x >> 2)
    x = x | (x >> 4)
    x = x | (x >> 8)
    x = x | (x >> 16)

    // i = 0x55555555
    i = 0x55 | (0x55 << 8)
    i = i | (i << 16)

    // j = 0x33333333
    j = 0x33 | (0x33 << 8)
    j = j | (j << 16)

    // k = 0x0f0f0f0f
    k = 0x0f | (0x0f << 8)
    k = k | (k << 16)

    // l = 0x00ff00ff
    const l = 0xff | (0xff << 16)

    // m = 0x0000ffff
    const m = 0xff | (0xff << 8)

    x = (x & i) + ((x >> 1) & i)
    x = (x & j) + ((x >> 2) & j)
    x = (x & k) + ((x >> 4) & k)
    x = (x & l) + ((x >> 8) & l)
    x = (x & m) + ((x >> 16) & m)
    x = x + ~0
    return x
}

function log2_floor(val: number): number {
    return val == 0 ? 0 : Math.floor(Math.log2(val)) // bit_width(val)
}

function log2_uint_floor(val: number) {
    return log2_floor(val)
}

// The number of outstanding (live) handles in the arena.
export const MAX_HANDLE_COUNT = 256 * 1024

// Warning level: When the number of handles exceeds this value, we start to emit
// warnings to the kernel's debug log.
export const HIGH_HANDLE_COUNT = (MAX_HANDLE_COUNT * 7) / 8

// Masks for building a Handle's base_value, which ProcessDispatcher
// uses to create fx_handle_t values.
//
// base_value bit fields:
//   [31..(32 - HANDLE_RESERVED_BITS)]                      : Must be zero
//   [(31 - HANDLE_RESERVED_BITS)..HANDLE_GENERATION_SHIFT] : Generation number
//                                                          Masked by kHandleGenerationMask
//   [HANDLE_GENERATION_SHIFT-1..0]                         : Index into handle_arena
//

invariant(0 == log2_uint_floor(0))
invariant(0 == log2_uint_floor(1))
invariant(1 == log2_uint_floor(2))
invariant(1 == log2_uint_floor(3))
invariant(2 == log2_uint_floor(4))

export const HANDLE_RESERVED_BITS = 2
export const HANDLE_INDEX_MASK = MAX_HANDLE_COUNT - 1
export const HANDLE_RESERVED_BITS_MASK = ((1 << HANDLE_RESERVED_BITS) - 1) << (32 - HANDLE_RESERVED_BITS)
export const HANDLE_GENERATION_MASK = ~HANDLE_INDEX_MASK & ~HANDLE_RESERVED_BITS_MASK
export const HANDLE_GENERATION_SHIFT = log2_uint_floor(MAX_HANDLE_COUNT)

invariant((HANDLE_INDEX_MASK & MAX_HANDLE_COUNT) == 0) //kMaxHandleCount must be a power of 2
invariant(((3 << (HANDLE_GENERATION_SHIFT - 1)) & HANDLE_GENERATION_MASK) == 1 << HANDLE_GENERATION_SHIFT) //Shift is wrong
invariant(HANDLE_GENERATION_MASK >> HANDLE_GENERATION_SHIFT >= 255) // Not enough room for a useful generation count
invariant((HANDLE_RESERVED_BITS_MASK & HANDLE_GENERATION_MASK) == 0) // Handle Mask Overlap!
invariant((HANDLE_RESERVED_BITS_MASK & HANDLE_INDEX_MASK) == 0) // Handle Mask Overlap!
invariant((HANDLE_GENERATION_MASK & HANDLE_INDEX_MASK) == 0) // Handle Mask Overlap!
// assert((HANDLE_RESERVED_BITS_MASK | HANDLE_GENERATION_MASK | HANDLE_INDEX_MASK) == 0xffffffff) // Handle masks do not cover all bits!
