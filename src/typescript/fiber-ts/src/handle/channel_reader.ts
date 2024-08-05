// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import { Status } from "@meshx-org/fiber-types"
import { Channel } from "./channel"
import { HandleWaiter } from "./handle_waiter"

function assert(condition: boolean, message: string) {
    if (!condition) {
        throw message || "Assertion failed"
    }
}

// TODO complete this fucntion
function getStringForStatus(status: number): string {
    return "todo"
}

// TODO: move this class to a different place
export class FiberApiError extends Error {
    constructor(message: string) {
        super(message)
        this.name = this.constructor.name
    }
}

export class ChannelReaderError extends Error {
    constructor(message: string) {
        super(message)
        this.name = this.constructor.name
    }
}

export type ChannelReaderReadableHandler = () => void
export type ChannelReaderErrorHandler = (error: ChannelReaderError) => void

export class ChannelReader {
    private _channel: Channel | null = null
    private _waiter: HandleWaiter | null = null
    public onReadable: ChannelReaderReadableHandler | null = null
    public onError: ChannelReaderErrorHandler | null = null

    get channel(): Channel {
        if (!this._channel) throw new Error("no channel")
        return this._channel
    }

    get isBound(): boolean {
        return this._channel != null
    }

    public bind(channel: Channel): void {
        if (this.isBound) {
            throw new FiberApiError("ChannelReader is already bound.")
        }
        this._channel = channel
        // this._asyncWait()
    }

    public unbind(): Channel {
        if (!this.isBound) {
            throw new FiberApiError("ChannelReader is not bound")
        }
        if (!this._channel) {
            throw new FiberApiError("ChannelReader is not bound")
        }

        this._waiter?.cancel()
        this._waiter = null
        const result = this._channel
        this._channel = null
        return result
    }

    public close(): void {
        if (!this.isBound) {
            return
        }
        this._waiter?.cancel()
        this._waiter = null
        this._channel?.close()
        this._channel = null
    }

    private _asyncWait(): void {
    //    this._waiter = this._channel!.handle!.asyncWait(
    //        Channel.READABLE | Channel.PEER_CLOSED,
    //        this._handleWaitComplete
    //    )
    }

    private _errorSoon(error: ChannelReaderError): void {
        if (this.onError == null) {
            return
        }
        queueMicrotask(() => {
            // We need to re-check onError because it might have changed during the
            // asynchronous gap.
            if (this.onError != null) {
                this.onError!(error)
            }
        })
    }

    toString(): string {
        return "ChannelReader($_channel)"
    }

    private _handleWaitComplete(status: number, pending: number): void {
        assert(this.isBound, "reader is not bound")

        if (status != Status.OK) {
            close()
            this._errorSoon(
                new ChannelReaderError(`Wait completed with status ${getStringForStatus(status)} (${status})`)
            )
            return
        }
        // TODO: Change this try/catch pattern now that we don't use
        // RawReceivePort any more.
        try {
            if ((pending & Channel.READABLE) != 0) {
                if (this.onReadable != null) {
                    this.onReadable()
                }
                if (this.isBound) {
                    this._asyncWait()
                }
            } else if ((pending & Channel.PEER_CLOSED) != 0) {
                close()
                this._errorSoon(new ChannelReaderError("Peer unexpectedly closed"))
            }
        } catch (e) {
            if (e instanceof Error) {
                // An Error exception from the core libraries is probably a programming
                // error that can't be handled. We rethrow the error so that
                // FidlEventHandlers can't swallow it by mistake.
                throw e
            } else {
                console.error(e)

                close()
                this._errorSoon(new ChannelReaderError((e as ChannelReaderError).message))
            }
        }
    }
}
