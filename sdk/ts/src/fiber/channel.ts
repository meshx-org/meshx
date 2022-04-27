import { Handle } from "./handle"
import { HandleWrapper, HandleWrapperPair } from "./handleWrapper"
import { FiberStatus, System } from "./system"

type HandlePairResult = any

class Channel extends HandleWrapper {
  constructor(handle: Handle | null) {
    super(handle)
  }

  public write(data: Uint8Array, handles?: Array<Handle>): number {
    if (this.handle == null) {
      return FiberStatus.ERR_INVALID_ARGS
    }
    return System.channelWrite(this.handle, data, handles ?? [])
  }

  public writeEtc(data: Uint8Array, handleDispositions?: Array<HandleDisposition>): number {
    if (this.handle == null) {
      return FiberStatus.ERR_INVALID_ARGS
    }
    return System.channelWriteEtc(this.handle, data, handleDispositions ?? [])
  }

  public read(): ReadResult {
    if (this.handle == null) {
      return ReadResult(ZX.ERR_INVALID_ARGS)
    }
    return System.channelRead(this.handle)
  }

  public readEtc(): ReadEtcResult {
    if (this.handle == null) {
      return ReadEtcResult(ZX.ERR_INVALID_ARGS)
    }
    return System.channelReadEtc(this.handle)
  }
}

/// Typed wrapper around a linked pair of channel objects and the
/// zx_channel_create() syscall used to create them.
class ChannelPair extends HandleWrapperPair<Channel | null> {
  static create(): ChannelPair {
    const result: HandlePairResult = System.channelCreate()

    if (result.status == FiberStatus.OK) {
      return new ChannelPair(result.first, result.second)
    } else {
      return new ChannelPair(null, null)
    }
  }

  constructor(first: Channel | null, second: Channel | null) {
    super(first, second)
  }
}
