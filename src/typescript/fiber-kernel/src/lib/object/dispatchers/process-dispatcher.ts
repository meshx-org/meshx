import invariant from "tiny-invariant";
import {
    FX_ERR_BAD_STATE,
    FX_OBJ_TYPE_PROCESS,
    FX_ERR_NO_MEMORY,
    FX_OK,
    Ref,
    fx_handle_t,
    fx_obj_type_t,
    fx_rights_t,
    fx_status_t,
    u32,
    fx_port_packet_t,
    fx_signals_t,
    fx_time_t,
    i64,
    u64,
    FX_DEFAULT_PROCESS_RIGHTS,
} from "@meshx-org/fiber-types";
import { Dispatcher, SoloDispatcher } from "./dispatcher";
import { HandleTable } from "../handle-table";
import { Err, Ok, Result, user_ptr } from "../../std";
import { Handle, KernelHandle } from "../handle";
import { JobDispatcher, JobPolicy } from "./job-dispatcher";

import { processes } from "../kernel-processes";
import { instantiateStreaming } from "../asyncify";
import * as fx from "@meshx-org/fiber-ts";

// state of the process
enum State {
    INITIAL, // initial state, no thread present in process
    RUNNING, // first thread has started and is running
    DYING, // process has delivered kill signal to all threads
    DEAD, // all threads have entered DEAD state and potentially dropped refs on process
}

interface ProcessInstance {
    initialize(): Promise<void>;
    run(entry: string, arg1: fx_handle_t): Promise<u32>;
}

type usize = number;

class WasmProcess implements ProcessInstance {
    #instance: WebAssembly.Instance | null = null;
    #module: WebAssembly.Module | null = null;

    async initialize() {
        const decoder = new TextDecoder();

        const response = await fetch("/output.wasm", {
            method: "GET",
        });

        const memory = new WebAssembly.Memory({ initial: 20 });

        const instance = await instantiateStreaming(response, {
            env: { memory },
            fiber: {
                fx_debug(data_ptr: usize, data_len: u32) {
                    const data = new Uint8Array(memory.buffer, data_ptr, data_len);
                    console.log(new TextDecoder("utf-8").decode(data));
                },
                fx_handle_duplicate: function (
                    handle: fx_handle_t,
                    rights: fx_rights_t,
                    handle_ptr: fx_handle_t
                ): fx_status_t {
                    throw new Error("Function not implemented.");
                },
                fx_handle_close(handle: fx_handle_t): fx_status_t {
                    throw new Error("Function not implemented.");
                },
                fx_job_create(parent_job: fx_handle_t, options: u32, job_ptr: fx_handle_t): fx_status_t {
                    throw new Error("Function not implemented.");
                },
                fx_process_create(
                    parent: fx_handle_t,
                    name_ptr: u32,
                    name_len: u32,
                    options: u32,
                    proc_handle_ptr: u32,
                    vmar_handle_ptr: u32
                ): fx_status_t {
                    const name_buf = new Uint8Array(memory.buffer, name_ptr, name_len);
                    const name = decoder.decode(name_buf);

                    const proc_handle_out = new Uint32Array(memory.buffer, proc_handle_ptr, 1);
                    const proc_handle = new Ref(proc_handle_out[0]);

                    const status = self.fiber.sys_process_create(parent, name, options, proc_handle);
                    proc_handle_out[0] = proc_handle.value;

                    return status;
                },
                fx_process_start: function (handle: fx_handle_t, entry: string, arg1: fx_handle_t): fx_status_t {
                    throw new Error("Function not implemented.");
                },
                fx_process_exit: function (retcode: i64): void {
                    throw new Error("Function not implemented.");
                },
                fx_port_create(port_ptr: usize): fx_status_t {
                    const port_out = new Uint32Array(memory.buffer, port_ptr, 1);

                    const port_handle = new Ref<fx_handle_t>(port_out[0]);
                    const status = self.fiber.sys_port_create(port_handle);
                    console.log("fx_port_create", status);
                    port_out[0] = port_handle.value;

                    return status;
                },
                async fx_port_wait(handle: fx_handle_t, deadline: fx_time_t, packet_ptr: usize): Promise<fx_status_t> {
                    const packet_data_out = new Uint8Array(memory.buffer, packet_ptr, fx_port_packet_t.size);

                    const packet = new Ref<fx_port_packet_t>({} as any);
                    const status = await self.fiber.sys_port_wait(handle, deadline, packet);

                    packet_data_out.set(packet.value.serialize());

                    return status;
                },
                fx_port_queue() {
                    return;
                },
                fx_object_wait_async: function (
                    handle: fx_handle_t,
                    port: fx_handle_t,
                    key: u64,
                    signals: fx_signals_t,
                    options: u32
                ): fx_status_t {
                    throw new Error("Function not implemented.");
                },
                fx_channel_create: function (options: u32, out1_ptr: fx_handle_t, out2_ptr: fx_handle_t): fx_status_t {
                    throw new Error("Function not implemented.");
                },
                fx_channel_read(
                    handle_value: fx_handle_t,
                    options: u32,
                    bytes_ptr: usize,
                    handle_info_ptr: usize,
                    num_bytes: u32,
                    num_handles: u32,
                    actual_bytes_ptr: usize,
                    actual_handles_ptr: usize
                ): fx_status_t {
                    const bytes_out = new Uint8Array(memory.buffer, bytes_ptr, num_bytes);
                    const handle_info_out = new Uint32Array(memory.buffer, handle_info_ptr, num_handles);
                    const actual_bytes_out = new Uint32Array(memory.buffer, actual_bytes_ptr, 1);
                    const actual_handles_out = new Uint32Array(memory.buffer, actual_handles_ptr, 1);

                    const bytes = { pointee: new Uint8Array(num_bytes) };
                    const handles: user_ptr<fx_handle_t[]> = { pointee: [] };
                    const actual_bytes = { pointee: 0 };
                    const actual_handles = { pointee: 0 };

                    const status = self.fiber.sys_channel_read(
                        handle_value,
                        options,
                        bytes,
                        handles,
                        num_bytes,
                        num_handles,
                        actual_bytes,
                        actual_handles
                    );

                    console.log(
                        "ch read",
                        handle_value,
                        options,
                        bytes_out,
                        handle_info_out,
                        bytes,
                        handles,
                        num_bytes,
                        num_handles,
                        actual_bytes,
                        actual_handles
                    );

                    actual_bytes_out[0] = actual_bytes.pointee;
                    actual_handles_out[0] = actual_handles.pointee;
                    bytes_out.set(bytes.pointee);

                    for (const [index, handle] of handles.pointee.entries()) {
                        handle_info_out[index] = handle;
                    }

                    return status;
                },
                fx_channel_write: function (handle: fx_handle_t): fx_status_t {
                    throw new Error("Function not implemented.");
                },
                fx_ticks_get(): bigint {
                    return 0n;
                },
            },
        });

        this.#instance = instance.instance;
    }

    run(entry: string, arg1: fx_handle_t) {
        if (this.#instance) {
            const entry_fn = this.#instance!.exports[entry] as (arg1: fx_handle_t) => Promise<u32>;
            return entry_fn(arg1);
        }

        return Promise.resolve(-1);
    }
}

class KernelProcess implements ProcessInstance {
    #fn_ref: ((arg1: fx.Channel) => Promise<u32>) | null = null;

    // eslint-disable-next-line @typescript-eslint/no-empty-function
    async initialize() {}

    run(entry: string, arg1: fx_handle_t) {
        try {
            this.#fn_ref = processes.get(entry) ?? null;
            return this.#fn_ref!(new fx.Channel(fx.Handle.from_raw(arg1)));
        } catch (error) {
            return Promise.resolve(-1);
        }
    }
}

const scope_stack: ProcessDispatcher[] = [];

export class ProcessDispatcher extends SoloDispatcher {
    #handle_table!: HandleTable;
    #name: string;
    #job: JobDispatcher;
    #policy: JobPolicy;
    #state: State;

    #instance: ProcessInstance;
    #process: Promise<u32> | null;

    private constructor(job: JobDispatcher, instance: ProcessInstance, name: string, flags: number) {
        super();

        this.#job = job;
        this.#policy = job.get_policy();

        this.#name = name;
        this.#instance = instance;
        this.#state = State.INITIAL;
        this.#process = null;
    }

    public static get_current(): ProcessDispatcher {
        const current = scope_stack.at(scope_stack.length - 1);
        invariant(current);
        return current;
    }

    // Creates a kernel process
    public static create_kernel(
        parent_job: JobDispatcher,
        name: string,
        flags: number
    ): Result<[KernelHandle<ProcessDispatcher>, fx_rights_t], fx_status_t> {
        const process_instance = new KernelProcess();
        const process_dispatcher = new ProcessDispatcher(parent_job, process_instance, name, flags);
        const handle_table = new HandleTable(process_dispatcher);
        process_dispatcher.#handle_table = handle_table;

        const handle = new KernelHandle(process_dispatcher);

        const status = process_dispatcher.init();
        if (status != FX_OK) {
            return Err(status);
        }

        // Only now that the process has been fully created and initialized can we register it with its
        // parent job. We don't want anyone to see it in a partially initalized state.
        if (!parent_job.add_child_process(process_dispatcher)) {
            return Err(FX_ERR_BAD_STATE);
        }

        return Ok([handle, ProcessDispatcher.default_rights()]);
    }

    // Creates a wasm process
    public static create(
        parent_job: JobDispatcher,
        name: string,
        flags: number
    ): Result<[KernelHandle<ProcessDispatcher>, fx_rights_t], fx_status_t> {
        const process_instance = new WasmProcess();
        const process_dispatcher = new ProcessDispatcher(parent_job, process_instance, name, flags);
        const handle_table = new HandleTable(process_dispatcher);
        process_dispatcher.#handle_table = handle_table;

        const handle = new KernelHandle(process_dispatcher);

        const status = process_dispatcher.init();
        if (status != FX_OK) {
            return Err(status);
        }

        // Only now that the process has been fully created and initialized can we register it with its
        // parent job. We don't want anyone to see it in a partially initalized state.
        if (!parent_job.add_child_process(process_dispatcher)) {
            return Err(FX_ERR_BAD_STATE);
        }

        return Ok([handle, ProcessDispatcher.default_rights()]);
    }

    init(): fx_status_t {
        //Guard<Mutex> guard{get_lock()};
        invariant(this.#state == State.INITIAL);

        // create an address space for this process, named after the process's koid.
        //let aspace_name: [u8; ZX_MAX_NAME_LEN] = format!("proc:{}", self.get_koid()).into();

        //let aspace_ = VmAspace::Create(VmAspace::TYPE_USER, aspace_name);

        //if (!aspace_) {
        //  trace!("error creating address space\n");
        //  return sys::FX_ERR_NO_MEMORY;
        //}

        return FX_OK;
    }

    // Start this process running with the provided entry state, only
    // valid to be called on a thread in the INITIALIZED state that has not yet been started. If
    // `ensure_initial_thread` is true, the thread will only start if it is the first thread in the
    // process.
    public start(entry: string, arg1: fx_handle_t, arg2: fx_handle_t) {
        this.#process = (async () => {
            await this.#instance.initialize();
            scope_stack.push(this);

            let result;
            try {
                result = await this.#instance.run(entry, arg1);
            } catch (e) {
                console.error(e);
                result = -1;
            }

            scope_stack.pop();
            return result;
        })();

        return FX_OK;
    }

    make_and_add_handle(
        from: Dispatcher | KernelHandle<Dispatcher>,
        rights: fx_rights_t,
        out: Ref<fx_handle_t>
    ): fx_status_t {
        let handle;

        if (from instanceof KernelHandle) {
            handle = Handle.make_khandle(from, rights);
        } else {
            handle = Handle.make_dispather(from, rights);
        }

        if (!handle) {
            return FX_ERR_NO_MEMORY;
        }

        out.value = this.#handle_table.map_handleowner_to_value(handle);
        this.#handle_table.add_handle(handle);

        return FX_OK;
    }

    public handle_table(): HandleTable {
        return this.#handle_table;
    }

    static override default_rights(): fx_rights_t {
        return FX_DEFAULT_PROCESS_RIGHTS;
    }

    override get_type(): fx_obj_type_t {
        return FX_OBJ_TYPE_PROCESS;
    }
}
