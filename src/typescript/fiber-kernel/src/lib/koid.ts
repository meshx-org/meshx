import { fx_koid_t } from "@meshx-org/fiber-types"

let KOID_GENERATOR = 0n

/** Generates unique 64bit ids for kernel objects. */
export function generate(): fx_koid_t {
    return KOID_GENERATOR++
}
