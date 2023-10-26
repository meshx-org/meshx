// @vitest-environment jsdom

import { Kernel } from "../lib"

function delay(ms = 1000) {
    return new Promise((res, rej) => {
        setTimeout(res, ms)
    })
}

describe("Fiber Kernel", () => {
    it("should init successfully", async () => {
        const mocked = vi.fn(() => 0.1)
        Math.random = mocked

        const kernel = new Kernel("leader", true)

        kernel.init()
        await kernel.boot()

        const handle = kernel.get_root_job_handle()

        expect(handle.dispatcher()).toEqual(kernel.get_root_job_dispatcher())
    })

    it("connect two kernels together", async () => {
        const mocked = vi.fn(() => 0.1)
        Math.random = mocked

        const messageChannel = new MessageChannel()

        const k1 = new Kernel("k1", true)
        k1.init()
        k1.boot()

        const k2 = new Kernel("k2", false)
        k2.init()
        k2.boot()
        k2.listen(messageChannel.port1)

        k1.dial( messageChannel.port2)

        k1.unstable_newProcess("p1")
        k2.unstable_newProcess("p2")

        await delay()

        console.log(k1)
        console.log(k2)
    })
})
