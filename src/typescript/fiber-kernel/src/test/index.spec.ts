import { Kernel } from "../lib"

describe("Fiber Kernel", () => {
    it("should init successfully", async () => {
        const kernel = new Kernel()

        kernel.init()
        await kernel.start()

        expect(true).toEqual(true)
    })
})
