/* eslint-disable @typescript-eslint/no-unused-vars */

import { System } from "@meshx-org/fiber-sys";
import {
    FX_CHANNEL_READ_MAY_DISCARD,
    FX_ERR_ACCESS_DENIED,
    FX_ERR_BAD_HANDLE,
    FX_ERR_BUFFER_TOO_SMALL,
    FX_ERR_INVALID_ARGS,
    FX_ERR_NOT_SUPPORTED,
    FX_ERR_NO_MEMORY,
    FX_ERR_WRONG_TYPE,
    FX_HANDLE_INVALID,
    FX_HANDLE_OP_DUPLICATE,
    FX_HANDLE_OP_MOVE,
    FX_MAX_NAME_LEN,
    FX_OBJ_TYPE_NONE,
    FX_OK,
    FX_RIGHT_DUPLICATE,
    FX_RIGHT_MANAGE_PROCESS,
    FX_RIGHT_READ,
    FX_RIGHT_SAME_RIGHTS,
    FX_RIGHT_TRANSFER,
    FX_RIGHT_WRITE,
    fx_handle_disposition_t,
    fx_handle_info_t,
    fx_handle_op_t,
    fx_handle_t,
    fx_obj_type_t,
    fx_port_packet_t,
    fx_rights_t,
    fx_status_t,
    u32,
} from "@meshx-org/fiber-types";
import { ChannelDispatcher, Dispatcher, JobDispatcher, PortDispatcher, ProcessDispatcher } from "./object/dispatchers";
import { Handle, HandleOwner, KernelHandle, MAX_MESSAGE_HANDLES, MessagePacket } from "./object";
import { userboot_init } from "./userboot";
import invariant from "tiny-invariant";
import { RootJobObserver } from "./root_job_observer";
import { debug_invariant, Err, Ok, Ref, Result, user_ptr } from "./std";
import { HandleTable } from "./object/handle-table";

type HandleT = fx_handle_t | fx_handle_info_t;
type UserHandle = fx_handle_t | fx_handle_disposition_t;

function map_handle_to_value(up: ProcessDispatcher, handle: Handle, out: Ref<HandleT>): void {
    if (typeof out.value == "number") {
        out.value = up.handle_table().map_handle_to_value(handle);
    } else {
        out.value.handle = up.handle_table().map_handle_to_value(handle);
        out.value.type = handle.dispatcher().get_type();
        out.value.rights = handle.rights();
        // out.value.unused = 0
    }
}

// Removes the handles from |msg|, install them in |up|'s handle table, and copies them out to the
// user array |handles|.
//
// Upon completion, the Handle object will either be owned by the process (success) or closed
// (error).

function msg_get_handles(
    up: ProcessDispatcher,
    msg: MessagePacket,
    handles: user_ptr<HandleT[]>,
    num_handles: u32
): fx_status_t {
    const handle_list = msg.handles;

    const hvs: HandleT[] = [];

    for (let i = 0; i < num_handles; ++i) {
        const ref = new Ref(0);
        map_handle_to_value(up, handle_list[i], ref);
        hvs[i] = ref.value;
    }

    handles.pointee = hvs;

    // The MessagePacket currently owns the handle. Only after transferring the handles into this
    // process's handle table can we relieve MessagePacket of its handle ownership responsibility.
    for (let i = 0; i < num_handles; ++i) {
        /*if (handle_list[i].dispatcher().is_waitable()) {
            // Cancel any waiters on this handle prior to adding it to the process's handle table.
            handle_list[i].dispatcher().cancel(handle_list[i])
            // If this handle refers to a channel, cancel any channel_call waits.
            const channel = handle_list[i].dispatcher() as ChannelDispatcher
            if (channel) {
                channel.cancel_message_waiters()
            }
        }*/

        const handle = new HandleOwner(handle_list[i]);
        // TODO(https://fxbug.dev/42105832): This takes a lock per call. Consider doing these in a
        // batch.
        up.handle_table().add_handle(handle);
    }

    msg.set_owns_handles(false);
    return FX_OK;
}

// Basic checks for a |handle| to be able to be sent via |channel|.
function handle_checks_locked(
    handle: Handle | null,
    channel: Dispatcher,
    operation: fx_handle_op_t,
    desired_rights: fx_rights_t,
    type: fx_obj_type_t
): fx_status_t {
    if (!handle) return FX_ERR_BAD_HANDLE;
    if (!handle.has_rights(FX_RIGHT_TRANSFER)) return FX_ERR_ACCESS_DENIED;
    if (handle.dispatcher() == channel) return FX_ERR_NOT_SUPPORTED;
    if (type != FX_OBJ_TYPE_NONE && handle.dispatcher().get_type() !== type) return FX_ERR_WRONG_TYPE;
    if (operation != FX_HANDLE_OP_MOVE && operation != FX_HANDLE_OP_DUPLICATE) return FX_ERR_INVALID_ARGS;
    if (desired_rights != FX_RIGHT_SAME_RIGHTS) {
        if ((handle.rights() & desired_rights) != desired_rights) {
            return FX_ERR_INVALID_ARGS;
        }
    }
    if (operation == FX_HANDLE_OP_DUPLICATE && !handle.has_rights(FX_RIGHT_DUPLICATE)) {
        return FX_ERR_ACCESS_DENIED;
    }
    return FX_OK;
}

function get_handledisposition_for_message_locked(
    process: ProcessDispatcher,
    channel: Dispatcher,
    handle_disposition: fx_handle_disposition_t
): Result<HandleOwner, fx_status_t> {
    const source = process.handle_table().get_handle_locked(process, handle_disposition.handle);

    const operation: fx_handle_op_t = handle_disposition.operation;
    const desired_rights: fx_rights_t = handle_disposition.rights;
    const type: fx_obj_type_t = handle_disposition.type;

    const status = handle_checks_locked(source, channel, operation, desired_rights, type);
    if (status != FX_OK) {
        handle_disposition.result = status;
        return Err(status);
    }

    // This if() block is purely an optimization and can be removed without
    // the rest of the function having to change.
    if (operation == FX_HANDLE_OP_MOVE && desired_rights == FX_RIGHT_SAME_RIGHTS) {
        return Ok(process.handle_table().remove_handle_locked(source!));
    }

    // For the non-optimized case, we always need to create a new handle because
    // the rights are a const member of Handle.
    const dest_rights = desired_rights === FX_RIGHT_SAME_RIGHTS ? source!.rights() : desired_rights;

    const raw_handle = Handle.dup(source!, dest_rights);
    if (!raw_handle) {
        // It's possible for the dup operation to fail if we run out of handles exactly
        // at this point.
        return Err(FX_ERR_NO_MEMORY);
    }

    // Use !ZX_HANDLE_OP_DUPLICATE so that we handle the case where operation
    // is an invalid value.
    if (operation !== FX_HANDLE_OP_DUPLICATE) {
        process.handle_table().remove_handle_locked(source!);
    }
    return Ok(raw_handle);
}

// This overload is used by zx_channel_write.
function get_handle_for_message_locked(
    process: ProcessDispatcher,
    channel: Dispatcher,
    handle_val: fx_handle_t
): Result<HandleOwner, fx_status_t> {
    const source = process.handle_table().get_handle_locked(process, handle_val);

    const status = handle_checks_locked(source, channel, FX_HANDLE_OP_MOVE, FX_RIGHT_SAME_RIGHTS, FX_OBJ_TYPE_NONE);
    if (status != FX_OK) return Err(status);

    return Ok(process.handle_table().remove_handle_locked(source!));
}

// For zx_handle_write or zx_handle_write_etc with the ZX_HANDLE_OP_MOVE flag,
// handles are closed whether success or failure. For zx_handle_write_etc
// with the ZX_HANDLE_OP_DUPLICATE flag, handles always remain open.
function msg_put_handles(
    up: ProcessDispatcher,
    msg: MessagePacket,
    handles: UserHandle[],
    num_handles: u32,
    channel: Dispatcher
): fx_status_t {
    debug_invariant(num_handles <= MAX_MESSAGE_HANDLES); // This must be checked before calling.

    let status = FX_OK;
    for (let ix = 0; ix != num_handles; ++ix) {
        const handle = handles[ix];
        let inner_status;
        if (typeof handle === "number") {
            inner_status = get_handle_for_message_locked(up, channel, handle);
        } else {
            inner_status = get_handledisposition_for_message_locked(up, channel, handle);
        }

        if (!inner_status.is_ok() && status == FX_OK) {
            // Latch the first error encountered. It will be what the function returns.
            status = inner_status.unwrap_err();
        }

        msg.handles[ix] = inner_status.is_ok() ? inner_status.unwrap().handle : null;
    }

    msg.set_owns_handles(true);
    return status;
}

type NodeId = u32;

export class Kernel implements System {
    #root_job: JobDispatcher;
    #root_job_handle: HandleOwner;
    #root_job_observer: RootJobObserver | null;

    imports: Map<NodeId, HandleTable> | null;
    exports: Map<NodeId, HandleTable> | null;
    channels: Map<NodeId, KernelHandle<ChannelDispatcher>> | null;

    constructor() {
        this.imports = null;
        this.exports = null;
        this.channels = null
        
        this.#root_job_observer = null;

        // Create root job.
        const root_job = JobDispatcher.create_root_job();
        this.#root_job = root_job;

        // Create handle.
        this.#root_job_handle = Handle.make_khandle(new KernelHandle(root_job), JobDispatcher.default_rights());

        invariant(this.#root_job_handle !== null);
    }

    #channel_write(
        handle_value: fx_handle_t,
        options: u32,
        user_bytes: Uint8Array,
        user_handles: UserHandle[]
    ): fx_status_t {
        // LTRACEF("handle %x bytes %p num_bytes %u handles %p num_handles %u options 0x%x\n", handle_value, user_bytes.get(), num_bytes, user_handles.get(), num_handles, options);

        const up = ProcessDispatcher.get_current();
        const result = up.handle_table().get_dispatcher_with_rights(up, handle_value, FX_RIGHT_READ);

        //if ((options & ~FX_CHANNEL_WRITE_USE_IOVEC) !== 0) {
        //    return FX_ERR_INVALID_ARGS;
        //}

        if (result.is_err()) {
            return result.unwrap_err();
        }

        const dispatcher = result.unwrap();

        const channel = dispatcher as ChannelDispatcher;

        //if ((options & FX_CHANNEL_WRITE_USE_IOVEC) != 0) {
        //    status = MessagePacket::Create(user_bytes.reinterpret<const zx_channel_iovec_t>(), num_bytes, num_handles, &msg);
        //} else {
        const status = MessagePacket.create(user_bytes, user_bytes.byteLength, user_handles.length);
        //}
        if (status.is_err()) {
            return status.unwrap_err();
        }

        const msg = status.unwrap();

        if (user_handles.length > 0) {
            const status = msg_put_handles(up, msg, user_handles, user_handles.length, channel);
            if (status != FX_OK) return status;
        }

        //TraceMessage(msg, *channel, MessageOp::Write);

        const status2 = channel.write(up.handle_table().get_koid(), msg);
        if (status2 != FX_OK) return status2;

        return FX_OK;
    }

    #channel_read(
        handle_value: fx_handle_t,
        options: u32,
        bytes: user_ptr<Uint8Array>,
        handles: user_ptr<HandleT[]>,
        num_bytes: u32,
        num_handles: u32,
        actual_bytes?: user_ptr<u32>,
        actual_handles?: user_ptr<u32>
    ): fx_status_t {
        const up = ProcessDispatcher.get_current();
        const result = up.handle_table().get_dispatcher_with_rights(up, handle_value, FX_RIGHT_READ);

        if (result.is_err()) {
            return result.unwrap_err();
        }

        const dispatcher = result.unwrap();

        const channel = dispatcher as ChannelDispatcher;

        // Currently MAY_DISCARD is the only allowable option.
        if (options & ~FX_CHANNEL_READ_MAY_DISCARD) {
            return FX_ERR_NOT_SUPPORTED;
        }

        const msg_ref = new Ref<MessagePacket | null>(null);
        const num_bytes_ref = new Ref(num_bytes);
        const num_handles_ref = new Ref(num_handles);
        const read_result = channel.read(up.handle_table().get_koid(), num_bytes_ref, num_handles_ref, msg_ref, false);

        // On FX_ERR_BUFFER_TOO_SMALL, Read() gives us the size of the next message (which remains
        // unconsumed, unless |options| has FX_CHANNEL_READ_MAY_DISCARD set).
        if (actual_bytes) {
            actual_bytes.pointee = num_bytes_ref.value;
            //if (status !== FX_OK) return status
        }

        if (actual_handles) {
            actual_handles.pointee = num_handles_ref.value;
            //if (status !== FX_OK) return status
        }

        if (read_result === FX_ERR_BUFFER_TOO_SMALL) {
            return read_result;
        }

        // TraceMessage(msg, *channel, MessageOp::Read);

        if (num_bytes > 0) {
            if (msg_ref.value!.copy_data_to(bytes) !== FX_OK) {
                return FX_ERR_INVALID_ARGS;
            }
        }

        // The documented public API states that that writing to the handles buffer
        // must happen after writing to the data buffer.
        if (num_handles > 0) {
            const status = msg_get_handles(up, msg_ref.value!, handles, num_handles);
            if (status !== FX_OK) {
                return status;
            }
        }

        // record_recv_msg_sz(num_bytes)
        return read_result;
    }

    sys_channel_read(
        handle_value: fx_handle_t,
        options: u32,
        bytes: user_ptr<Uint8Array>,
        handle_info: user_ptr<fx_handle_t[]>,
        num_bytes: u32,
        num_handles: u32,
        actual_bytes: user_ptr<u32>,
        actual_handles: user_ptr<u32>
    ): fx_status_t {
        return this.#channel_read(
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

    sys_channel_write(
        handle_value: fx_handle_t,
        options: u32,
        user_bytes: Uint8Array,
        user_handles: fx_handle_t[]
    ): fx_status_t {
        /*console.trace(
            "handle %x bytes %p num_bytes %u handles %p num_handles %u options 0x%x\n",
            handle_value,
            user_bytes.get(),
            num_bytes,
            user_handles.get(),
            num_handles,
            options
        );*/

        return this.#channel_write(handle_value, options, user_bytes, user_handles);
    }

    #handle_dup_replace(
        is_replace: boolean,
        handle_value: fx_handle_t,
        rights: fx_rights_t,
        out: Ref<fx_handle_t>
    ): fx_status_t {
        //LTRACEF("handle %x\n", handle_value);

        const up = ProcessDispatcher.get_current();

        //AutoExpiringPreemptDisabler preempt_disable{Mutex::DEFAULT_TIMESLICE_EXTENSION};
        //Guard<BrwLockPi, BrwLockPi::Writer> guard{up.handle_table().get_lock()};

        const source = up.handle_table().get_handle_locked(up, handle_value);

        if (!source) {
            return FX_ERR_BAD_HANDLE;
        }

        if (!is_replace) {
            if (!source.has_rights(FX_RIGHT_DUPLICATE)) return FX_ERR_ACCESS_DENIED;
        }

        if (rights == FX_RIGHT_SAME_RIGHTS) {
            rights = source.rights();
        } else if ((source.rights() & rights) != rights) {
            if (is_replace) up.handle_table().remove_handle_locked(source);
            return FX_ERR_INVALID_ARGS;
        }

        const handle = Handle.dup(source, rights);
        if (!handle) {
            return FX_ERR_NO_MEMORY;
        }

        if (is_replace) {
            up.handle_table().remove_handle_locked(source);
        }

        console.log("dup", handle);

        out.value = up.handle_table().map_handleowner_to_value(handle);
        up.handle_table().add_handle_locked(handle);
        return FX_OK;
    }

    sys_handle_duplicate(handle_value: fx_handle_t, rights: fx_rights_t, handle_out: Ref<fx_handle_t>): fx_status_t {
        return this.#handle_dup_replace(false, handle_value, rights, handle_out);
    }

    public async boot() {
        userboot_init(this);
    }

    public get_root_job_dispatcher() {
        invariant(this.#root_job !== null);
        return this.#root_job;
    }

    public get_root_job_handle(): HandleOwner {
        invariant(this.#root_job_handle !== null);
        return this.#root_job_handle;
    }

    public start_root_job_observer() {
        invariant(this.#root_job_observer === null, "root_job_observer should be null");
        // TODO: debug_invariant
        invariant(this.#root_job !== null);

        this.#root_job_observer = new RootJobObserver(this.#root_job, this.#root_job_handle);

        // Initialize the memory watchdog.
        // memory_watchdog_.Init(this);
    }

    sys_handle_close(handle: fx_handle_t): fx_status_t {
        return 0;
    }

    sys_job_create(parent_job: fx_handle_t, options: number, job_out: Ref<fx_handle_t>): fx_status_t {
        throw new Error("Method not implemented.");
    }

    sys_process_create(
        job_handle: fx_handle_t,
        name_og: string,
        options: number,
        proc_handle_out: Ref<fx_handle_t>
        //vmar_handle_out: Ref<fx_handle_t>
    ): fx_status_t {
        // console.trace(`job handle ${job_handle}, options ${options}`);

        // currently, the only valid option values are 0 or ZX_PROCESS_SHARED
        if (/*options !== FX_PROCESS_SHARED &&*/ options !== 0) {
            return FX_ERR_INVALID_ARGS;
        }

        const up = ProcessDispatcher.get_current();

        // We check the policy against the process calling zx_process_create, which
        // is the operative policy, rather than against |job_handle|. Access to
        // |job_handle| is controlled by the rights associated with the handle.
        //let result: zx_status_t = up.EnforceBasicPolicy(ZX_POL_NEW_PROCESS);
        //        if (result != ZX_OK) {
        //    return result;
        //}

        // copy out the name
        // Silently truncate the given name.
        const name = name_og.substring(0, FX_MAX_NAME_LEN);

        const result1 = up.handle_table().get_dispatcher_with_rights(up, job_handle, FX_RIGHT_MANAGE_PROCESS);
        if (result1.is_err()) {
            return result1.unwrap_err();
        }

        const job = result1.unwrap() as JobDispatcher;

        // create a new process dispatcher
        //let new_process_handle: KernelHandle<ProcessDispatcher>
        // let new_vmar_handle: KernelHandle<VmAddressRegionDispatcher>;
        //let proc_rights, vmar_rights // zx_rights_t
        const result2 = ProcessDispatcher.create(
            job,
            name,
            options
            //new_vmar_handle,
            //vmar_rights
        );
        if (result2.is_err()) return result2.unwrap_err();

        const [new_process_handle, proc_rights] = result2.unwrap();

        //console.trace("name %s\n", buf);

        // KTRACE_KERNEL_OBJECT("kernel:meta", new_process_handle.dispatcher()->get_koid(), ZX_OBJ_TYPE_PROCESS, buf);

        const result3 = up.make_and_add_handle(new_process_handle, proc_rights, proc_handle_out);
        return result3;
    }

    sys_process_start(
        process_handle: fx_handle_t,
        entry: string,
        arg_handle_value: fx_handle_t,
        arg2: fx_handle_t
    ): fx_status_t {
        console.trace(
            "phandle %x, thandle %x, pc %#",
            ", sp %#",
            ", arg_handle %x, arg2 %#",
            process_handle,
            arg_handle_value,
            arg2
        );
        const up = ProcessDispatcher.get_current();

        // get process dispatcher
        const status = up.handle_table().get_dispatcher_with_rights(up, process_handle, FX_RIGHT_WRITE);
        if (status.is_err()) {
            if (arg_handle_value !== FX_HANDLE_INVALID) {
                up.handle_table().remove_handle(up, arg_handle_value);
            }
            return status.unwrap_err();
        }

        const process = status.unwrap() as ProcessDispatcher;

        let arg_handle: HandleOwner | null = null;
        if (arg_handle_value !== FX_HANDLE_INVALID) {
            arg_handle = up.handle_table().remove_handle(up, arg_handle_value);
        }

        let arg_nhv: fx_handle_t = FX_HANDLE_INVALID;
        if (arg_handle) {
            if (!arg_handle.has_rights(FX_RIGHT_TRANSFER)) {
                console.log("process2", FX_ERR_ACCESS_DENIED);
                return FX_ERR_ACCESS_DENIED;
            }

            arg_nhv = process.handle_table().map_handleowner_to_value(arg_handle);
            process.handle_table().add_handle(arg_handle);
        }

        const status2 = process.start(entry, arg_nhv, arg2);
        if (status2 !== FX_OK) {
            if (arg_nhv !== FX_HANDLE_INVALID) {
                // Remove |arg_handle| from the process that failed to start.
                process.handle_table().remove_handle(process, arg_nhv);
            }
            return status2;
        }

        return FX_OK;
    }

    sys_process_exit(retcode: bigint): void {
        return;
    }

    sys_port_create(port_out: Ref<fx_handle_t>): fx_status_t {
        //console.trace("options %u\n", options);
        const up = ProcessDispatcher.get_current();

        // TODO: enforce policy
        //let result = up.enforce_basic_policy(FX_POL_NEW_PORT);
        //if (result != ZX_OK) {
        //    return result;
        //}

        const result = PortDispatcher.create(0);
        if (result.is_err()) {
            return result.unwrap_err();
        }

        const [handle, rights] = result.unwrap();

        return up.make_and_add_handle(handle, rights, port_out);
    }

    async sys_port_wait(handle: fx_handle_t, deadline: bigint, packet: Ref<fx_port_packet_t>): Promise<fx_status_t> {
        throw new Error("Method not implemented.");
    }

    sys_object_wait_async(
        handle: fx_handle_t,
        port: number,
        key: bigint,
        signals: number,
        options: number
    ): fx_status_t {
        throw new Error("Method not implemented.");
    }

    sys_channel_create(options: u32, out0: Ref<fx_handle_t>, out1: Ref<fx_handle_t>): fx_status_t {
        if (options !== 0) {
            return FX_ERR_INVALID_ARGS;
        }

        const up = ProcessDispatcher.get_current();

        // TODO: enforce policy
        //const res: fx_status_t = up.enforce_basic_policy(FX_POL_NEW_CHANNEL)
        //if (res != FX_OK) return res

        const result0 = ChannelDispatcher.create();
        if (result0.is_err()) {
            return result0.unwrap_err();
        }

        const [handle0, handle1, rights] = result0.unwrap();

        const result1 = up.make_and_add_handle(handle0, rights, out0);
        let result2: fx_status_t;

        if (result1 === FX_OK) {
            result2 = up.make_and_add_handle(handle1, rights, out1);
            return result2;
        }

        return result1;
    }
}
