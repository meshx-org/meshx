import { Handle, HandleOwner } from "./object"
import { JobDispatcher } from "./object/dispatchers"

export class RootJobObserver {
    #root_job
    #root_job_handle

    // Create a RootJobObserver that halts the system when the root job terminates
    // (i.e. asserts ZX_JOB_NO_CHILDREN).
    constructor(root_job: JobDispatcher, root_job_handle: HandleOwner) {
        this.#root_job = root_job
        this.#root_job_handle = root_job_handle
    }
}
