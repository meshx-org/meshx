import { FX_OBJ_TYPE_PORT, fx_obj_type_t, fx_rights_t, fx_status_t } from "@meshx-org/fiber-types"
import { SoloDispatcher } from "./dispatcher"
import { KernelHandle } from "../handle"
import { Err, Ok, Result } from "../../std"

export class PortDispatcher extends SoloDispatcher {
    #options
    #zero_handles = false
    #num_ephemeral_packets = 0

    override get_type(): fx_obj_type_t {
        return FX_OBJ_TYPE_PORT
    }

    constructor(options: number) {
        super()
        this.#options = options
        //kcounter_add(dispatcher_port_create_count, 1)
    }

    static create(options: number): Result<[KernelHandle<PortDispatcher>, fx_rights_t], fx_status_t> {
        //if (options && options != FX_PORT_BIND_TO_INTERRUPT) {
        //    return FX_ERR_INVALID_ARGS;
        // }

        const new_handle = new KernelHandle(new PortDispatcher(options))

        const rights = PortDispatcher.default_rights()
        const handle = new_handle
        return Ok([handle, rights])
    }
}
