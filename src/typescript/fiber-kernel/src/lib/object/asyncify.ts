/* eslint-disable @typescript-eslint/ban-types */
/**
 * Copyright 2019 Google Inc. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// Put `__asyncify_data` somewhere at the start.
// This address is pretty hand-wavy and we might want to make it configurable in future.
// See https://github.com/WebAssembly/binaryen/blob/6371cf63687c3f638b599e086ca668c04a26cbbb/src/passes/Asyncify.cpp#L106-L113
// for structure details.
const DATA_ADDR = 16

// Place actual data right after the descriptor (which is 2 * sizeof(i32) = 8 bytes).
const DATA_START = DATA_ADDR + 8

// End data at 1024 bytes. This is where the unused area by Clang ends and real stack / data begins.
// Because this might differ between languages and parameters passed to wasm-ld, ideally we would
// use `__stack_pointer` here, but, sadly, it's not exposed via exports yet.
const DATA_END = 1024

const WRAPPED_EXPORTS = new Map()

const State = {
    None: 0,
    Unwinding: 1,
    Rewinding: 2,
}

function isPromise(obj: any) {
    return !!obj && (typeof obj === "object" || typeof obj === "function") && typeof obj.then === "function"
}

function proxyGet(obj: Record<string, any>, transform: any) {
    return new Proxy(obj, {
        get: (obj, name) => transform(obj[name as string]),
    })
}

export class Asyncify {
    #value: unknown
    exports = null
    #asyncify_stop_unwind: Function | null
    #asyncify_start_unwind: Function | null
    #asyncify_stop_rewind: Function | null
    #asyncify_start_rewind: Function | null
    #asyncify_get_state: Function | null

    constructor() {
        this.#value = undefined

        this.#asyncify_stop_unwind = null
        this.#asyncify_start_unwind = null
        this.#asyncify_stop_rewind = null
        this.#asyncify_start_rewind = null
        this.#asyncify_get_state = null
    }

    getState() {
        return this.#asyncify_get_state!()
    }

    assertNoneState() {
        const state = this.getState()
        if (state !== State.None) {
            throw new Error(`Invalid async state ${state}, expected 0.`)
        }
    }

    wrapImportFn(fn: any) {
        return (...args: any[]) => {
            if (this.getState() === State.Rewinding) {
                this.#asyncify_stop_rewind!()
                return this.#value
            }
            this.assertNoneState()
            const value = fn(...args)
            if (!isPromise(value)) {
                return value
            }
            this.#asyncify_start_unwind!(DATA_ADDR)
            this.#value = value
        }
    }

    wrapModuleImports(module: WebAssembly.Module) {
        return proxyGet(module, (value: unknown) => {
            if (typeof value === "function") {
                return this.wrapImportFn(value)
            }
            return value
        })
    }

    wrapImports(imports: WebAssembly.Imports) {
        if (imports === undefined) return

        return proxyGet(imports, (moduleImports = Object.create(null)) => this.wrapModuleImports(moduleImports))
    }

    wrapExportFn(fn: Function) {
        let newExport = WRAPPED_EXPORTS.get(fn)

        if (newExport !== undefined) {
            return newExport
        }

        newExport = async (...args: unknown[]) => {
            this.assertNoneState()

            let result = fn(...args)

            while (this.getState() === State.Unwinding) {
                this.#asyncify_stop_unwind!()
                this.#value = await this.#value
                this.assertNoneState()
                this.#asyncify_start_rewind!(DATA_ADDR)
                result = fn(...args)
            }

            this.assertNoneState()

            return result
        }

        WRAPPED_EXPORTS.set(fn, newExport)

        return newExport
    }

    wrapExports(exports: WebAssembly.Exports) {
        const newExports = Object.create(null)

        for (const exportName in exports) {
            let value = exports[exportName]
            if (typeof value === "function" && !exportName.startsWith("asyncify_")) {
                value = this.wrapExportFn(value)
            }
            Object.defineProperty(newExports, exportName, {
                enumerable: true,
                value,
            })
        }

        WRAPPED_EXPORTS.set(exports, newExports)

        return newExports
    }

    init(instance: WebAssembly.Instance, imports: any) {
        const { exports } = instance

        const memory = (exports["memory"] || (imports.env && imports.env.memory)) as WebAssembly.Memory

        new Int32Array(memory.buffer, DATA_ADDR).set([DATA_START, DATA_END])

        const newExports = this.wrapExports(exports)
        this.#asyncify_stop_unwind = newExports["asyncify_stop_unwind"]
        this.#asyncify_start_rewind = newExports["asyncify_start_rewind"]
        this.#asyncify_stop_rewind = newExports["asyncify_stop_rewind"]
        this.#asyncify_start_unwind = newExports["asyncify_start_unwind"]
        this.#asyncify_get_state = newExports["asyncify_get_state"]
        this.exports = newExports

        Object.setPrototypeOf(instance, AsyncifyInstance.prototype)
    }
}

export class AsyncifyInstance extends WebAssembly.Instance {
    constructor(module: WebAssembly.Module, imports: WebAssembly.Imports) {
        const state = new Asyncify()
        super(module, state.wrapImports(imports))
        console.log(state)
        state.init(this, imports)
    }

    override get exports() {
        return WRAPPED_EXPORTS.get(super.exports)
    }
}

Object.defineProperty(AsyncifyInstance.prototype, "exports", { enumerable: true })

export async function instantiate(source: WebAssembly.Module, imports: WebAssembly.Imports) {
    const state = new Asyncify()
    const instance = await WebAssembly.instantiate(source, state.wrapImports(imports))

    state.init(instance, imports)
    return instance
}

export async function instantiateStreaming(source: Response | PromiseLike<Response>, imports: WebAssembly.Imports) {
    const state = new Asyncify()
    const result = await WebAssembly.instantiateStreaming(source, state.wrapImports(imports))
    state.init(result.instance, imports)
    return result
}
