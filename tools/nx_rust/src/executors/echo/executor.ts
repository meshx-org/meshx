import type { ExecutorContext } from "@nrwl/devkit"
import { logger } from "@nrwl/devkit"
import { exec } from "child_process"
import { promisify } from "util"

export interface EchoExecutorOptions {
    textToEcho: string
}

export default async function echoExecutor(
    options: EchoExecutorOptions,
    context: ExecutorContext
): Promise<{ success: boolean }> {
    logger.info(`Executing "echo"...`)
    logger.info(`Options: ${JSON.stringify(options, null, 2)}`)

    const { stdout, stderr } = await promisify(exec)(
        `echo ${options.textToEcho}`
    )
    
    logger.log(stdout)
    logger.error(stderr)

    const success = !stderr
    return { success }
}
