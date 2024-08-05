"use client"

import type { Libp2p, PeerId, PrivateKey, PublicKey } from "@libp2p/interface"
import { keys } from "@libp2p/crypto"
import { peerIdFromKeys } from "@libp2p/peer-id"
import { webSockets } from "@libp2p/websockets"
import { WebRTC, WebSockets, WebSocketsSecure, WebTransport, Circuit } from "@multiformats/multiaddr-matcher"
import { Multiaddr, protocols } from "@multiformats/multiaddr"
import { createKernel } from "../lib/kernel"
import { useEffect, useRef, useState } from "react"
import { createLibp2p } from "libp2p"
import { webRTC, webRTCDirect } from "@libp2p/webrtc"
import { Identify, identify } from "@libp2p/identify"
import { enable } from "@libp2p/logger"
import { Echo, echo } from "@libp2p/echo"
import { noise } from "@chainsafe/libp2p-noise"
import { yamux } from "@chainsafe/libp2p-yamux"
import { circuitRelayTransport } from "@libp2p/circuit-relay-v2"
import { bootstrap } from "@libp2p/bootstrap"
import { multiaddr } from "@multiformats/multiaddr"
import * as filters from "@libp2p/websockets/filters"
import { myProtocol, MyProtocol } from "./protocol"

type ClientOnlyProps = {}

type PeerInfo = {
    peerId: PeerId
    privateKey: PrivateKey
    publicKey: PublicKey
}

// peer ids of known bootstrap nodes
export const bootstrapPeers = ["12D3KooWHRRExqPNTn1oGPvuSpqSzASNzeB7wC9P9nQWeP6FG1gB"]

async function generatePeerInfo(): Promise<PeerInfo> {
    console.info("Generating a secp256k1 private/public key pair...")
    const privateKey: PrivateKey = await keys.generateKeyPair("secp256k1")
    console.info("Generated a secp256k1 private key. Extracting public key...")
    const publicKey: PublicKey = privateKey.public
    console.info("Extracted public key. Generating PeerId...")
    const peerId: PeerId = await peerIdFromKeys(publicKey.bytes, privateKey.bytes)
    console.info("Generated PeerId: " + peerId.toString())
    return {
        peerId,
        privateKey,
        publicKey,
    }
}

async function createNode(peerInfo: PeerInfo) {
    enable("*")
    // the listener has a WebSocket transport to dial the relay, a Circuit Relay
    // transport to make a reservation, and a WebRTC transport to accept incoming
    // WebRTC connections
    const node = await createLibp2p({
        peerId: peerInfo.peerId,
        addresses: {
            listen: [
                // 👇 Listen for webRTC connections
                "/webrtc",
            ],
        },
        start: false,
        peerDiscovery: [
            bootstrap({
                // replace with your relay multiaddr
                list: ["/ip4/127.0.0.1/tcp/9001/ws/p2p/12D3KooWHRRExqPNTn1oGPvuSpqSzASNzeB7wC9P9nQWeP6FG1gB"],
            }),
        ],
        transports: [
            circuitRelayTransport({ discoverRelays: 1 }),
            webSockets({
                // Allow all WebSocket connections inclusing without TLS
                filter: filters.all,
            }),
            webRTC(),
        ],
        connectionManager: {
            minConnections: 1,
        },
        connectionGater: {
            // Allow private addresses for local testing
            denyDialMultiaddr: async () => false,
        },
        connectionEncryption: [noise()],
        streamMuxers: [yamux()],
        services: {
            fiber: myProtocol(),
            identify: identify(),
        },
    })

    //await node.start()
    await createKernel()

    // stop the node
    // await node.stop()
    //console.info('Node stopped')

    //const stream = await node.dialProtocol(ma, '/my-protocol/1.0.0', {
    //    signal: AbortSignal.timeout(10_000),
    //})

    return node
}

export function getAddresses(libp2p: Libp2p) {
    return libp2p.getMultiaddrs()
}

export function getPeerTypes(libp2p: Libp2p) {
    const types = {
        "Circuit Relay": 0,
        WebRTC: 0,
        WebSockets: 0,
        "WebSockets (secure)": 0,
        WebTransport: 0,
        Other: 0,
    }

    libp2p
        .getConnections()
        .map((conn) => conn.remoteAddr)
        .forEach((ma) => {
            if (WebRTC.exactMatch(ma) || ma.toString().includes("/webrtc/")) {
                types["WebRTC"]++
            } else if (WebSockets.exactMatch(ma)) {
                types["WebSockets"]++
            } else if (WebSocketsSecure.exactMatch(ma)) {
                types["WebSockets (secure)"]++
            } else if (WebTransport.exactMatch(ma)) {
                types["WebTransport"]++
            } else if (Circuit.exactMatch(ma)) {
                types["Circuit Relay"]++
            } else {
                types["Other"]++
                console.info("wat", ma.toString())
            }
        })

    return Object.entries(types).map(([name, count]) => (
        <li key={name}>
            {name}: {count}
        </li>
    ))
}

export function getPeerDetails(libp2p: Libp2p) {
    return libp2p.getPeers().map((peer) => {
        const peerConnections = libp2p.getConnections(peer)

        let nodeType = []

        // detect if this is a bootstrap node
        if (bootstrapPeers.includes(peer.toString())) {
            nodeType.push("bootstrap")
        }

        const relayMultiaddrs = libp2p.getMultiaddrs().filter((ma) => Circuit.exactMatch(ma))
        const relayPeers = relayMultiaddrs.map((ma) => {
            return ma
                .stringTuples()
                .filter(([name, _]) => name === protocols("p2p").code)
                .map(([_, value]) => value)[0]
        })

        // detect if this is a relay we have a reservation on
        if (relayPeers.includes(peer.toString())) {
            nodeType.push("relay")
        }

        const connections = peerConnections.map((conn) => {
            return (
                <li key={conn.id} className="break-all text-sm">
                    <button
                        className="bg-teal-500 hover:bg-teal-700 text-white px-2 mx-2 rounded focus:outline-none focus:shadow-outline"
                        onClick={() => navigator.clipboard.writeText(conn.remoteAddr.toString())}>
                        Copy
                    </button>
                    {conn.remoteAddr.toString()}{" "}
                </li>
            )
        })

        return (
            <li key={peer.toString()}>
                <span>
                    <code>{peer.toString()}</code>
                    {nodeType.length > 0 ? `(${nodeType.join(", ")})` : ""}
                </span>
                <ul className="pl-6">{connections}</ul>
            </li>
        )
    })
}

export default function ClientOnly(props: ClientOnlyProps) {
    const [node, setNode] = useState<Libp2p<{ identify: Identify; fiber: MyProtocol }> | null>(null)
    const [peerDetails, setPeerDetails] = useState<JSX.Element[]>([])
    const [nodePeerId, setNodePeerId] = useState<string>()
    const [nodePeerStatus, setNodePeerStatus] = useState<string>()
    const [nodePeerCount, setNodePeerCount] = useState(0)
    const [nodePeerTypes, setNodePeerTypes] = useState<JSX.Element[]>([])
    const [connectValue, setConnectValue] = useState("")
    const [nodeAddresses, setNodeAddress] = useState<Multiaddr[]>([])
    const [nodeAddressCount, setNodeAddressCount] = useState(0)
    const workerRef = useRef<Worker>()

    useEffect(() => {
        async function init() {
            //workerRef.current = new Worker("./worker.ts", {
            //  type: "module",
            //})

            //workerRef.current.postMessage('test')

            // generate peer info
            const peerInfo: PeerInfo = await generatePeerInfo()
            const node = await createNode(peerInfo)

            setNode(node)

            setNodePeerStatus("Online")
            setNodeAddress(getAddresses(node))
            setNodePeerTypes(getPeerTypes(node))

            setInterval(() => {
                setNodePeerCount(node.getConnections().length)
                setNodePeerTypes(getPeerTypes(node))
                setNodeAddressCount(node.getMultiaddrs().length)
                setNodeAddress(getAddresses(node))
                setPeerDetails(getPeerDetails(node))
            }, 1000)
        }

        init()
    }, [])

    const handleConnect = async () => {
        let maddr = multiaddr(connectValue)

        if (node) {
            try {
                await node.dial(maddr)
            } catch (e) {
                console.log(e)
            }
        }
    }

    return (
        <>
            <main className="text-black container mx-auto p-6 bg-white shadow-md rounded-lg mt-10">
                <h1 className="font-bold text-2xl text-blue-600 mb-4">WebRTC Connectivity with js-libp2p</h1>

                <ul className="list-disc list-inside mb-6">
                    <li>
                        Opened sessions in the last <span id="output-webtransport-opened-per-unit">0s</span>:
                        <span id="output-webtransport-opened-per-minute">0</span>
                    </li>
                    <li>
                        Max opened connections per minute: <span id="output-webtransport-max-opened-per-minute">0</span>
                    </li>
                    <li>
                        <button
                            id="button-logging-enable"
                            className="bg-blue-500 hover:bg-blue-700 text-white px-1 rounded">
                            Enable libp2p Logging
                        </button>
                        <button
                            id="button-logging-disable"
                            className="bg-red-500 hover:bg-red-700 text-white px-1 rounded">
                            Disable libp2p Logging
                        </button>
                    </li>
                </ul>

                <section className="my-4">
                    <h2 className="font-bold text-xl text-blue-600 mb-2">Node</h2>
                    <ul className="list-disc list-inside ml-4">
                        <li>
                            Peer ID:{" "}
                            <span className="font-bold" id="output-node-peer-id">
                                {nodePeerId}
                            </span>
                        </li>
                        <li>
                            Status: <span id="output-node-status">{nodePeerStatus}</span>
                        </li>
                        <li>
                            Connections: <span id="output-peer-count">{nodePeerCount}</span>
                            <ul className="list-disc list-inside ml-6" id="output-peer-types">
                                {nodePeerTypes}
                            </ul>
                        </li>
                        <li>
                            Addresses: <span id="output-address-count">{nodeAddressCount}</span>
                            <ul className="list-disc list-inside ml-6" id="output-addresses">
                                {nodeAddresses.map((ma) => {
                                    return (
                                        <li key={ma.toString()} className="text-sm break-all">
                                            <button
                                                className="bg-teal-500 hover:bg-teal-700 text-white mx-2"
                                                onClick={() => navigator.clipboard.writeText(ma.toString())}>
                                                Copy
                                            </button>
                                            {ma.toString()}
                                        </li>
                                    )
                                })}
                            </ul>
                        </li>
                    </ul>
                </section>

                <section className="my-6">
                    <h2 className="font-bold text-xl text-blue-600 mb-2">Peers</h2>
                    <p className="flex flex-row items-center gap-2">
                        <label className="block text-gray-700 text-sm font-bold" htmlFor="input-multiaddr">
                            Multiaddr
                        </label>
                        <input
                            value={connectValue}
                            onChange={(e) => setConnectValue(e.target.value)}
                            className="shadow appearance-none border rounded w-full py-2 px-2 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                            id="input-multiaddr"
                            type="text"
                            placeholder="/ip4/..."
                        />
                        <button
                            onClick={handleConnect}
                            className="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                            id="button-connect"
                            type="button">
                            Connect
                        </button>
                    </p>

                    <ul className="list-disc list-inside ml-4" id="output-peer-details">
                        {peerDetails}
                    </ul>
                </section>

                <h2 className="font-bold text-xl text-blue-600 mb-2">Output</h2>
                <pre className="bg-gray-100 p-4 rounded shadow-inner" id="output"></pre>

                <hr className="my-6 border-gray-300" />
            </main>
        </>
    )
}
