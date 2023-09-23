// @vitest-environment jsdom

import { Kernel } from "../lib"

describe("Fiber Kernel", () => {
    it("should init successfully", async () => {
        const mocked = vi.fn(() => 0.1)
        Math.random = mocked

        const kernel = new Kernel()

        kernel.init()
        await kernel.start()

        let handle = kernel.get_root_job_handle()

        expect(handle.dispatcher()).toEqual(kernel.get_root_job_dispatcher())
    })
})
