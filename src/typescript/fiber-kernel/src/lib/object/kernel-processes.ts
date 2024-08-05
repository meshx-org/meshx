import { u32 } from "@meshx-org/fiber-types";
import { Channel } from "@meshx-org/fiber-ts";

export const processes = new Map<string, (arg1: Channel) => Promise<u32>>();
