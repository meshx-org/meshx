type RawHandle = u32
type Ref<T> = [T]

interface GlobalSyscalls {
  handle_duplicate(handle: RawHandle, out: Ref<RawHandle>): Promise<void>
  handle_close(handle: RawHandle): Promise<void>

  process_create: () => void
  process_start: () => void
  channel_create: (out1: Ref<RawHandle>, out0: Ref<RawHandle>) => void
  channel_read: () => void
  channel_write: () => void
  channel_call: () => void
}

type SyscallArgs = any[]

interface SyscallResult {
  status: any
  refs: Ref<any>[]
  data: any[]
}

declare var __fiber__: GlobalSyscalls

// if we runing inside a browser we need to use the postMessage api to send syscalls
if (typeof window !== "undefined") {
  const syscall = <T>(namespace: string, name: string, args: SyscallArgs) => {
    return new Promise<T>((res, rej) => {
      const channel = new MessageChannel()

      channel.port1.onmessage = ({ data }) => {
        channel.port1.close()
        res(data)
      }

      channel.port1.onmessageerror = (err) => {
        channel.port1.close()
        rej(err)
      }

      globalThis.postMessage({ type: `${namespace}::${name}`, args }, "/", [channel.port2])
    })
  }

  globalThis.__fiber__ = {
    handle_duplicate: async (handle: RawHandle, out: Ref<RawHandle>) => {
      let result = await syscall<SyscallResult>("fiber", "handle_duplicate", [handle])
      out = result.refs[0]
      return result.status
    },
    handle_close: async (handle: RawHandle) => {
      let result = await syscall<SyscallResult>("fiber", "handle_close", [handle])
      return result.status
    },
    process_create: () => globalThis.postMessage({ type: "fiber::process::create" }),
    process_start: () => globalThis.postMessage({ type: "fiber::process::start" }),
    channel_create: async (out0: Ref<RawHandle>, out1: Ref<RawHandle>) => {},

    channel_read: () => globalThis.postMessage({ type: "fiber::channel::read" }),
    channel_write: () => globalThis.postMessage({ type: "fiber::channel::write" }),
    channel_call: () => globalThis.postMessage({ type: "fiber::channel::call" }),
  }
} else {
  // if we runing inside a qjs we need to use the native syscalls api to send syscalls
}
