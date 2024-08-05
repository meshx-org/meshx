import invariant from "tiny-invariant"

/**Represents the result of a computation that can either succeed with a value of type T or fail with an error of type E.*/
export type Result<T, E> = Left<T, E> | Right<T, E>

/** Represents a failed computation.*/
export interface Left<T, E> {
    ok: false
    error: E
    /*** Returns the value of the Result if it is successful, otherwise throws an error.*/
    unwrap(): T
    unwrap_err(): E
    /*** Returns the value of the Result if it is successful, otherwise returns the provided default value.*/
    //unwrap_or(defaultValue: T): T
    /*** Returns the value of the Result if it is successful, otherwise calls the provided function with the error and returns its result.*/
    //unwrap_or_else(fn: (error: E) => T): T
    /*** Returns true if the Result is an error, false otherwise.*/
    is_err(this: Result<T, E>): this is Left<T, E>
    /*** Returns true if the Result is successful, false otherwise.*/
    is_ok(this: Result<T, E>): this is Right<T, E>
}

/** Represents a successful computation.*/
export interface Right<T, E> {
    ok: true
    value: T
    /*** Returns the value of the Result.*/
    unwrap(): T
    unwrap_err(): E
    /*** Returns the value of the Result.*/
    //unwrap_or(defaultValue: T): T
    /*** Returns the value of the Result.*/
    //unwrap_or_else(fn: (error: E) => T): T
    /*** Returns true if the Result is an error, false otherwise.*/
    is_err(this: Result<T, E>): this is Left<T, E>
    /*** Returns true if the Result is successful, false otherwise.*/
    is_ok(this: Result<T, E>): this is Right<T, E>
}

export function Ok<T, E>(value: T): Result<T, E> {
    return {
        ok: true,
        value,
        unwrap: () => value,
        unwrap_err: () => {
            throw value
        },
        //unwrap_or: () => value,
        //unwrap_or_else: () => value,
        is_err: (() => false) as any,
        is_ok: (() => true) as any,
    }
}

export function Err<T, E>(error: E): Result<T, E> {
    return {
        ok: false,
        error,
        unwrap: () => {
            throw error
        },
        unwrap_err: () => error,
        //unwrap_or: (defaultValue: T) => defaultValue,
        //unwrap_or_else: (fn: (error: E) => T) => fn(error),
        is_err: (() => true) as any,
        is_ok: (() => false) as any,
    }
}

export type user_ptr<T> = {
    pointee: T
}

export class Ref<T> {
    ref: T | null

    constructor(value: T) {
        this.ref = value
    }

    get value(): T {
        if (this.ref === null) throw new Error("ref is empty")
        return this.ref
    }

    set value(newValue: T) {
        this.ref = newValue
    }

    take(): T | null {
        const value = this.ref
        this.ref = null
        return value
    }
}

// TODO: strip this from builds
export function debug_invariant(condition: any, message?: string | (() => string)) {
    invariant(condition, message)
}

export class Deque<T = any> {
    deque: T[]
    constructor() {
        this.deque = new Array<T>()
    }

    pust_front(element: T) {
        this.deque.unshift(element)
    }

    push_back(element: T) {
        this.deque.push(element)
    }

    pop_front() {
        if (!this.is_empty()) {
            return this.deque.shift() ?? null
        }
        return null
    }

    pop_back() {
        if (!this.is_empty()) {
            return this.deque.pop() ?? null
        }
        return null
    }

    get front() {
        if (!this.is_empty()) {
            return this.deque[0]
        }
        return null
    }

    get back() {
        if (!this.is_empty()) {
            return this.deque[this.length - 1]
        }
        return null
    }

    is_empty() {
        return this.deque.length === 0
    }

    get length() {
        return this.deque.length
    }
}
