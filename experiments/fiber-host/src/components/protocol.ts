import type { AbortOptions, ComponentLogger, PeerId, Startable } from '@libp2p/interface'
import type { ConnectionManager, Registrar } from '@libp2p/interface-internal'
import { Logger } from '@libp2p/logger'

export interface MyProtocol {
    /**
     * Sends a request to fetch the value associated with the given key from the given peer
     */
    fetch(peer: PeerId, key: string): Promise<Uint8Array | undefined>
}

const PROTOCOL_NAME = 'fiberd'
const PROTOCOL_VERSION = '0.1'

/**
 * A simple libp2p protocol for requesting a value corresponding to a key from a peer.
 * Developers can register one or more lookup function for retrieving the value corresponding to
 * a given key.  Each lookup function must act on a distinct part of the overall key space, defined
 * by a fixed prefix that all keys that should be routed to that lookup function will start with.
 */
export class MyProtocolImpl implements Startable, MyProtocol {
    public readonly protocol: string
    private readonly components: MyProtocolComponents
    //private readonly lookupFunctions: Map<string, LookupFunction>
    private started: boolean
    private readonly init: MyProtocolInit
    private readonly log: Logger

    constructor(components: MyProtocolComponents, init: MyProtocolInit = {}) {
        this.log = components.logger.forComponent('libp2p:myproto')
        this.started = false
        this.components = components
        this.protocol = `/${PROTOCOL_NAME}/${PROTOCOL_VERSION}`
        //this.lookupFunctions = new Map() // Maps key prefix to value lookup function
        //this.handleMessage = this.handleMessage.bind(this)
        this.init = init
    }

    async start(): Promise<void> {
        await this.components.registrar.handle(this.protocol, (data) => {}, {
            maxInboundStreams: this.init.maxInboundStreams,
            maxOutboundStreams: this.init.maxOutboundStreams,
        })

        this.started = true
    }

    async stop(): Promise<void> {
        await this.components.registrar.unhandle(this.protocol)
        this.started = false
    }

    /**
    * Sends a request to fetch the value associated with the given key from the given peer
    */
    async fetch(peer: PeerId, key: string): Promise<Uint8Array | undefined> {
        this.log('dialing %s to %p', this.protocol, peer)

        //const connection = await this.components.connectionManager.openConnection(peer, options)
        //let signal = options.signal
        //let stream: Stream | undefined
        //let onAbort = (): void => {}

        return
    }
}

export interface MyProtocolComponents {
    registrar: Registrar
    connectionManager: ConnectionManager
    logger: ComponentLogger
}

export interface MyProtocolInit {
    protocolPrefix?: string
    maxInboundStreams?: number
    maxOutboundStreams?: number
    /**
     * How long we should wait for a remote peer to send any data
     */
    timeout?: number
}

export function myProtocol(init: MyProtocolInit = {}): (components: MyProtocolComponents) => MyProtocol {
    return (components) => new MyProtocolImpl(components, init)
}
