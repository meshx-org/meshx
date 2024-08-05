import { fx_handle_t, FX_HANDLE_INVALID, fx_rights_t, fx_status_t, Ref } from "@meshx-org/fiber-types"
import { fx_handle_close } from "@meshx-org/fiber-sys"

export class Handle {
    private $handle: fx_handle_t = FX_HANDLE_INVALID

    protected constructor(handle: fx_handle_t) {
        this.$handle = handle
    }

    public static invalid(): Handle {
        return new Handle(FX_HANDLE_INVALID)
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
        return this.$handle !== FX_HANDLE_INVALID
    }

    public close(): void {
        const status = fx_handle_close(this.$handle)
    }

    public duplicate(rights: fx_rights_t): { status: fx_status_t, handle: Handle } {
        const handle_out = new Ref(FX_HANDLE_INVALID)
        const status = self.fiber.sys_handle_duplicate(this.$handle, rights, handle_out)
        return { status, handle: Handle.from_raw(handle_out.value) }
    }

    // TODO: Implement
    public replace(rights: fx_rights_t): Handle {
        throw new Error("Not implemented")
    }
}
