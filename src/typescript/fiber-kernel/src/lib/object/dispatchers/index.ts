import { ChannelDispatcher } from './channel-dispatcher'
import { JobDispatcher } from "./job-dispatcher"
import { ProcessDispatcher } from "./process-dispatcher"
import { PortDispatcher } from "./port-dispatcher"

export * from "./dispatcher"
export * from "./channel-dispatcher"
export * from "./job-dispatcher"
export * from "./process-dispatcher"
export * from "./port-dispatcher"

export type GenericDispatcher  =
    | { tag: "Process"; value: ProcessDispatcher }
    | { tag: "Job"; value: JobDispatcher }
    | { tag: "Channel"; value: ChannelDispatcher }
    | { tag: "Port"; value: PortDispatcher }
