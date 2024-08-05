export type u16 = number;
export type i16 = number;

export type u32 = number;
export type i32 = number;

export type u64 = bigint;
export type i64 = bigint;

export type f32 = number; // A 32-bit float.
export type f64 = number; // A 64-bit float.

export type fx_handle_t = u32;
export type fx_status_t = i32;
export type fx_vaddr_t = u64;
export type fx_rights_t = u32;
export type fx_koid_t = u64;
export type fx_txid_t = number;
export type fx_signals_t = u32;
export type fx_policy_t = number;
export type fx_time_t = i64;
export type fx_handle_type_t = u32;
export type fx_handle_op_t = u32;
export type fx_obj_type_t = u32;

// multiconst: fx_time_t
export const FX_TIME_INFINITE: fx_time_t = 9_223_372_036_854_775_807n;
export const FX_TIME_INFINITE_PAST: fx_time_t = -9_223_372_036_854_775_808n;

// The value 0 is always considered to be an invalid handle.  In addition, the
// lowest two bits of a valid handle will _always_ be 1.  Users are free to
// store additional application specific information encoded in these lower two
// bits, provided that the are forced back to being set to 1 any time the handle
// is passed to a syscall.
//
// See https://fuchsia.dev/fuchsia-src/concepts/kernel/handles for more details.
export const FX_HANDLE_INVALID: fx_handle_t = 0;
export const FX_HANDLE_FIXED_BITS_MASK: fx_handle_t = 0x3;

// multiconst: fx_handle_op_t
export const FX_HANDLE_OP_MOVE: fx_handle_op_t = 0;
export const FX_HANDLE_OP_DUPLICATE: fx_handle_op_t = 1;

// multiconst: fx_koid_t
export const FX_KOID_INVALID: fx_koid_t = 0n;
export const FX_KOID_KERNEL: fx_koid_t = 1n;
export const FX_KOID_FIRST: fx_koid_t = 1024n;

// multiconst: fx_signals_t
export const FX_SIGNAL_NONE = 0;
export const FX_OBJECT_SIGNAL_ALL = 0x00ffffff;
export const FX_USER_SIGNAL_ALL = 0xff000000;
export const FX_OBJECT_SIGNAL_0 = 1 << 0;
export const FX_OBJECT_SIGNAL_1 = 1 << 1;
export const FX_OBJECT_SIGNAL_2 = 1 << 2;
export const FX_OBJECT_SIGNAL_3 = 1 << 3;
export const FX_OBJECT_SIGNAL_4 = 1 << 4;
export const FX_OBJECT_SIGNAL_5 = 1 << 5;
export const FX_OBJECT_SIGNAL_6 = 1 << 6;
export const FX_OBJECT_SIGNAL_7 = 1 << 7;
export const FX_OBJECT_SIGNAL_8 = 1 << 8;
export const FX_OBJECT_SIGNAL_9 = 1 << 9;
export const FX_OBJECT_HANDLE_CLOSED = 1 << 23;

export const FX_OBJECT_READABLE = FX_OBJECT_SIGNAL_0;
export const FX_OBJECT_WRITABLE = FX_OBJECT_SIGNAL_1;
export const FX_OBJECT_PEER_CLOSED = FX_OBJECT_SIGNAL_2;

// channel signals
export const FX_CHANNEL_READABLE = FX_OBJECT_SIGNAL_0;
export const FX_CHANNEL_WRITABLE = FX_OBJECT_SIGNAL_1;
export const FX_CHANNEL_PEER_CLOSED = FX_OBJECT_SIGNAL_2;

// process signals
export const FX_PROCESS_TERMINATED = FX_OBJECT_SIGNAL_3;

// cancelation (handle was closed while waiting with it)
export const FX_SIGNAL_HANDLE_CLOSED = FX_OBJECT_HANDLE_CLOSED;

// other contants
export const FX_MAX_NAME_LEN = 100;
export const FX_PROCESS_SHARED = 1;

export const FX_CHANNEL_READ_MAY_DISCARD = 0;
export const FX_CHANNEL_MAX_MSG_BYTES = 65536;
export const FX_CHANNEL_MAX_MSG_HANDLES = 64;

export const FX_RIGHT_NONE: fx_rights_t = 0;
export const FX_RIGHT_DUPLICATE: fx_rights_t = 1 << 0;
export const FX_RIGHT_TRANSFER: fx_rights_t = 1 << 1;
export const FX_RIGHT_READ: fx_rights_t = 1 << 2;
export const FX_RIGHT_WRITE: fx_rights_t = 1 << 3;
export const FX_RIGHT_EXECUTE: fx_rights_t = 1 << 4;
export const FX_RIGHT_MAP: fx_rights_t = 1 << 5;
export const FX_RIGHT_GET_PROPERTY: fx_rights_t = 1 << 6;
export const FX_RIGHT_SET_PROPERTY: fx_rights_t = 1 << 7;
export const FX_RIGHT_ENUMERATE: fx_rights_t = 1 << 8;
export const FX_RIGHT_DESTROY: fx_rights_t = 1 << 9;
export const FX_RIGHT_SET_POLICY: fx_rights_t = 1 << 10;
export const FX_RIGHT_GET_POLICY: fx_rights_t = 1 << 11;
export const FX_RIGHT_SIGNAL: fx_rights_t = 1 << 12;
export const FX_RIGHT_SIGNAL_PEER: fx_rights_t = 1 << 13;
export const FX_RIGHT_WAIT: fx_rights_t = 1 << 14;
export const FX_RIGHT_INSPECT: fx_rights_t = 1 << 15;
export const FX_RIGHT_MANAGE_JOB: fx_rights_t = 1 << 16;
export const FX_RIGHT_MANAGE_PROCESS: fx_rights_t = 1 << 17;
export const FX_RIGHT_MANAGE_THREAD: fx_rights_t = 1 << 18;
export const FX_RIGHT_APPLY_PROFILE: fx_rights_t = 1 << 19;
export const FX_RIGHT_MANAGE_SOCKET: fx_rights_t = 1 << 20;
export const FX_RIGHT_SAME_RIGHTS: fx_rights_t = 1 << 31;

// Convenient names for commonly grouped rights.
const FX_RIGHTS_BASIC = FX_RIGHT_TRANSFER | FX_RIGHT_DUPLICATE | FX_RIGHT_WAIT | FX_RIGHT_INSPECT;
const FX_RIGHTS_IO = FX_RIGHT_READ | FX_RIGHT_WRITE;

export const FX_DEFAULT_CHANNEL_RIGHTS =
    (FX_RIGHTS_BASIC & ~FX_RIGHT_DUPLICATE) | FX_RIGHTS_IO | FX_RIGHT_SIGNAL | FX_RIGHT_SIGNAL_PEER;

export const FX_DEFAULT_PROCESS_RIGHTS =
    FX_RIGHTS_BASIC |
    FX_RIGHTS_IO |
    // FX_RIGHTS_PROPERTY |
    FX_RIGHT_ENUMERATE |
    FX_RIGHT_DESTROY |
    FX_RIGHT_SIGNAL |
    FX_RIGHT_MANAGE_PROCESS |
    FX_RIGHT_MANAGE_THREAD;

export const FX_DEFAULT_JOB_RIGHTS =
    FX_RIGHTS_BASIC |
    FX_RIGHTS_IO |
    // FX_RIGHTS_PROPERTY |
    // FX_RIGHTS_POLICY |
    FX_RIGHT_ENUMERATE |
    FX_RIGHT_DESTROY |
    FX_RIGHT_SIGNAL |
    FX_RIGHT_MANAGE_JOB |
    FX_RIGHT_MANAGE_PROCESS |
    FX_RIGHT_MANAGE_THREAD;

export const FX_POL_NEW_PROCESS = 1;

export function todo() {
    throw new Error("not implemented");
}

export type fx_handle_disposition_t = {
    handle: fx_handle_t;
    result: fx_status_t;
    type: fx_handle_type_t;
    rights: fx_rights_t;
    operation: number;
};

export type fx_handle_info_t = {
    handle: fx_handle_t;
    type: fx_handle_type_t;
    rights: fx_rights_t;
};

export class fx_port_packet_t {
    public key!: u64;
    public status!: fx_status_t;
    public packet_type!: fx_packet_type_t;
    public union!: fx_packet_signal_t | fx_packet_user_t;

    serialize(): Uint8Array {
        const data = new Uint8Array(fx_port_packet_t.size);
        const writter = new DataView(data.buffer);

        writter.setBigUint64(0, this.key);
        writter.setUint32(8, this.status);
        writter.setUint32(12, this.packet_type);
        data.set(this.union.serialize(), 16);

        return data;
    }

    static get size() {
        return 16 + 32;
    }
}

export class fx_packet_user_t {
    data!: Uint8Array;

    serialize(): Uint8Array {
        return this.data.subarray(0, 32);
    }
}

export class fx_packet_signal_t {
    trigger!: fx_signals_t;
    observed!: fx_signals_t;
    count!: u64;
    timestamp!: u64;

    serialize(): Uint8Array {
        const data = new Uint8Array(32);
        const writter = new DataView(data.buffer);

        writter.setUint32(0, this.trigger);
        writter.setUint32(4, this.observed);
        writter.setBigUint64(8, this.count);
        writter.setBigUint64(16, this.timestamp);
        writter.setBigUint64(24, 0n);

        return data;
    }
}

export type fx_packet_type_t = u32;

// packet types.  zx_port_packet_t::type
export const FX_PKT_TYPE_USER: fx_packet_type_t = 0x00;
export const FX_PKT_TYPE_SIGNAL_ONE: fx_packet_type_t = 0x01;
// 0x02 was previously used for "ZX_PKT_TYPE_SIGNAL_REP".
export const FX_PKT_TYPE_GUEST_BELL: fx_packet_type_t = 0x03;
export const FX_PKT_TYPE_GUEST_MEM: fx_packet_type_t = 0x04;
export const FX_PKT_TYPE_GUEST_IO: fx_packet_type_t = 0x05;
export const FX_PKT_TYPE_GUEST_VCPU: fx_packet_type_t = 0x06;
export const FX_PKT_TYPE_INTERRUPT: fx_packet_type_t = 0x07;
export const FX_PKT_TYPE_PAGE_REQUEST: fx_packet_type_t = 0x09;

export const FX_OK: fx_status_t = 0;
export const FX_ERR_INTERNAL = -1;
export const FX_ERR_NOT_SUPPORTED = -2;
export const FX_ERR_NO_RESOURCES = -3;
export const FX_ERR_NO_MEMORY = -4;
export const FX_ERR_INTERRUPTED_RETRY = -6;
export const FX_ERR_INVALID_ARGS = -10;
export const FX_ERR_BAD_HANDLE = -11;
export const FX_ERR_WRONG_TYPE = -12;
export const FX_ERR_BAD_SYSCALL = -13;
export const FX_ERR_OUT_OF_RANGE = -14;
export const FX_ERR_BUFFER_TOO_SMALL = -15;
export const FX_ERR_BAD_STATE = -20;
export const FX_ERR_TIMED_OUT = -21;
export const FX_ERR_SHOULD_WAIT = -22;
export const FX_ERR_CANCELED = -23;
export const FX_ERR_PEER_CLOSED = -24;
export const FX_ERR_NOT_FOUND = -25;
export const FX_ERR_ALREADY_EXISTS = -26;
export const FX_ERR_ALREADY_BOUND = -27;
export const FX_ERR_UNAVAILABLE = -28;
export const FX_ERR_ACCESS_DENIED = -30;
export const FX_ERR_IO = -40;
export const FX_ERR_IO_REFUSED = -41;
export const FX_ERR_IO_DATA_INTEGRITY = -42;
export const FX_ERR_IO_DATA_LOSS = -43;
export const FX_ERR_IO_NOT_PRESENT = -44;
export const FX_ERR_IO_OVERRUN = -45;
export const FX_ERR_IO_MISSED_DEADLINE = -46;
export const FX_ERR_IO_INVALID = -47;
export const FX_ERR_BAD_PATH = -50;
export const FX_ERR_NOT_DIR = -51;
export const FX_ERR_NOT_FILE = -52;
export const FX_ERR_FILE_BIG = -53;
export const FX_ERR_NO_SPACE = -54;
export const FX_ERR_NOT_EMPTY = -55;
export const FX_ERR_STOP = -60;
export const FX_ERR_NEXT = -61;
export const FX_ERR_ASYNC = -62;
export const FX_ERR_PROTOCOL_NOT_SUPPORTED = -70;
export const FX_ERR_ADDRESS_UNREACHABLE = -71;
export const FX_ERR_ADDRESS_IN_USE = -72;
export const FX_ERR_NOT_CONNECTED = -73;

export const FX_OBJ_TYPE_NONE = 0;
export const FX_OBJ_TYPE_CHANNEL = 1;
export const FX_OBJ_TYPE_PROCESS = 2;
export const FX_OBJ_TYPE_JOB = 3;
export const FX_OBJ_TYPE_PORT = 4;

export enum Status {
    OK = FX_OK,
    // ERR_UNSUPPORTED = FX_ERR_UNSUPPORTED,
    ERR_NOT_SUPPORTED = FX_ERR_NOT_SUPPORTED,
    ERR_CANCELED = FX_ERR_CANCELED,
    ERR_OUT_OF_RANGE = FX_ERR_OUT_OF_RANGE,
    ERR_BAD_STATE = FX_ERR_BAD_STATE,
    ERR_BUFFER_TOO_SMALL = FX_ERR_BUFFER_TOO_SMALL,
    ERR_BAD_HANDLE = FX_ERR_BAD_HANDLE,
    ERR_SHOULD_WAIT = FX_ERR_SHOULD_WAIT,
    ERR_PEER_CLOSED = FX_ERR_PEER_CLOSED,
    ERR_INVALID_ARGS = FX_ERR_INVALID_ARGS,
}

export class Ref<T> {
    ref: T | null;

    constructor(value: T) {
        this.ref = value;
    }

    get value(): T {
        if (this.ref === null) throw new Error("ref is empty");
        return this.ref;
    }

    set value(newValue: T) {
        this.ref = newValue;
    }

    take(): T | null {
        const value = this.ref;
        this.ref = null;
        return value;
    }
}
