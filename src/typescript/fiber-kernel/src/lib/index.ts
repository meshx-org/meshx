/* eslint-disable @typescript-eslint/no-unused-vars */

import { System } from "@meshx-org/fiber-sys"
import { Ref, fx_handle_t, fx_port_packet_t, fx_status_t } from "@meshx-org/fiber-types"
import { JobDispatcher } from "./object/dispatchers"
import { Handle, HandleOwner, KernelHandle } from "./object"
import { userboot_init } from "./userboot"
import invariant from "tiny-invariant"

export type IWrite = {
    postMessage: any
}

export type IRead = {
    addEventListener: (event: "message", handler: any) => void
}

export class Kernel implements System {
    private processes: Array<string> = []

    private parent: null | IWrite = null
    private broadcast: Map<string, IWrite> = new Map()

    private isBooted = false

    private _boot_process: null
    private _root_job: null | JobDispatcher
    private _root_job_handle: null | HandleOwner
    private _root_job_observer: null

    constructor(private kid: string, private isLeader: boolean) {
        this._boot_process = null
        this._root_job = null
        this._root_job_handle = null
        this._root_job_observer = null
    }

    public dial(target: IWrite & IRead) {
        if (!this.isBooted) throw new Error("only booted kernels can dial")
        if (!this.isLeader) throw new Error("only leaders can dial")

        target.addEventListener("message", (event: any) => this.onLeaderMessage.call(this, event))
        this.broadcast.set("", target)
    }

    public listen(listener: IWrite & IRead) {
        if (!this.isBooted) throw new Error("only booted kernels can listen")
        if (this.isLeader) throw new Error("only followers can listen")

        listener.addEventListener("message", (event: any) => this.onFollowerMessage.call(this, event))
        this.parent = listener
    }

    private onFollowerMessage(event: { data: { type: string; pid: string; kid: string } }) {
        if (event.data.type === "newProcess") {
            this.processes.push(event.data.kid + ":" + event.data.pid)
        }
    }

    private onLeaderMessage(event: { data: { type: string; pid: string; kid: string } }) {
        if (event.data.type == "newProcess") {
            this.processes.push(event.data.kid + ":" + event.data.pid)

            for (const [, target] of this.broadcast) {
                target.postMessage({ type: "newProcess", pid: event.data.pid, kid: event.data.kid })
            }
        }
    }

    public unstable_newProcess(pid: string) {
        if (this.isLeader) {
            this.processes.push(this.kid + ":" + pid)

            for (const [, target] of this.broadcast) {
                target.postMessage({ type: "newProcess", pid, kid: this.kid })
            }
        } else {
            this.parent?.postMessage({ type: "newProcess", pid, kid: this.kid })
        }
    }

    public init() {
        // Create root job.
        const root_job = JobDispatcher.create_root_job()
        this._root_job = root_job

        // Create handle.
        this._root_job_handle = Handle.make(new KernelHandle(root_job), JobDispatcher.default_rights())

        invariant(this._root_job_handle !== null)
    }

    public async boot() {
        userboot_init(this)

        console.info("Now wait until the root job is childless.")
        console.log("Hello world")

        this.isBooted = true
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
        name: string,
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
