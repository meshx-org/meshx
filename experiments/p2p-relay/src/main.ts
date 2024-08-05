// @ts-check
import { createLibp2p } from 'libp2p'
//import { autoNAT } from '@libp2p/autonat'
import { identify } from '@libp2p/identify'
import { noise } from '@chainsafe/libp2p-noise'
import { yamux } from '@chainsafe/libp2p-yamux'
//import { gossipsub } from '@chainsafe/libp2p-gossipsub'
import { webSockets } from '@libp2p/websockets'
import { webRTC, webRTCDirect } from '@libp2p/webrtc'
import { enable } from '@libp2p/logger'
//import { pubsubPeerDiscovery } from '@libp2p/pubsub-peer-discovery'
// import { update, getPeerTypes, getAddresses, getPeerDetails } from './utils'
//import { bootstrap } from '@libp2p/bootstrap'
import { circuitRelayServer } from '@libp2p/circuit-relay-v2'
//import { PUBSUB_PEER_DISCOVERY } from './constants.js'

async function main() {
    enable('*')
    const node = await createLibp2p({
        addresses: {
            listen: ['/ip4/0.0.0.0/tcp/9001/ws'],
        },
        connectionManager: {
            minConnections: 0,
        },
        transports: [webSockets()],
        connectionEncryption: [noise()],
        streamMuxers: [yamux()],
        connectionGater: {
            // Allow private addresses for local testing
            denyDialMultiaddr: async () => false,
        },
        services: {
            identify: identify(),
            relay: circuitRelayServer(),
        },
    })

    console.log('PeerID: ', node.peerId.toString())
    console.log('Multiaddrs: ', node.getMultiaddrs())
}

main()

export {}
