import { u32 } from "@meshx-org/fiber-types";
import { processes } from "../object/kernel-processes";
import { Channel } from "@meshx-org/fiber-ts";

async function distribution(channel: Channel): Promise<u32> {
    return 0;
}

processes.set("distribution", distribution);
