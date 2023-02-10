import { srcTypescriptKernel } from "../lib"

describe("srcTypescriptKernel", () => {
    it("should work", () => {
        expect(srcTypescriptKernel()).toEqual("src-typescript-kernel")
    })
})
