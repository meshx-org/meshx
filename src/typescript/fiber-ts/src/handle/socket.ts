/// Typed wrapper around a socket objects

import { Handle } from './handle';
import { HandleWrapper } from './handle-wrapper';

/// fx_socket_create() syscall used to create them.
export class Socket extends HandleWrapper {
    constructor(handle: Handle | null) {
        super(handle)
    }
}