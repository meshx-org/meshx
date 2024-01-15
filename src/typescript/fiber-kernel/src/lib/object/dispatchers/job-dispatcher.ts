import { fx_rights_t } from "@meshx-org/fiber-types"
import { SoloDispatcher } from "./dispatcher"
import { ProcessDispatcher } from "./process-dispatcher"

// The starting max_height value of the root job.
const ROOT_JOB_MAX_HEIGHT = 32
const ROOT_JOB_NAME = "root"

export class JobPolicy {
    static create_root_policy() {
        return new JobPolicy()
    }

    get [Symbol.toStringTag]() {
        return "JobPolicy"
    }
}

enum State {
    READY,
    KILLING,
    DEAD,
}

type GuardedState = {
    // Access to the pointers in these lists, especially any promotions to
    // RefPtr, must be handled very carefully, because the children can die
    // even when |lock_| is held. See ForEachChildInLocked() for more details
    // and for a safe way to enumerate them.
    _jobs: Array<JobDispatcher>
    _procs: Array<ProcessDispatcher>
}

export class JobDispatcher extends SoloDispatcher {
    // The user-friendly job name. For debug purposes only. That
    // is, there is no mechanism to mint a handle to a job via this name.
    _name: string
    _parent_job: JobDispatcher | null
    _max_height: number
    _policy: JobPolicy
    _return_code: number
    _kill_on_oom: boolean
    _state: State
    _guarded: GuardedState

    private constructor(flags: number, parent: JobDispatcher | null, policy: JobPolicy) {
        super()

        this._parent_job = parent
        this._max_height = parent ? parent.max_height() - 1 : ROOT_JOB_MAX_HEIGHT

        this._name = ""
        this._state = State.READY
        this._return_code = 0
        this._kill_on_oom = false
        this._policy = policy
        this._guarded = { _jobs: [], _procs: [] }

        // kcounter_add(dispatcher_job_create_count, 1);
    }

    public static create_root_job(): JobDispatcher {
        const job = new JobDispatcher(0, null, JobPolicy.create_root_policy())
        // TODO: job.set_name("root")
        return job
    }

    parent(): JobDispatcher | null {
        return this._parent_job
    }

    public max_height(): number {
        return this._max_height
    }

    get_policy(): JobPolicy {
        // Guard<Mutex> guard{ get_lock() };
        return this._policy
    }

    public add_child_job(job: JobDispatcher): boolean {
        //canary_.Assert();
        //Guard<Mutex> guard{get_lock()};
        const guarded_state = this._guarded

        if (this._state != State.READY) {
            return false
        }

        // Put the new job after our next-youngest child, or us if we have none.
        //
        // We try to make older jobs closer to the root (both hierarchically and
        // temporally) show up earlier in enumeration.
        // TODO: JobDispatcher* neighbor = if self.jobs_.is_empty() { this } else { &self.jobs_.back() };

        // This can only be called once, the job should not already be part
        // of any job tree.
        // DEBUG_ASSERT(!fbl::InContainer<JobDispatcher::RawListTag>(*job));
        // DEBUG_ASSERT(neighbor != job.get());

        guarded_state._jobs.push(job)

        // TODO: UpdateSignalsLocked();
        return true
    }

    public add_child_process(process: ProcessDispatcher): boolean {
        //canary_.Assert();
        const guarded_state = this._guarded

        if (this._state != State.READY) {
            return false
        }

        guarded_state._procs.push(process)

        // TODO:  UpdateSignalsLocked();
        return true
    }

    static override default_rights(): fx_rights_t {
        return 0
    }
}
