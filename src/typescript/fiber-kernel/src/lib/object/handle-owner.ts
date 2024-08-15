import { fx_koid_t, fx_rights_t } from '@meshx-org/fiber-types'
import { Handle } from "./handle"

// HandleOwner wraps a Handle in an Arc that has shared
// ownership of the Handle and deletes it whenever it falls out of scope.
export class HandleOwner {
    constructor(public handle: Handle) {}

    dispatcher() {
        return this.handle.dispatcher()
    }

    index_self() {
        return this.handle.index_self()
    }

    set_handle_table_id(koid: fx_koid_t) {
        return this.handle.set_handle_table_id(koid)
    }
    
    has_rights(rights: fx_rights_t) {
        return this.handle.has_rights(rights)
    }
}
