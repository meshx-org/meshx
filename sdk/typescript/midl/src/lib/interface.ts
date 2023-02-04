// Copyright 2023 The MeshX Authors. All rights reserved.
// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import { FidlError, FidlErrorCode } from './errors'
import { IncomingMessage, IncomingMessageSink, OutgoingMessage, OutgoingMessageSink } from './message'

type Channel = any

export const epitaphOrdinal = 0xffffffffffffffffn
export type EpitaphHandler = (statusCode: number) => void

type VoidCallback = () => void

/// A channel over which messages from interface T can be sent.
///
/// An interface handle holds a [channel] whose peer expects to receive messages
/// from the FIDL interface T. The channel held by an interface handle is not
/// currently bound, which means messages cannot yet be exchanged with the
/// channel's peer.
///
/// To send messages over the channel, bind the interface handle to a `TProxy`
/// object using use [ProxyController<T>.bind] method on the proxy's
/// [Proxy<T>.ctrl] property.
///
/// Example:
///
/// ```dart
/// InterfaceHandle<T> fooHandle = [...]
/// FooProxy foo = new FooProxy();
/// foo.ctrl.bind(fooHandle);
/// foo.bar();
/// ```
///
/// To obtain an interface handle to send over a channel, used the
/// [Binding<T>.wrap] method on an object of type `TBinding`.
///
/// Example:
///
/// ```dart
/// class FooImpl extends Foo {
///   final FooBinding _binding = new FooBinding();
///
///   InterfaceHandle<T> getInterfaceHandle() => _binding.wrap(this);
///
///   @override
///   void bar() {
///     print('Received bar message.');
///   }
/// }
/// ```
export class InterfaceHandle<T> {
    private _channel: Channel | null

    constructor(channel: Channel) {
        this._channel = channel
    }

    /// The underlying channel messages will be sent over when the interface
    /// handle is bound to a [Proxy].
    ///
    /// To take the channel from this object, use [passChannel].
    get channel(): Channel | null {
        return this._channel
    }

    /// Returns [channel] and sets [channel] to `null`.
    ///
    /// Useful for taking ownership of the underlying channel.
    passChannel(): Channel | null {
        const result = this._channel
        this._channel = null
        return result
    }

    /// Closes the underlying channel.
    close(): void {
        this._channel?.close()
        this._channel = null
    }
}

/// A channel over which messages from interface T can be received.
///
/// An interface request holds a [channel] whose peer expects to be able to send
/// messages from the FIDL interface T. A channel held by an interface request
/// is not currently bound, which means messages cannot yet be exchanged with
/// the channel's peer.
///
/// To receive messages sent over the channel, bind the interface handle using
/// [Binding<T>.bind] on a `TBinding` object, which you typically hold as a
/// private member variable in a class that implements [T].
///
/// Example:
///
/// ```dart
/// class FooImpl extends Foo {
///   final FooBinding _binding = new FooBinding();
///
///   void bind(InterfaceRequest<T> request) {
///     _binding.bind(request);
///   }
///
///   @override
///   void bar() {
///     print('Received bar message.');
///   }
/// }
/// ```
///
/// To obtain an interface request to send over a channel, used the
/// [ProxyController<T>.request] method on the [Proxy<T>.ctrl] property of an
/// object of type `TProxy`.
///
/// Example:
///
/// ```dart
/// FooProxy foo = new FooProxy();
/// InterfaceRequest<T> request = foo.ctrl.request();
/// ```
export class InterfaceRequest<T> {
    private _channel: Channel | null

    constructor(channel: Channel) {
        this._channel = channel
    }

    /// The underlying channel messages will be sent over when the interface
    /// handle is bound to a [Proxy].
    ///
    /// To take the channel from this object, use [passChannel].
    get channel(): Channel | null {
        return this._channel
    }

    /// Returns [channel] and sets [channel] to `null`.
    ///
    /// Useful for taking ownership of the underlying channel.
    passChannel(): Channel | null {
        const result = this._channel
        this._channel = null
        return result
    }

    /// Closes the underlying channel.
    close(): void {
        this._channel?.close()
        this._channel = null
    }
}

export class InterfacePair<T> {
    public request: InterfaceRequest<T> | null
    public handle: InterfaceHandle<T> | null

    constructor() {
        let pair = new FX.ChannelPair()
        this.request = new InterfaceRequest<T>(pair.first)
        this.handle = new InterfaceHandle<T>(pair.second)
    }

    passRequest(): InterfaceRequest<T> | null {
        const result = this.request
        this.request = null
        return result
    }

    passHandle(): InterfaceHandle<T> | null {
        const result = this.handle
        this.handle = null
        return result
    }
}

/// Listens for messages and dispatches them to an implementation of T.
export abstract class Binding<T> {
    private readonly _reader = new ChannelReader()

    /// The implementation of [T] bound using this object.
    ///
    /// If this object is not bound, this property is null.
    private _impl: T | null = null
    get impl(): T | null {
        return this._impl
    }

    /// Whether this object is bound to a channel.
    ///
    /// See [bind] and [unbind] for more information.
    get isBound(): boolean {
        return this._impl !== null
    }

    /// Called when the channel underneath closes.
    public onConnectionError: VoidCallback | null = null

    /// Event for binding.
    public onBind: VoidCallback | null = null

    /// Event for unbinding.
    public onUnbind: VoidCallback | null = null

    /// Event for when the binding is closed.
    public onClose: VoidCallback | null = null

    /// Creates a binding object in an unbound state.
    ///
    /// Rather than creating a [Binding<T>] object directly, you typically create
    /// a `TBinding` object, which are subclasses of [Binding<T>] created by the
    /// FIDL compiler for a specific interface.
    constructor(reader: any) {
        this._reader = reader
        this._reader.onReadable = _handleReadable
        this._reader.onError = _handleError
    }

    /// Returns an interface handle whose peer is bound to the given object.
    ///
    /// Creates a channel pair, binds one of the channels to this object, and
    /// returns the other channel. Messages sent over the returned channel will be
    /// decoded and dispatched to `impl`.
    ///
    /// The `impl` parameter must not be null.
    wrap(impl: T): InterfaceHandle<T> | null {
        // TODO assert(!isBound)
        let pair = new FX.ChannelPair()
        if (pair.status != FX.OK) {
            return null
        }

        this._impl = impl
        this._reader.bind(pair.first!)

        const callback = this.onBind
        if (callback != null) {
            callback()
        }

        return new InterfaceHandle<T>(pair.second)
    }

    /// Binds the given implementation to the given interface request.
    ///
    /// Listens for messages on channel underlying the given interface request,
    /// decodes them, and dispatches the decoded messages to `impl`.
    ///
    /// This object must not already be bound.
    ///
    /// The `impl` and `interfaceRequest` parameters must not be `null`. The
    /// `channel` property of the given `interfaceRequest` must not be `null`.
    bind(impl: T, interfaceRequest: InterfaceRequest<T>): void {
        // assert(!isBound);
        let channel = interfaceRequest.passChannel()!
        this._impl = impl
        this._reader.bind(channel)
        const callback = this.onBind
        if (callback != null) {
            callback()
        }
    }

    /// Unbinds [impl] and returns the unbound channel as an interface request.
    ///
    /// Stops listening for messages on the bound channel, wraps the channel in an
    /// interface request of the appropriate type, and returns that interface
    /// request.
    ///
    /// The object must have previously been bound (e.g., using [bind]).
    unbind(): InterfaceRequest<T> {
        // TODO: assert(isBound);
        const result = new InterfaceRequest<T>(this._reader.unbind())
        this._impl = null

        const callback = this.onUnbind
        if (callback != null) {
            callback()
        }

        return result
    }

    /// Close the bound channel.
    ///
    /// This function does nothing if the object is not bound.
    close(): void {
        if (this.isBound) {
            this._reader.close()
            this._impl = null
            const callback = this.onClose
            if (callback != null) {
                callback()
            }
        }
    }

    /// Decodes the given message and dispatches the decoded message to [impl].
    ///
    /// This function is called by this object whenever a message arrives over a
    /// bound channel.
    protected abstract handleMessage(message: IncomingMessage, respond: OutgoingMessageSink): void

    private handleReadable(): void {
        const result = this._reader.channel!.queryAndReadEtc()
        if (result.bytes.lengthInBytes == 0) {
            throw new FidlError(`Unexpected empty message or error: ${result} from channel ${this._reader.channel}`)
        }
        const message = IncomingMessage.fromReadEtcResult(result)

        if (!message.isCompatible()) {
            close()
            throw new FidlError('Incompatible wire format', FidlErrorCode.fidlUnknownMagic)
        }

        this.handleMessage(message, this.sendMessage)
    }

    /// Always called when the channel underneath closes. If [onConnectionError]
    /// is set, it is called.
    private handleError(error: ChannelReaderError) {
        const callback = this.onConnectionError
        if (callback != null) {
            callback()
        }
    }

    /// Sends the given message over the bound channel.
    ///
    /// If the channel is not bound, the handles inside the message are closed and
    /// the message itself is discarded.
    sendMessage(response: OutgoingMessage): void {
        if (!this._reader.isBound) {
            response.closeHandles()
            return
        }
        this._reader.channel!.writeEtc(response.data, response.handleDispositions)
    }
}

/// The object that [ProxyController<T>.error] completes with when there is
/// an error.
export class ProxyError {
    /// Creates a proxy error with the given message.
    ///
    /// The `message` argument must not be null.
    constructor(private message: string) {}

    toString(): string {
        return `ProxyError: ${this.message}`
    }
}

/// The control plane for an interface proxy.
///
/// A proxy controller lets you operate on the local [Proxy<T>] object itself
/// rather than send messages to the remote implementation of the proxy. For
/// example, you can [unbind] or [close] the proxy.
///
/// You typically obtain a [ProxyController<T>] object as the [Proxy<T>.ctrl]
/// property of a `TProxy` object.
///
/// Example:
///
/// ```dart
/// FooProxy foo = new FooProxy();
/// fooProvider.getFoo(foo.ctrl.request());
/// ```
export class ProxyController<T> {
    /// Creates proxy controller.
    ///
    /// Proxy controllers are not typically created directly. Instead, you
    /// typically obtain a [ProxyController<T>] object as the [Proxy<T>.ctrl]
    /// property of a `TProxy` object.
    constructor($serviceName: string, $interfaceName: string) {
        this.$interfaceName = $interfaceName
        this.$serviceName = $serviceName

        this._reader.onReadable = _handleReadable
        this._reader.onError = _handleError
    }

    /// Event for binding.
    onBind: VoidCallback | null = null

    /// Event for unbinding.
    onUnbind: VoidCallback | null = null

    /// Event for when the binding is closed.
    onClose: VoidCallback | null = null

    /// Called when the channel underneath closes.
    onConnectionError: VoidCallback | null = null

    /// Called whenever this object receives a response on a bound channel.
    ///
    /// Used by subclasses of [Proxy<T>] to receive responses to messages.
    onResponse: IncomingMessageSink | null = null

    private _nextTxid = 1
    private _pendingResponsesCount = 0

    private readonly _reader = new ChannelReader()
    private readonly _callbackMap = new Map<number, Function>()

    /// A future that completes when the proxy is bound.
    private _boundCompleter = new Completer<null>()
    get bound(): Promise<null> {
        return this._boundCompleter.promise
    }

    /// A promise that completes when an error is generated by the proxy.
    private _errorCompleter = new Completer<ProxyError>()
    get error(): Promise<ProxyError> {
        return this._errorCompleter.promise
    }

    /// Whether this object is bound to a channel.
    ///
    /// See [bind] and [unbind] for more information.
    get isBound(): boolean {
        return this._reader.isBound
    }

    /// The service name associated with [T], if any.
    ///
    /// Will be set if the `[Discoverable]` attribute is on the FIDL interface
    /// definition. If set it will be the fully-qualified name of the interface.
    ///
    /// This string is typically used with the `ServiceProvider` interface to
    /// request an implementation of [T].
    private readonly $serviceName: string | null

    /// The name of the interface of [T].
    ///
    /// Unlike [$serviceName] should always be set and won't be fully qualified.
    /// This should only be used for debugging and logging purposes.
    private readonly $interfaceName: string | null

    /// Creates an interface request whose peer is bound to this interface proxy.
    ///
    /// Creates a channel pair, binds one of the channels to this object, and
    /// returns the other channel. Calls to the proxy will be encoded as messages
    /// and sent to the returned channel.
    ///
    /// The proxy must not already have been bound.
    request(): InterfaceRequest<T> {
        // TODO: assert(!isBound);
        const pair = new FX.ChannelPair()
        // TODO: assert(pair.status == FX.OK);
        this._reader.bind(pair.first!)
        this._boundCompleter.complete()

        const callback = this.onBind
        if (callback != null) {
            callback()
        }

        return new InterfaceRequest<T>(pair.second)
    }

    /// Binds the proxy to the given interface handle.
    ///
    /// Calls to the proxy will be encoded as messages and sent over the channel
    /// underlying the given interface handle.
    ///
    /// This object must not already be bound.
    ///
    /// The `interfaceHandle` parameter must not be null. The `channel` property
    /// of the given `interfaceHandle` must not be null.
    bind(interfaceHandle: InterfaceHandle<T>): void {
        // TODO: assert(!isBound);
        this._reader.bind(interfaceHandle.passChannel()!)
        this._boundCompleter.complete()

        const callback = this.onBind
        if (callback != null) {
            callback()
        }
    }

    /// Unbinds the proxy and returns the unbound channel as an interface handle.
    ///
    /// Calls on the proxy will no longer be encoded as messages on the bound
    /// channel.
    ///
    /// The proxy must have previously been bound (e.g., using [bind]).
    unbind(): InterfaceHandle<T> | null {
        // TODO: assert(isBound);
        if (!this._reader.isBound) {
            return null
        }

        const callback = this.onUnbind
        if (callback != null) {
            callback()
        }

        // TODO(rosswang): Do we need to _reset() here?
        return new InterfaceHandle<T>(this._reader.unbind())
    }

    /// Close the channel bound to the proxy.
    ///
    /// The proxy must have previously been bound (e.g., using [bind]).
    close(): void {
        if (this.isBound) {
            if (this._pendingResponsesCount > 0) {
                this.proxyError('The proxy is closed.')
            }
            this._reset()
            this._reader.close()
            const callback = this.onClose
            if (callback != null) {
                callback()
            }
        }
    }

    private _reset(): void {
        this._callbackMap.clear()
        this._errorCompleter = new Completer<ProxyError>()
        if (!this._boundCompleter.isCompleted) {
            this._boundCompleter.completeError(`Proxy<${this.$interfaceName}> closed.`)
        }
        this._boundCompleter = new Completer<null>()
        this._nextTxid = 1
        this._pendingResponsesCount = 0
    }

    private _handleReadable(): void {
        const result = this._reader.channel!.queryAndReadEtc()

        if (result.bytes.lengthInBytes == 0) {
            this.proxyError('Read from channel ${_reader.channel} failed')
            return
        }

        try {
            this._pendingResponsesCount--
            const callback = this.onResponse
            if (callback != null) {
                callback(IncomingMessage.fromReadEtcResult(result))
            }
        } catch (e: unknown) {
            if (e instanceof Error) {
                for (let handleInfo in result.handleInfos) {
                    handleInfo.handle.close()
                }
                this.proxyError(e.toString())
                close()
            }
        }
    }

    /// Always called when the channel underneath closes. If [onConnectionError]
    /// is set, it is called.
    private _handleError(error: ChannelReaderError): void {
        this.proxyError(error.toString())
        this._reset()

        const callback = this.onConnectionError
        if (callback != null) {
            callback()
        }
    }

    /// Sends the given messages over the bound channel.
    ///
    /// Used by subclasses of [Proxy<T>] to send encoded messages.
    public sendMessage(message: OutgoingMessage): void {
        if (!this._reader.isBound) {
            this.proxyError('The proxy is closed.')
            return
        }
        const status = this._reader.channel!.writeEtc(message.data, message.handleDispositions)
        if (status != FX.OK) {
            this.proxyError(`Failed to write to channel: ${this._reader.channel} (status: ${status})`)
        }
    }

    /// Sends the given messages over the bound channel and registers a callback
    /// to handle the response.
    ///
    /// Used by subclasses of [Proxy<T>] to send encoded messages.
    sendMessageWithResponse(message: OutgoingMessage, callback: Function): void {
        if (!this._reader.isBound) {
            this.proxyError('The sender is closed.')
            return
        }
        const _kUserspaceTxidMask = 0x7fffffff
        let txid = this._nextTxid++ & _kUserspaceTxidMask
        while (txid == 0 || this._callbackMap.has(txid)) txid = this._nextTxid++ & _kUserspaceTxidMask
        message.txid = txid
        const status = this._reader.channel!.writeEtc(message.data, message.handleDispositions)
        if (status != FX.OK) {
            this.proxyError('Failed to write to channel: ${_reader.channel} (status: $status)')
            return
        }
        this._callbackMap.set(message.txid, callback)
        this._pendingResponsesCount++
    }

    /// Returns the callback associated with the given response message.
    ///
    /// Used by subclasses of [Proxy<T>] to retrieve registered callbacks when
    /// handling response messages.
    getCallback(txid: number): Function | null {
        const message = this._callbackMap.get(txid)
        const result = this._callbackMap.delete(txid)

        if (!result) {
            this.proxyError('Message had unknown request id: $txid')
            return null
        }

        return message!
    }

    /// Complete the [error] future with the given message.
    proxyError(message: string): void {
        const fullMessage = `Error in proxy with interface name [${this.$interfaceName}] and service name [${this.$serviceName}]: ${message}`
        console.log(fullMessage)

        if (!this._errorCompleter.isCompleted) {
            this.error.whenComplete(() => {
                this._errorCompleter = new Completer<ProxyError>()
            })
            this._errorCompleter.complete(new ProxyError(fullMessage))
        }
    }
}

/// Sends messages to a remote implementation of [T]
export class Proxy<T> {
    /// The control plane for this proxy.
    ///
    /// Methods that manipulate the local proxy (as opposed to sending messages
    /// to the remote implementation of [T]) are exposed on this [ctrl] object to
    /// avoid naming conflicts with the methods of [T].
    readonly ctrl: ProxyController<T>

    /// Creates a proxy object with the given [ctrl].
    ///
    /// Rather than creating [Proxy<T>] object directly, you typically create
    /// `TProxy` objects, which are subclasses of [Proxy<T>] created by the FIDL
    /// compiler for a specific interface.
    constructor(ctrl: ProxyController<T>) {
        this.ctrl = ctrl
    }

    // In general it's probably better to avoid adding fields and methods to this
    // class. Names added to this class have to be mangled by bindings generation
    // to avoid name conflicts.
}

/// Identifies which type of unknown method was received.
export enum UnknownMethodType {
    /// Unknown method was a one-way method.
    oneWay,
    /// Unknown method was a two-way method.
    twoWay,
}

/// Metadata about an unknown flexible method that was received.
export class UnknownMethodMetadata {
    /// Ordinal of the method.
    readonly ordinal: bigint

    /// Type of the unknown method.
    ///
    /// For an ajar protocol, this will always be oneWay.
    readonly unknownMethodType: UnknownMethodType

    constructor(ordinal: bigint, unknownMethodType: UnknownMethodType) {
        this.ordinal = ordinal
        this.unknownMethodType = unknownMethodType
    }
}

/// Event used when an unknown, flexible event is received.
export class UnknownEvent {
    /// Ordinal of the event.
    readonly ordinal: bigint

    constructor(ordinal: bigint) {
        this.ordinal = ordinal
    }
}
