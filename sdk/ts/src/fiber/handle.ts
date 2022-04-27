const INVALID_HANDLE: u32 = 0

class Handle {
  #handle: RawHandle

  static invalid(): Handle {
    const h = new Handle()
    h.#handle = INVALID_HANDLE
    return h
  }

  public async close(): Promise<void> {
    await __fiber__.handle_close(this.#handle)
  }

  public get isValid(): bool {
    return this.#handle !== INVALID_HANDLE
  }

  public async duplicate(): Promise<Handle> {
    const h = new Handle()
    const outRef: Ref<RawHandle> = [h.#handle]

    await __fiber__.handle_duplicate(this.#handle, outRef)
    return h
  }
}


export { Handle }
