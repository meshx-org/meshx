import { fx_handle_t } from "@meshx-org/fiber-types"
import { Channel, Handle } from "@meshx-org/fiber-ts"

export const PROC_SELF = 0
export const ROOT_JOB = 1

export const HANDLE_COUNT = 3
export const CHILD_HANDLE_COUNT = HANDLE_COUNT + 5

function bootstrap(channel: Channel) {
    console.log(channel)
}

export function _start(arg1: fx_handle_t) {
    bootstrap(Channel.from_handle(Handle.from_raw(arg1)))
}
