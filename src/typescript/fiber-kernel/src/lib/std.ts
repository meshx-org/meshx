export type Result<T, E = undefined> = { ok: true; value: T } | { ok: false; error: E | undefined }

export const Ok = <T>(data: T): Result<T, never> => {
    return { ok: true, value: data }
}

export const Err = <E>(error?: E): Result<never, E> => {
    return { ok: false, error }
}

export class Ref<T> {
    private _value: T | undefined

    constructor(value: T) {
        this._value = value
    }

    get value(): T {
        if (typeof this._value == "undefined") throw new Error("ref is empty")
        return this._value
    }

    set value(newValue: T) {
        this._value = newValue
    }

    take(): T | undefined {
        const value = this._value
        this._value = undefined
        return value
    }
}
