export type u16 = number
export type i16 = number

export type u32 = number
export type i32 = number

export type u64 = bigint
export type i64 = bigint

export type f32 = number // A 32-bit float.
export type f64 = number // A 64-bit float.

export type fx_handle_t = u32
export type fx_status_t = i32
export type fx_vaddr_t = u64
export type fx_rights_t = u32
export type fx_koid_t = u64
export type fx_txid_t = number
export type fx_signals_t = u32
export type fx_policy_t = number
export type fx_time_t = i64
export type fx_handle_type_t = u32
export type fx_handle_op_t = u32
export type fx_obj_type_t = u32

// multiconst: fx_time_t
export const FX_TIME_INFINITE: fx_time_t = 9_223_372_036_854_775_807n
export const FX_TIME_INFINITE_PAST: fx_time_t = -9_223_372_036_854_775_808n

// multiconst: fx_handle_t
export const FX_INVALID_HANDLE: fx_handle_t = 0

// multiconst: fx_handle_op_t
export const FX_HANDLE_OP_MOVE: fx_handle_op_t = 0
export const FX_HANDLE_OP_DUPLICATE: fx_handle_op_t = 1

// multiconst: fx_koid_t
export const FX_KOID_INVALID: fx_koid_t = 0n
export const FX_KOID_KERNEL: fx_koid_t = 1n
export const FX_KOID_FIRST: fx_koid_t = 1024n

// multiconst: fx_signals_t
export const FX_SIGNAL_NONE = 0
export const FX_OBJECT_SIGNAL_ALL = 0x00ffffff
export const FX_USER_SIGNAL_ALL = 0xff000000
export const FX_OBJECT_SIGNAL_0 = 1 << 0
export const FX_OBJECT_SIGNAL_1 = 1 << 1
export const FX_OBJECT_SIGNAL_2 = 1 << 2
export const FX_OBJECT_SIGNAL_3 = 1 << 3
export const FX_OBJECT_SIGNAL_4 = 1 << 4
export const FX_OBJECT_SIGNAL_5 = 1 << 5
export const FX_OBJECT_SIGNAL_6 = 1 << 6
export const FX_OBJECT_SIGNAL_7 = 1 << 7
export const FX_OBJECT_SIGNAL_8 = 1 << 8
export const FX_OBJECT_SIGNAL_9 = 1 << 9
export const FX_OBJECT_HANDLE_CLOSED = 1 << 23

export const FX_OBJECT_READABLE = FX_OBJECT_SIGNAL_0
export const FX_OBJECT_WRITABLE = FX_OBJECT_SIGNAL_1
export const FX_OBJECT_PEER_CLOSED = FX_OBJECT_SIGNAL_2

// channel signals
export const FX_CHANNEL_READABLE = FX_OBJECT_SIGNAL_0
export const FX_CHANNEL_WRITABLE = FX_OBJECT_SIGNAL_1
export const FX_CHANNEL_PEER_CLOSED = FX_OBJECT_SIGNAL_2

// process signals
export const FX_PROCESS_TERMINATED = FX_OBJECT_SIGNAL_3

// cancelation (handle was closed while waiting with it)
export const FX_SIGNAL_HANDLE_CLOSED = FX_OBJECT_HANDLE_CLOSED

// other contants
export const FX_MAX_NAME_LEN = 100
export const FX_PROCESS_SHARED = 1

export const FX_CHANNEL_READ_MAY_DISCARD = 0
export const FX_CHANNEL_MAX_MSG_BYTES = 65536
export const FX_CHANNEL_MAX_MSG_HANDLES = 64

export const FX_RIGHT_NONE: fx_rights_t = 0
export const FX_RIGHT_DUPLICATE: fx_rights_t = 1 << 0
export const FX_RIGHT_TRANSFER: fx_rights_t = 1 << 1
export const FX_RIGHT_READ: fx_rights_t = 1 << 2
export const FX_RIGHT_WRITE: fx_rights_t = 1 << 3
export const FX_RIGHT_EXECUTE: fx_rights_t = 1 << 4
export const FX_RIGHT_MAP: fx_rights_t = 1 << 5
export const FX_RIGHT_GET_PROPERTY: fx_rights_t = 1 << 6
export const FX_RIGHT_SET_PROPERTY: fx_rights_t = 1 << 7
export const FX_RIGHT_ENUMERATE: fx_rights_t = 1 << 8
export const FX_RIGHT_DESTROY: fx_rights_t = 1 << 9
export const FX_RIGHT_SET_POLICY: fx_rights_t = 1 << 10
export const FX_RIGHT_GET_POLICY: fx_rights_t = 1 << 11
export const FX_RIGHT_SIGNAL: fx_rights_t = 1 << 12
export const FX_RIGHT_SIGNAL_PEER: fx_rights_t = 1 << 13
export const FX_RIGHT_WAIT: fx_rights_t = 1 << 14
export const FX_RIGHT_INSPECT: fx_rights_t = 1 << 15
export const FX_RIGHT_MANAGE_JOB: fx_rights_t = 1 << 16
export const FX_RIGHT_MANAGE_PROCESS: fx_rights_t = 1 << 17
export const FX_RIGHT_MANAGE_THREAD: fx_rights_t = 1 << 18
export const FX_RIGHT_APPLY_PROFILE: fx_rights_t = 1 << 19
export const FX_RIGHT_MANAGE_SOCKET: fx_rights_t = 1 << 20
export const FX_RIGHT_SAME_RIGHTS: fx_rights_t = 1 << 31

export const FX_POL_NEW_PROCESS = 1

export function todo() {
    throw new Error("not implemented")
}

export type fx_handle_disposition_t = {
    handle: fx_handle_t
    result: fx_status_t
    type: fx_handle_type_t
    rights: fx_rights_t
    operation: number
}

export type fx_handle_info_t = {
    handle: fx_handle_t
    type: fx_handle_type_t
    rights: fx_rights_t
}

export type fx_port_packet_t = {
    key: u64
    status: fx_status_t
    packet_type: fx_packet_type_t
    union: fx_packet_signal_t | fx_packet_user_t
}

export enum fx_packet_type_t {
    FX_PKT_TYPE_USER = 0,
    FX_PKT_TYPE_SIGNAL_ONE = 1,
}

export type fx_packet_user_t = {}

export type fx_packet_signal_t = {
    trigger: fx_signals_t
    observed: fx_signals_t
    count: u64
}

export enum Status {
    OK = 1,
    ERR_UNSUPPORTED = 2,
    ERR_NOT_SUPPORTED = 3,
    ERR_CANCELED = 3,
    ERR_OUT_OF_RANGE = 4,
    ERR_BAD_STATE = 5,
    ERR_BUFFER_TOO_SMALL = 6,
    ERR_BAD_HANDLE = 7,
    ERR_SHOULD_WAIT = 8,
    ERR_PEER_CLOSED = 9,
    ERR_INVALID_ARGS = 10,
}

export class Ref<T> {
    private _value: T | undefined

    constructor(value: T) {
        this._value = value
    }

    get value(): T {
        if (typeof this._value == "undefined") throw new Error("ref is empty")
        return this._value
    }

    set value(newValue: T) {
        this._value = newValue
    }

    take(): T | undefined {
        const value = this._value
        this._value = undefined
        return value
    }
}
