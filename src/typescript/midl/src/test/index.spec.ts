// @vitest-environment jsdom

import { Encoder, Decoder } from "../lib"
import * as midl from "../lib"

export class Foo implements midl.Struct {
    static #fieldType0: midl.UInt8Type = new midl.UInt8Type()
    static #fieldType1: midl.UInt64Type = new midl.UInt64Type()
    static #fieldType2: midl.StringType = new midl.StringType()

    constructor(public byte: number, public bignum: bigint, public str: string) {}

    $encode(encoder: Encoder, offset: number, depth: number): void {
        //encoder.set_next_handle_subtype($member_handle_subtype);
        //encoder.set_next_handle_rights($member_handle_rights);
        Foo.#fieldType0.encode(encoder, this.byte, offset + 0, depth)

        //encoder.set_next_handle_subtype($member_handle_subtype);
        //encoder.set_next_handle_rights($member_handle_rights);
        Foo.#fieldType1.encode(encoder, this.bignum, offset + 8, depth)

        //encoder.set_next_handle_subtype($member_handle_subtype);
        //encoder.set_next_handle_rights($member_handle_rights);
        Foo.#fieldType2.encode(encoder, this.str, offset + 16, depth)
    }

    static $decode(decoder: Decoder, offset: number, depth: number) {
        return new Foo(
            Foo.#fieldType0.decode(decoder, offset + 0, depth),
            Foo.#fieldType1.decode(decoder, offset + 8, depth),
            Foo.#fieldType2.decode(decoder, offset + 16, depth)
        )
    }
}

const _FooType = new midl.StructType<Foo>(32, Foo.$decode)

describe("Coding", () => {
    it("encode structs", async () => {
        const foo = new Foo(5, 12345678n, "你好")

        const handles: any[] = []

        const bytes = Encoder.encode(handles, foo, _FooType)

        expect(bytes).toEqual(
            /* prettier-ignore */
            new Uint8Array([
                /* byte */      5, 0  , 0  , 0, 0, 0, 0, 0,
                /* bigint */    78 , 97 , 188, 0, 0, 0, 0, 0,
                /* text size */ 6  , 0  , 0  , 0, 0, 0, 0, 0,
                /* absent */    255, 255, 255, 255, 255, 255, 255, 255,
                /* "你好" */     228, 189, 160, 229,165, 189, 0, 0,
            ])
        )
    })

    it("decode structs", async () => {
        const bytes =
            /* prettier-ignore */
            new Uint8Array([
                /* byte */      5, 0  , 0  , 0, 0, 0, 0, 0,
                /* bigint */    78 , 97 , 188, 0, 0, 0, 0, 0,
                /* text size */ 6  , 0  , 0  , 0, 0, 0, 0, 0,
                /* absent */    255, 255, 255, 255, 255, 255, 255, 255,
                /* "你好" */     228, 189, 160, 229,165, 189, 0, 0,
            ])

        const handles: any[] = []

        const val = Decoder.decode(bytes, handles, _FooType)

        expect(val).toEqual({ byte: 5, bignum: 12345678n, str: "你好" })
    })
})
