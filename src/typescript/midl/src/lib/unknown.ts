import * as fiber from "@meshx-org/fiber-ts"

/// UnknownRawData is a container for the raw bytes and handles of an unknown
/// envelope. It has an associate UnknownRawDataType that allows encoding/
/// decoding instances of this class.
export class UnknownRawData {
    constructor(public data: Uint8Array, public handles: Array<fiber.Handle>) { }
    
    closeHandles() {
        throw new Error('not implemneted')
    }
}