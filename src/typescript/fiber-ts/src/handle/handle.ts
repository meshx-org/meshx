import { fx_handle_t, FX_INVALID_HANDLE } from "@meshx-org/fiber-types"
import { fx_handle_close } from "@meshx-org/fiber-sys"

export class Handle {
    private $handle: fx_handle_t = FX_INVALID_HANDLE

    protected constructor(handle: fx_handle_t) {
        this.$handle = handle
    }

    public static invalid(): Handle {
        return new Handle(FX_INVALID_HANDLE)
    }

    /**  If a raw handle is obtained from some other source, this method converts
     * it into a type-safe owned handle.
     */
    public static from_raw(raw: fx_handle_t): Handle {
        return new Handle(raw)
    }

    public get raw(): fx_handle_t {
        return this.$handle
    }

    public get isValid(): boolean {
        return this.$handle !== FX_INVALID_HANDLE
    }

    public close(): void {
        const status = fx_handle_close(this.$handle)
    }

    /*public async duplicate(): Promise<Handle> {
        const { status, handle: raw } = await fx_handle_duplicate(this.$handle)

        return new Handle(raw!)
    }*/

    // TODO: Implement
    public async replace(): Promise<Handle> {
        throw new Error("Not implemented")
    }
}
