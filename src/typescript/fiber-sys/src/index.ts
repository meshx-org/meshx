/* eslint-disable no-var */

import {
    fx_handle_t,
    fx_vaddr_t,
    fx_status_t,
    fx_signals_t,
    fx_time_t,
    fx_port_packet_t,
    u32,
    u64,
    i64,
    Ref,
} from "@meshx-org/fiber-types"

export interface System {
    sys_handle_close(handle: fx_handle_t): fx_status_t

    /** Job operations */
    sys_job_create(
        parent_job: fx_handle_t,
        options: u32,
        job_out: Ref<fx_handle_t>
    ): fx_status_t

    /** Process operations */
    sys_process_create(
        parent: fx_handle_t,
        name: Uint8Array,
        name_size: u32,
        options: u32,
        proc_handle_out: Ref<fx_handle_t>,
        vmar_handle_out: Ref<fx_handle_t>
    ): fx_status_t

    sys_process_start(
        handle: fx_handle_t,
        entry: fx_vaddr_t,
        arg1: fx_handle_t
    ): fx_status_t

    sys_process_exit(retcode: i64): void

    /** Object operations */

    sys_port_create(port_out: Ref<fx_handle_t>): fx_status_t
    sys_port_wait(
        handle: fx_handle_t,
        deadline: fx_time_t,
        on_packet: (packet: fx_port_packet_t) => void
    ): fx_status_t

    /** Port operations */

    sys_object_wait_async(
        handle: fx_handle_t,
        port: fx_handle_t,
        key: u64,
        signals: fx_signals_t,
        options: u32
    ): fx_status_t

    /** Channel operations */

    sys_channel_create(
        out1: Ref<fx_handle_t>,
        out2: Ref<fx_handle_t>
    ): fx_status_t
}

declare global {
    var sys_handle_close: ((handle: fx_handle_t) => fx_status_t) | undefined
    var sys_job_create:
        | ((
              parent_job: fx_handle_t,
              options: u32,
              job_out: Ref<fx_handle_t>
          ) => fx_status_t)
        | undefined

    var sys_port_create:
        | ((port_out: Ref<fx_handle_t>) => fx_status_t)
        | undefined

    var sys_port_wait:
        | ((
              handle: fx_handle_t,
              deadline: fx_time_t,
              on_packet: (packet: fx_port_packet_t) => void
          ) => fx_status_t)
        | undefined

    var sys_object_wait_async:
        | ((
              handle: fx_handle_t,
              port: fx_handle_t,
              key: u64,
              signals: fx_signals_t,
              options: u32
          ) => fx_status_t)
        | undefined

    var sys_process_create:
        | ((
              parent: fx_handle_t,
              name: Uint8Array,
              name_size: u32,
              options: u32,
              proc_handle_out: Ref<fx_handle_t>,
              vmar_handle_out: Ref<fx_handle_t>
          ) => fx_status_t)
        | undefined

    var sys_process_start:
        | ((
              handle: fx_handle_t,
              entry: fx_vaddr_t,
              arg1: fx_handle_t
          ) => fx_status_t)
        | undefined

    var sys_process_exit: ((retcode: i64) => void) | undefined
    var sys_channel_create:
        | ((out1: Ref<fx_handle_t>, out2: Ref<fx_handle_t>) => fx_status_t)
        | undefined
}

let sys: System | undefined = undefined

const init = (system: System) => (sys = system)

function fx_job_create(
    parent_job: fx_handle_t,
    options: u32,
    job_out: Ref<fx_handle_t>
): fx_status_t {
    if (self.sys_job_create)
        return self.sys_job_create(parent_job, options, job_out)
    else throw new Error("system is not initialized")
}

export function fx_handle_close(handle: fx_handle_t): fx_status_t {
    if (self.sys_handle_close) return self.sys_handle_close(handle)
    else if (sys) return sys.sys_handle_close(handle)
    else throw new Error("system is not initialized")
}

export function fx_port_create(port_out: Ref<fx_handle_t>): fx_status_t {
    if (self.sys_port_create) return self.sys_port_create(port_out)
    else if (sys) return sys.sys_port_create(port_out)
    else throw new Error("system is not initialized")
}

export function fx_port_wait(
    handle: fx_handle_t,
    deadline: fx_time_t,
    on_packet: (packet: fx_port_packet_t) => void
): fx_status_t {
    if (self.sys_port_wait)
        return self.sys_port_wait(handle, deadline, on_packet)
    else if (sys) return sys.sys_port_wait(handle, deadline, on_packet)
    else throw new Error("system is not initialized")
}

export function fx_object_wait_async(
    handle: fx_handle_t,
    port: fx_handle_t,
    key: u64,
    signals: fx_signals_t,
    options: u32
): fx_status_t {
    if (self.sys_object_wait_async)
        return self.sys_object_wait_async(handle, port, key, signals, options)
    else if (sys)
        return sys.sys_object_wait_async(handle, port, key, signals, options)
    else throw new Error("system is not initialized")
}

export function fx_process_create(
    parent: fx_handle_t,
    name: Uint8Array,
    name_size: u32,
    options: u32,
    proc_handle_out: Ref<fx_handle_t>,
    vmar_handle_out: Ref<fx_handle_t>
): fx_status_t {
    if (self.sys_process_create)
        return self.sys_process_create(
            parent,
            name,
            name_size,
            options,
            proc_handle_out,
            vmar_handle_out
        )
    else if (sys)
        return sys.sys_process_create(
            parent,
            name,
            name_size,
            options,
            proc_handle_out,
            vmar_handle_out
        )
    else throw new Error("system is not initialized")
}

export function fx_process_start(
    handle: fx_handle_t,
    entry: fx_vaddr_t,
    arg1: fx_handle_t
): fx_status_t {
    if (self.sys_process_start)
        return self.sys_process_start(handle, entry, arg1)
    else if (sys) return sys.sys_process_start(handle, entry, arg1)
    else throw new Error("system is not initialized")
}

export function fx_process_exit(retcode: i64): void {
    if (self.sys_process_exit) return self.sys_process_exit(retcode)
    else if (sys) return sys.sys_process_exit(retcode)
    else throw new Error("system is not initialized")
}

export function fx_channel_create(
    out1: Ref<fx_handle_t>,
    out2: Ref<fx_handle_t>
): fx_status_t {
    if (self.sys_channel_create) return self.sys_channel_create(out1, out2)
    else if (sys) return sys.sys_channel_create(out1, out2)
    else throw new Error("system is not initialized")
}

export { init, fx_job_create }
