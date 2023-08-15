import { FX_ERR_BAD_STATE, FX_OK, fx_handle_t, fx_rights_t, fx_status_t } from "@meshx-org/fiber-types"
import { SoloDispatcher } from "./dispatcher"
import { HandleTable } from "./handle-table"
import assert from "assert"
import { Err, Ok, Result } from "../std"
import { KernelHandle } from "./handle"
import { JobDispatcher, JobPolicy } from "./job-dispatcher"

// state of the process
enum State {
    INITIAL, // initial state, no thread present in process
    RUNNING, // first thread has started and is running
    DYING, // process has delivered kill signal to all threads
    DEAD, // all threads have entered DEAD state and potentially dropped refs on process
}

export class ProcessDispatcher extends SoloDispatcher {
    private _handle_table!: HandleTable
    private _name: string
    private _job: JobDispatcher
    private _policy: JobPolicy
    private _state: State

    private constructor(job: JobDispatcher, name: string, flags: number) {
        super()

        console.debug("ProcessDispatcher::new({:?})", name)

        this._job = job
        this._policy = job.get_policy()

        this._name = name
        this._state = State.INITIAL
    }

    public static create(
        parent_job: JobDispatcher,
        name: string,
        flags: number
    ): Result<[KernelHandle<ProcessDispatcher>, fx_rights_t], fx_status_t> {
        const process_dispatcher = new ProcessDispatcher(parent_job, name, flags)
        const handle_table = new HandleTable(process_dispatcher)
        process_dispatcher._handle_table = handle_table

        const handle = new KernelHandle(process_dispatcher)

        const status = process_dispatcher.init()
        if (status != FX_OK) {
            return Err(status)
        }

        // Only now that the process has been fully created and initialized can we register it with its
        // parent job. We don't want anyone to see it in a partially initalized state.
        if (!parent_job.add_child_process(process_dispatcher)) {
            return Err(FX_ERR_BAD_STATE)
        }

        return Ok([handle, ProcessDispatcher.default_rights()])
    }

    init(): fx_status_t {
        //Guard<Mutex> guard{get_lock()};
        assert.equal(this._state, State.INITIAL)

        // create an address space for this process, named after the process's koid.
        //let aspace_name: [u8; ZX_MAX_NAME_LEN] = format!("proc:{}", self.get_koid()).into();

        //let aspace_ = VmAspace::Create(VmAspace::TYPE_USER, aspace_name);

        //if (!aspace_) {
        //  trace!("error creating address space\n");
        //  return sys::FX_ERR_NO_MEMORY;
        //}

        return FX_OK
    }

    // Start this process running with the provided entry state, only
    // valid to be called on a thread in the INITIALIZED state that has not yet been started. If
    // `ensure_initial_thread` is true, the thread will only start if it is the first thread in the
    // process.
    public start(entry: (arg1: fx_handle_t, arg2: fx_handle_t) => void, arg1: fx_handle_t, arg2: fx_handle_t) {
        console.debug("ProcessDispatcher::start({:?}, {:?})", entry, self.name)

        //const stack = new OneMbStack.unwrap()

        /*const task = new Generator(stack, (yielder, input) => {
            const context = {
                process: self,
                yielder: yielder,
            }

            // Make sure to save the guard, see documentation for more information
            const _guard = new ScopeGuard(context)

            entry(arg1, input)
        })*/

        entry(arg1, 0)

        return FX_OK
    }

    public handle_table(): HandleTable {
        return this._handle_table
    }
}
