import { fx_rights_t, fx_handle_type_t, fx_status_t, Status } from "@meshx-org/fiber-types"
import { Handle } from "./handle"

export interface HandleDisposition {
    handle?: Handle
    result: fx_status_t

    readonly type: fx_handle_type_t
    readonly rights: fx_rights_t
    readonly operation: number
}

export interface HandleInfo {
    readonly handle: Handle
    readonly type: fx_handle_type_t
    readonly rights: fx_rights_t
}

export interface ReadResult {
    readonly status: Status

    readonly bytes?: DataView
    readonly numBytes?: number
    readonly handles?: Handle[]
}

export interface ReadEtcResult {
    readonly status: Status

    readonly bytes?: DataView
    readonly numBytes?: number
    readonly handleInfos?: HandleInfo[]
}
