import * as fiber from "@meshx-org/fiber-ts"
import * as midl from ".."

enum TheUnionTag {
    $unknown,
    v = 0x1,
}

const _TheUnionTag_map: Map<number, TheUnionTag> = new Map<number, TheUnionTag>([[0x1, TheUnionTag.v]])

class TheUnion extends midl.Union {
    #ordinal: number
    #data: any

    constructor(ordinal: number, data: any) {
        super()
        this.#ordinal = ordinal
        this.#data = data
    }

    static withV(value: number): TheUnion {
        return new TheUnion(1, value)
    }

    static with$UnknownData(ordinal: number, data: midl.UnknownRawData): TheUnion {
        return new TheUnion(ordinal, data)
    }

    get $tag(): TheUnionTag {
        return _TheUnionTag_map.get(this.#ordinal) ?? TheUnionTag.$unknown
    }

    get v(): number | null {
        if (this.#ordinal !== 1) {
            return null
        }
        return this.#data as number
    }

    get $unknownData(): midl.UnknownRawData | null {
        if (this.#ordinal === 1) {
            return null
        }
        return this.#data as midl.UnknownRawData
    }

    override get $ordinal(): number {
        return this.#ordinal
    }

    override get $data(): unknown {
        return this.#data
    }

    override valueOf() {
        return this.#data
    }

    static ctor(ordinal: number, data: object): TheUnion {
        return new TheUnion(ordinal, data)
    }
}

const _TheUnionType = new midl.UnionType<TheUnion>(
    {
        1: new midl.UInt32Type(),
    },
    TheUnion.ctor,
    true,
    false
)

describe("Unions", () => {
    it("encode unions", async () => {
        const union = TheUnion.withV(12345678)

        const handles: any[] = []
        const bytes = midl.Encoder.encode(handles, union, _TheUnionType)

        expect(bytes).toEqual(
            /* prettier-ignore */
            new Uint8Array([
                /* ordinal */ 1 , 0 , 0  , 0, 0, 0, 0, 0,
                /* data */    78, 97, 188, 0, 0, 0, 1, 0,
            ])
        )
    })

    it("decode unions", async () => {
        const bytes =
            /* prettier-ignore */
            new Uint8Array([
                /* ordinal */ 1 , 0 , 0  , 0, 0, 0, 0, 0,
                /* data */    78, 97, 188, 0, 0, 0, 1, 0,
            ])

        const handles: any[] = []
        const union = midl.Decoder.decode(bytes, handles, _TheUnionType)

        expect(union.valueOf()).toEqual(12345678)
        expect(union.$ordinal).toEqual(1)
    })
})
