import { ChannelReader, ChannelReaderError } from "@meshx-org/fiber-ts"
import { ErrorCode, MidlError } from "./errors"
import { IncomingMessage, IncomingMessageSink, OutgoingMessage, OutgoingMessageSink } from "./message"
import { Completer } from "./completer"
import { FX_OK } from "@meshx-org/fiber-types"

/// The different states that an [AsyncBinding] or [AsyncProxy] can be in.
enum InterfaceState {
    /// The binding or proxy has not yet been bound.
    unbound,
    /// The binding or proxy has been bound to a channel.
    bound,
    /// The binding or proxy has been closed.
    closed,
}

/// An exception that's thrown if an [AsyncBinding] or [AsyncProxy] isn't in the required
/// state for the requested operation.
export class MidlStateException extends MidlError {
    /// Create a new [MidlStateException].
    constructor(message: string) {
        super(message)
    }
}

abstract class Stateful {
    state: InterfaceState = InterfaceState.unbound
    isBound!: boolean
}

/// Listens for messages and dispatches them to an implementation of [T].
export abstract class AsyncBinding<T> extends Stateful {
    /// The name of the interface [T] as a string.
    ///
    /// This is used to generate meaningful error messages at runtime.
    readonly #interfaceName: string
    readonly #reader = new ChannelReader()

    /// Decodes the given message and dispatches the decoded message to [impl].
    ///
    /// This function is called by this object whenever a message arrives over a
    /// bound channel.
    protected abstract handleMessage(message: IncomingMessage, respond: OutgoingMessageSink): void

    /// Creates a binding object in an unbound state.
    ///
    /// Rather than creating a [AsyncBinding<T>] object directly, you typically create
    /// a `TBinding` object, which are subclasses of [AsyncBinding<T>] created by the
    /// FIDL compiler for a specific interface.

    /// The implementation of [T] bound using this object.
    ///
    /// If this object is not bound, this property is null.
    #impl!: T | null
    get impl(): T | null {
        return this.#impl
    }

    constructor(interfaceName: string) {
        super()
        this.#interfaceName = interfaceName

        this.#reader.onReadable = this.#handleReadable
        this.#reader.onError = this.#handleError
    }

    #handleReadable(): void {
        const result = this.#reader.channel.queryAndReadEtc()
        if (result.bytes?.byteLength === 0) {
            throw new MidlError(`AsyncBinding<${this.#interfaceName}> Unexpected empty message or error: ${result}`)
        }

        const message = IncomingMessage.fromReadEtcResult(result)
        if (!message.isCompatible()) {
            close()
            throw new MidlError("Incompatible wire format", ErrorCode.UnknownMagic)
        }

        this.handleMessage(message, this.sendMessage)
    }

    /// Always called when the channel underneath closes.
    #handleError(error: ChannelReaderError): void {
        /// TODO(ianloic): do something with [error].
        close()
    }

    /// Sends the given message over the bound channel.
    ///
    /// If the channel is not bound, the handles inside the message are closed and
    /// the message itself is discarded.
    sendMessage(response: OutgoingMessage): void {
        if (!this.isBound) {
            response.closeHandles()
            return
        }

        this.#reader.channel.writeEtc(response.buffer, response.handleDispositions)
    }
}

/// Representation of a service that all [T] implementations should extend from.
export abstract class Service {
    /// Getter for the [ServiceData]
    abstract get $serviceData(): ServiceData<any> | null
}

/// Exposes the ability to get a hold of the service runtime name and bindings.
export abstract class ServiceData<T> {
    /// Returns the generated runtime service name.
    abstract get $name(): string

    /// Returns the generated runtime service bindings.
    abstract get $binding(): AsyncBinding<T>
}

/// Sends messages to a remote implementation of [T]
export class AsyncProxy<T> {
    /// The control plane for this proxy.
    ///
    /// Methods that manipulate the local proxy (as opposed to sending messages
    /// to the remote implementation of [T]) are exposed on this [ctrl] object to
    /// avoid naming conflicts with the methods of [T].
    readonly ctrl: AsyncProxyController<T>

    /// Creates a proxy object with the given [ctrl].
    ///
    /// Rather than creating [Proxy<T>] object directly, you typically create
    /// `TProxy` objects, which are subclasses of [Proxy<T>] created by the FIDL
    /// compiler for a specific interface.
    constructor(ctrl: AsyncProxyController<T>) {
        this.ctrl = ctrl
    }

    // In general it's probably better to avoid adding fields and methods to this
    // class. Names added to this class have to be mangled by bindings generation
    // to avoid name conflicts.
}

/// A controller for Future based proxies.
export class AsyncProxyController<T> extends Stateful {
    readonly #reader = new ChannelReader()
    readonly #completerMap: Map<number, Completer<unknown>> = new Map()

    #nextTxid = 1

    /// The service name associated with [T], if any.
    ///
    /// Will be set if the `[Discoverable]` attribute is on the FIDL interface
    /// definition. If set it will be the fully-qualified name of the interface.
    ///
    /// This string is typically used with the `ServiceProvider` interface to
    /// request an implementation of [T].
    readonly $serviceName?: string | null

    /// The name of the interface of [T].
    ///
    /// Unlike [$serviceName] should always be set and won't be fully qualified.
    /// This should only be used for debugging and logging purposes.
    readonly $interfaceName?: string | null

    /// Called whenever this object receives a response on a bound channel.
    ///
    /// Used by subclasses of [Proxy<T>] to receive responses to messages.
    public onResponse?: IncomingMessageSink

    /// Creates proxy controller.
    ///
    /// Proxy controllers are not typically created directly. Instead, you
    /// typically obtain an [AsyncProxyController<T>] object as the [AsyncProxy<T>.ctrl]
    /// property of a `TProxy` object.
    constructor(serviceName: string, interfaceName: string) {
        super()
        this.$serviceName = serviceName
        this.$interfaceName = interfaceName
    }

    /// Close the channel bound to the proxy.
    ///
    /// The proxy must have previously been bound (e.g., using [bind]).
    close(): void {
        this._close(null)
    }

    ///  close the channel and forwards error to any open completers.
    proxyError(error: MidlError): void {
        this._close(error)
    }

    private _close(error: MidlError | null): void {
        if (this.isBound) {
            this.#reader.onReadable = null
            this.#reader.onError = null
            this.#reader.close()

            this.state = InterfaceState.closed
            this.#completerMap.forEach((completer, e) =>
                completer.completeError(
                    new MidlStateException(
                        error != null
                            ? "AsyncProxyController<${$interfaceName}> is closed with error: ${error.message}"
                            : "AsyncProxyController<${$interfaceName}> is closed."
                    )
                )
            )
        }
    }

    /// Returns the completer associated with the given response message.
    ///
    /// Used by subclasses of [AsyncProxy<T>] to retrieve registered completers when
    /// handling response messages.
    getCompleter(txid: number): Completer<unknown> | null {
        const result = this.#completerMap.get(txid)!
        const removeResult = this.#completerMap.delete(txid)

        if (!removeResult) {
            this.proxyError(new MidlError("Message had unknown request id: $txid"))
        }

        return result
    }

    /// Sends the given messages over the bound channel and registers a Completer
    /// to handle the response.
    ///
    /// Used by subclasses of [AsyncProxy<T>] to send encoded messages.
    sendMessageWithResponse(message: OutgoingMessage, completer: Completer<any>): void {
        if (!this.#reader.isBound) {
            this.proxyError(new MidlStateException(`AsyncProxyController<${this.$interfaceName}> is closed.`))
            return
        }

        const _userspaceTxidMask = 0x7fffffff

        let txid = this.#nextTxid++ & _userspaceTxidMask
        
        while (txid == 0 || this.#completerMap.has(txid)) {
            txid = this.#nextTxid++ & _userspaceTxidMask
        }
        
        message.txid = txid

        this.#completerMap.set(message.txid, completer)
        const status = this.#reader.channel.writeEtc(message.buffer, message.handleDispositions)

        if (status != FX_OK) {
            this.proxyError(
                new MidlError(
                    `AsyncProxyController<${this.$interfaceName}> failed to write to channel: ${
                        this.#reader.channel
                    } (status: $status)`
                )
            )
            return
        }
    }
}
