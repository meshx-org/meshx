import { Status } from "@meshx-org/fiber-types"
import { Handle } from "./handle"

/// A base class for TypedHandles.
export abstract class HandleWrapper {
    constructor(private _handle: Handle | null) {}

    get isValid() {
        return this.handle?.isValid ?? false
    }

    get handle() {
        return this._handle
    }

    public equals(other: HandleWrapper): boolean {
        return other instanceof HandleWrapper && this.handle == other.handle
    }

    close(): void {
        this._handle?.close()
        this._handle = null
    }

    passHandle(): Handle | null {
        const result = this._handle
        this._handle = null
        return result
    }

    toString(): string {
        return `HandleWrapper(${this.handle})`
    }
}

/// A base class for classes that wrap a pair of TypedHandles.
export abstract class HandleWrapperPair<T> {
    private _first: T | null
    private _second: T | null
    private _status: Status | null = null

    get first(): T {
        if (!this._first) throw new Error("invalid first handle")
        return this._first
    }

    get second(): T {
        if (!this._second) throw new Error("invalid second handle")
        return this._second
    }

    get status(): Status {
        if (!this._status) throw new Error("missing status")
        return this._status
    }

    protected constructor(status: Status | null, first: T | null, second: T | null) {
        this._status = status
        this._first = first
        this._second = second
    }
}
