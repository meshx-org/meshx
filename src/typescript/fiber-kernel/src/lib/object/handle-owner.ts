import { Handle } from "./handle"

// HandleOwner wraps a Handle in an Arc that has shared
// ownership of the Handle and deletes it whenever it falls out of scope.
export class HandleOwner {
    constructor(public handle: Handle) {}

    dispatcher() {
        return this.handle.dispatcher()
    }

    base_value() {
        return this.handle.base_value()
    }

    set_handle_table_id(v: bigint) {
        return this.handle.set_handle_table_id(v)
    }
}
