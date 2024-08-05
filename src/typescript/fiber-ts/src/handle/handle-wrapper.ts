import { fx_rights_t, fx_status_t } from "@meshx-org/fiber-types";
import { Handle } from "./handle";

/// A base class for TypedHandles.
export abstract class HandleWrapper {
    constructor(private _handle: Handle) {}

    get isValid() {
        return this.handle?.isValid ?? false;
    }

    get handle() {
        return this._handle;
    }

    public equals(other: HandleWrapper): boolean {
        return other instanceof HandleWrapper && this.handle == other.handle;
    }

    duplicate(rights: fx_rights_t): { status: fx_status_t; handle: Handle } {
        return this.handle.duplicate(rights);
    }

    close() {
        this._handle?.close();
        this._handle = Handle.invalid();
    }

    passHandle(): Handle | null {
        const result = this._handle;
        this._handle = Handle.invalid();
        return result;
    }

    toString(): string {
        return `HandleWrapper(${this.handle})`;
    }
}

/// A base class for classes that wrap a pair of TypedHandles.
export abstract class HandleWrapperPair<T> {
    public first: T;
    public second: T;

    protected constructor(first: T, second: T) {
        this.first = first;
        this.second = second;
    }
}
