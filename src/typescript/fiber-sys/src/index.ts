/* eslint-disable no-var */

import { Kernel } from '@meshx-org/fiber-kernel';
import {
    fx_handle_t,
    fx_status_t,
    fx_signals_t,
    fx_time_t,
    fx_port_packet_t,
    u32,
    u64,
    i64,
    Ref,
    fx_handle_info_t,
    fx_handle_disposition_t,
    fx_rights_t,
} from "@meshx-org/fiber-types";

type Ptr<T> = {
    pointee: T;
};

export interface System {
    sys_handle_duplicate(handle: fx_handle_t, rights: fx_rights_t, handle_out: Ref<fx_handle_t>): fx_status_t;
    sys_handle_close(handle: fx_handle_t): fx_status_t;

    /** Job operations */
    sys_job_create(parent_job: fx_handle_t, options: u32, job_out: Ref<fx_handle_t>): fx_status_t;

    /** Process operations */
    sys_process_create(
        parent: fx_handle_t,
        name: string,
        options: u32,
        proc_handle_out: Ref<fx_handle_t>,
        vmar_handle_out: Ref<fx_handle_t>
    ): fx_status_t;

    sys_process_start(handle: fx_handle_t, entry: string, arg1: fx_handle_t, arg2: fx_handle_t): fx_status_t;

    sys_process_exit(retcode: i64): void;

    /** Object operations */

    sys_port_create(port_out: Ref<fx_handle_t>): fx_status_t;
    sys_port_wait(handle: fx_handle_t, deadline: fx_time_t, packet: Ref<fx_port_packet_t>): Promise<fx_status_t>;

    /** Port operations */

    sys_object_wait_async(
        handle: fx_handle_t,
        port: fx_handle_t,
        key: u64,
        signals: fx_signals_t,
        options: u32
    ): fx_status_t;

    /** Channel operations */

    sys_channel_create(options: u32, out1: Ref<fx_handle_t>, out2: Ref<fx_handle_t>): fx_status_t;
    sys_channel_read(
        handle_value: fx_handle_t,
        options: u32,
        bytes: Ptr<Uint8Array>,
        handle_info: Ptr<fx_handle_t[]>,
        num_bytes: u32,
        num_handles: u32,
        actual_bytes: Ptr<u32>,
        actual_handles: Ptr<u32>
    ): fx_status_t;

    sys_channel_write(
        handle_value: fx_handle_t,
        options: u32,
        bytes: Uint8Array,
        handles: fx_handle_t[]
    ): fx_status_t;
}

declare global {
    var fiber: Kernel;

    var sys_handle_close: ((handle: fx_handle_t) => fx_status_t) | undefined;
    var sys_handle_duplicate: ((handle: fx_handle_t) => fx_status_t) | undefined;

    var sys_job_create: ((parent_job: fx_handle_t, options: u32, job_out: Ref<fx_handle_t>) => fx_status_t) | undefined;

    var sys_port_create: ((port_out: Ref<fx_handle_t>) => fx_status_t) | undefined;

    var sys_port_wait:
        | ((handle: fx_handle_t, deadline: fx_time_t, on_packet: (packet: fx_port_packet_t) => void) => fx_status_t)
        | undefined;

    var sys_object_wait_async:
        | ((handle: fx_handle_t, port: fx_handle_t, key: u64, signals: fx_signals_t, options: u32) => fx_status_t)
        | undefined;

    var sys_process_create:
        | ((
              parent: fx_handle_t,
              name: string,
              options: u32,
              proc_handle_out: Ref<fx_handle_t>,
              vmar_handle_out: Ref<fx_handle_t>
          ) => fx_status_t)
        | undefined;

    var sys_process_start: ((handle: fx_handle_t, entry: string, arg1: fx_handle_t) => fx_status_t) | undefined;
    var sys_process_exit: ((retcode: i64) => void) | undefined;
    var sys_channel_read:
        | ((handle: fx_handle_t, options: u32, num_bytes: u32, num_handles: u32) => fx_status_t)
        | undefined;
}

const sys: System | undefined = undefined;

export const initSys = (system: Kernel) => {
    self.fiber = system;
};

export function fx_job_create(parent_job: fx_handle_t, options: u32, job_out: Ref<fx_handle_t>): fx_status_t {
    if (self.sys_job_create) return self.sys_job_create(parent_job, options, job_out);
    else throw new Error("system is not initialized");
}

export function fx_handle_close(handle: fx_handle_t): fx_status_t {
    if (self.sys_handle_close) return self.sys_handle_close(handle);
    else if (sys) return sys.sys_handle_close(handle);
    else throw new Error("system is not initialized");
}

export function fx_port_create(port_out: Ref<fx_handle_t>): fx_status_t {
    if (self.sys_port_create) return self.sys_port_create(port_out);
    else if (sys) return sys.sys_port_create(port_out);
    else throw new Error("system is not initialized");
}

export function fx_object_wait_async(
    handle: fx_handle_t,
    port: fx_handle_t,
    key: u64,
    signals: fx_signals_t,
    options: u32
): fx_status_t {
    if (self.sys_object_wait_async) return self.sys_object_wait_async(handle, port, key, signals, options);
    else if (sys) return sys.sys_object_wait_async(handle, port, key, signals, options);
    else throw new Error("system is not initialized");
}

export function fx_process_create(
    parent: fx_handle_t,
    name: string,
    options: u32,
    proc_handle_out: Ref<fx_handle_t>,
    vmar_handle_out: Ref<fx_handle_t>
): fx_status_t {
    if (self.sys_process_create)
        return self.sys_process_create(parent, name, options, proc_handle_out, vmar_handle_out);
    else if (sys) return sys.sys_process_create(parent, name, options, proc_handle_out, vmar_handle_out);
    else throw new Error("system is not initialized");
}

export function fx_channel_read(
    handle_value: fx_handle_t,
    options: u32,
    bytes: Ptr<Uint8Array>,
    handle_info: Ptr<fx_handle_t[]>,
    num_bytes: u32,
    num_handles: u32,
    actual_bytes: Ptr<u32>,
    actual_handles: Ptr<u32>
): fx_status_t {
    return self.fiber.sys_channel_read(
        handle_value,
        options,
        bytes,
        handle_info,
        num_bytes,
        num_handles,
        actual_bytes,
        actual_handles
    );
}

export function fx_channel_read_etc(
    handle: fx_handle_t,
    options: u32,
    bytes: ArrayBuffer | null,
    handleInfos: fx_handle_info_t[] | null,
    num_bytes: u32,
    num_handles: u32,
    actual_bytes: Ref<u32>,
    actual_handles: Ref<u32>
): fx_status_t {
    return 0;
}

export function fx_channel_write_etc(
    handle: fx_handle_t,
    options: u32,
    bytes: ArrayBuffer,
    num_bytes: u32,
    handles: fx_handle_disposition_t[],
    num_handles: u32
): fx_status_t {
    return 0;
}

export function fx_channel_create(options: u32, out1: Ref<fx_handle_t>, out2: Ref<fx_handle_t>): fx_status_t {
    return self.fiber.sys_channel_create(options, out1, out2);
}
