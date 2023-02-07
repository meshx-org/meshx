import { EchoExecutorSchema } from "./schema"

export default async function runExecutor(options: EchoExecutorSchema) {
    console.log("Executor ran for Echo", options)
    return {
        success: true,
    }
}
