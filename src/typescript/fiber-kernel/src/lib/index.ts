/* eslint-disable @typescript-eslint/no-unused-vars */

import { System } from "@meshx-org/fiber-sys"
import { Ref, fx_handle_t, fx_port_packet_t, fx_rights_t, fx_status_t } from "@meshx-org/fiber-types"
import { JobDispatcher } from "./object/job-dispatcher"
import { Handle, HandleOwner, KernelHandle } from "./object/handle"
import { userboot_init } from "./userboot"
import invariant from "tiny-invariant"

export class Kernel implements System {
    private _boot_process: null
    private _root_job: null | JobDispatcher
    private _root_job_handle: null | HandleOwner
    private _root_job_observer: null

    constructor() {
        this._boot_process = null
        this._root_job = null
        this._root_job_handle = null
        this._root_job_observer = null
    }

    public init() {
        // Create root job.
        const root_job = JobDispatcher.create_root_job()
        this._root_job = root_job

        // Create handle.
        this._root_job_handle = Handle.make(new KernelHandle(root_job), JobDispatcher.default_rights())

        invariant(this._root_job_handle !== null)
    }

    public async start() {
        userboot_init(this)

        console.info("Now wait until the root job is childless.")
        console.log("Hello world")
    }

    public get_root_job_dispatcher() {
        invariant(this._root_job !== null)
        return this._root_job!
    }

    public get_root_job_handle(): HandleOwner {
        invariant(this._root_job !== null)
        return this._root_job_handle!
    }

    public start_root_job_observer() {
        //TODO
    }

    sys_handle_close(handle: fx_handle_t): fx_status_t {
        return 0
    }

    sys_job_create(parent_job: fx_handle_t, options: number, job_out: Ref<fx_handle_t>): number {
        throw new Error("Method not implemented.")
    }

    sys_process_create(
        parent: fx_handle_t,
        name: Uint8Array,
        name_size: number,
        options: number,
        proc_handle_out: Ref<fx_handle_t>,
        vmar_handle_out: Ref<fx_handle_t>
    ): number {
        throw new Error("Method not implemented.")
    }

    sys_process_start(handle: number, entry: bigint, arg1: number): number {
        return 0
    }

    sys_process_exit(retcode: bigint): void {
        return
    }

    sys_port_create(port_out: Ref<number>): number {
        throw new Error("Method not implemented.")
    }

    sys_port_wait(handle: number, deadline: bigint, on_packet: (packet: fx_port_packet_t) => void): number {
        throw new Error("Method not implemented.")
    }

    sys_object_wait_async(handle: number, port: number, key: bigint, signals: number, options: number): number {
        throw new Error("Method not implemented.")
    }

    sys_channel_create(out1: Ref<number>, out2: Ref<number>): number {
        throw new Error("Method not implemented.")
    }
}
