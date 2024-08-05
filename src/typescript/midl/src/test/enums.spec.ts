import * as midl from "../lib"

class MyStrictEnum extends midl.Enum {
    public static readonly foo = new MyStrictEnum(0x1)
    public static readonly bar = new MyStrictEnum(0x2)

    static readonly #values: Map<number, MyStrictEnum> = new Map<number, MyStrictEnum>([
        [0x1, MyStrictEnum.foo],
        [0x2, MyStrictEnum.bar],
    ])

    public readonly value: number
    public static readonly values: MyStrictEnum[] = [MyStrictEnum.foo, MyStrictEnum.bar]
    public static readonly valuesMap: Record<string, MyStrictEnum> = {
        foo: MyStrictEnum.foo,
        bar: MyStrictEnum.bar,
    }

    private constructor(value: number) {
        super()
        this.value = value
    }

    public static override valueOf(name: string): MyStrictEnum | undefined {
        return this.valuesMap[name]
    }

    public isUnknown(): boolean {
        return false
    }

    public static create(value: number): MyStrictEnum {
        if (!this.#values.has(value)) {
            throw new midl.MidlError("Invalid strict enum value: " + value, midl.ErrorCode.InvalidEnumValue)
        }
        return this.#values.get(value)!
    }
}

class MyFlexibleEnum extends midl.Enum {
    public static readonly foo = new MyFlexibleEnum(0x1, false)
    public static readonly bar = new MyFlexibleEnum(0x2, false)
    public static readonly $unknown = new MyFlexibleEnum(0xffffffff, true)

    readonly #isUnknown: boolean
    static readonly #type = new midl.UInt32Type()
    static readonly #values: Map<number, MyFlexibleEnum> = new Map<number, MyFlexibleEnum>([
        [0x1, MyFlexibleEnum.foo],
        [0x2, MyFlexibleEnum.bar],
    ])

    public readonly value: number
    public static readonly values: MyFlexibleEnum[] = [MyFlexibleEnum.foo, MyFlexibleEnum.bar]
    public static readonly valuesMap: Record<string, MyFlexibleEnum> = {
        foo: MyFlexibleEnum.foo,
        bar: MyFlexibleEnum.bar,
    }

    private constructor(value: number, isUnknown: boolean) {
        super()
        this.value = value
        this.#isUnknown = isUnknown
    }

    public static override valueOf(name: string): MyFlexibleEnum | undefined {
        return this.valuesMap[name]
    }

    public isUnknown(): boolean {
        return this.#isUnknown
    }

    public static create(value: number): MyFlexibleEnum {
        if (!this.#values.has(value)) {
            this.#values.set(value, new MyFlexibleEnum(value, true))
        }

        return this.#values.get(value)!
    }
}

const _MyStrictEnumType = new midl.EnumType<MyStrictEnum>(new midl.UInt32Type())
const _MyFlexibleEnumType = new midl.EnumType<MyFlexibleEnum>(new midl.UInt32Type())

describe("Enums", () => {
    it("flexible enums", async () => {
        const foo = MyFlexibleEnum.$unknown

        const handles: any[] = []
        const bytes = midl.Encoder.encode(handles, foo, _MyFlexibleEnumType)
        console.log(bytes)
    })

    it("strict enums", async () => {
        const foo = MyStrictEnum.bar

        const handles: any[] = []
        const bytes = midl.Encoder.encode(handles, foo, _MyStrictEnumType)
        console.log(bytes)
    })
})
