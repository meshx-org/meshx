import { fx_rights_t, fx_handle_type_t, fx_status_t, Status, fx_obj_type_t, FX_OK, u32 } from "@meshx-org/fiber-types"
import { Handle } from "./handle"

export class HandleDisposition implements HandleDisposition {
    handle: Handle
    result: number
    readonly type: number
    readonly rights: number
    readonly operation: number

    constructor(operation: number, handle: Handle, type: fx_obj_type_t, rights: fx_rights_t) {
        this.operation = operation
        this.handle = handle
        this.type = type
        this.rights = rights
        this.result = FX_OK
    }
}

export interface HandleInfo {
    readonly handle: Handle
    readonly type: fx_handle_type_t
    readonly rights: fx_rights_t
}

export interface ReadResult {
    readonly status: Status

    readonly numBytes: u32,
    readonly bytes: Uint8Array
    readonly handles: Handle[]
}

export interface ReadEtcResult {
    readonly status: Status

    readonly bytes?: Uint8Array
    readonly handleInfos?: HandleInfo[]
}

export interface CreateResult<T> {
    status: Status
    handle?: T
}
