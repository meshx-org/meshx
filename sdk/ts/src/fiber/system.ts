import { Handle } from "./handle"
import { HandleTransfer } from "./transfer"

class FiberPlatform {
  static get isHub(): boolean {
    return typeof globalThis.__fiber__ !== "undefined"
  }

  static get isEdge(): boolean {
    return typeof window !== "undefined"
  }
}

enum Status {
  OK = 0,
  ERR_UNKNOWN = 1,
  ERR_INVALID_ARGS = 2,
  ERR_NOT_SUPPORTED = 3,
}

/// Users of the [Result] subclasses should check the status before
/// trying to read any data. Attempting to use a value stored in a result
/// when the status in not OK will result in an exception.
interface Result {
  status: number
  toString(): string
}

export interface HandleInfo extends Result {
  handle: Handle
  type: number
  rights: number
}

export interface ReadResult extends Result {
  bytes: ArrayBuffer
  numBytes: number
  handles: Array<Handle>
}

export interface ReadEtcResult extends Result {
  bytes: ArrayBuffer
  numBytes: number
  handleInfos: Array<HandleInfo>
}

export interface WriteResult extends Result {
  numBytes: number
}

export interface HandleResult extends Result {
  handle: Handle
}

export interface HandlePairResult extends Result {
  left: Handle
  right: Handle
}

class System {
  // Channel operations.
  public static channelCreate(): HandlePairResult {
    if (FiberPlatform.isEdge) {
      // TODO: postMessage
      return { status: Status.OK, left: Handle.invalid(), right: Handle.invalid() }
    } else if (FiberPlatform.isHub) {
      // native 'System_ChannelCreate';
    }
  }

  public static channelWrite(channel: Handle, data: Uint8Array, handles: Handle[]): WriteResult {
    if (FiberPlatform.isEdge) {
      // TODO: postMessage
      return { status: Status.OK, numBytes: 0 }
    } else if (FiberPlatform.isHub) {
      // native 'System_ChannelWrite';
    }
  }

  public static channelWriteEtc(channel: Handle, data: Uint8Array, handleTransfers: HandleTransfer[]): WriteResult {
    if (FiberPlatform.isEdge) {
      // TODO: postMessage
      return { status: Status.OK, numBytes: 0 }
    } else if (FiberPlatform.isHub) {
      // native 'System_ChannelWriteEtc';
    }
  }

  public static channelRead(channel: Handle): ReadResult {
    if (FiberPlatform.isEdge) {
      // TODO: postMessage
      return { status: Status.OK, numBytes: 0, bytes: new Uint8Array(0), handles: [] }
    } else if (FiberPlatform.isHub) {
      // native 'System_ChannelRead';
    }
  }

  public static channelReadEtc(channel: Handle): ReadEtcResult {
    if (FiberPlatform.isEdge) {
      // TODO: postMessage
      return { status: Status.OK, numBytes: 0, bytes: new Uint8Array(0), handleInfos: [] }
    } else if (FiberPlatform.isHub) {
      // native 'System_ChannelReadEtc';
    }
  }

  // Process operations.
  public static processCreate(parent: Handle, tid: string): HandleResult {
    if (FiberPlatform.isEdge) {
      // TODO: postMessage
      return { status: Status.OK, handle: Handle.invalid() }
    } else if (FiberPlatform.isHub) {
      // native 'System_ProcessCreate';
    }
  }

  public static processStart(process: Handle, bootstrap: Handle) {}
}

export { System, FiberPlatform, Status as FiberStatus }
