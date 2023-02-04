import { srcTypescriptKernel } from "../lib/src-typescript-kernel"

describe("srcTypescriptKernel", () => {
    it("should work", () => {
        expect(srcTypescriptKernel()).toEqual("src-typescript-kernel")
    })
})
