import { Kernel } from "@meshx-org/fiber-kernel";
import { initSys } from "@meshx-org/fiber-sys";

export async function createKernel() {
    const kernel = new Kernel();
    initSys(kernel);
    await kernel.boot();
}
