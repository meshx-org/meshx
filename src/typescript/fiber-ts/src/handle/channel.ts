import { fx_channel_create } from "@meshx-org/fiber-sys"
import { FX_INVALID_HANDLE, Ref, Status, fx_handle_t } from "@meshx-org/fiber-types"
import { HandleWrapper } from "./handle-wrapper"
import { Handle } from "./handle"

/// Typed wrapper around a linked pair of channel objects and the
/// fx_channel_create() syscall used to create them.
export class Channel extends HandleWrapper {
    static create(): [Channel, Channel] | null {
        const first = new Ref(FX_INVALID_HANDLE)
        const second = new Ref(FX_INVALID_HANDLE)
        const status = fx_channel_create(first, second)

        if (status !== Status.OK) {
            return null
        }

        const firstChannel = new Channel(first.value)
        const secondChannel = new Channel(second.value)

        return [firstChannel, secondChannel]
    }

    private constructor(first: fx_handle_t) {
        super(first)
    }

    /** Creates an instance of this type from a handle. */ 
    public static from_handle(handle: Handle) {
        return new Channel(handle.raw)
    }
}
