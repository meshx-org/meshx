import { ExecutorContext } from "@nx/devkit";

import { EchoExecutorSchema } from "./schema";
import executor from "./executor";

const options: EchoExecutorSchema = {};
const context: ExecutorContext = {
    root: "",
    cwd: process.cwd(),
    isVerbose: false,
};

describe("Echo Executor", () => {
    it("can run", async () => {
        const output = await executor(options, context);
        expect(output.success).toBe(true);
    });
});
