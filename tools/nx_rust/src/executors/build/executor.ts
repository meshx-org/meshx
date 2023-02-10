import { logger, ExecutorContext, offsetFromRoot } from "@nrwl/devkit"
import { spawn } from "child_process"
import { resolve } from "path"

export interface BuildOptions {
    outDir: string
    cwd?: string
}

async function doBuild(options: BuildOptions, context: ExecutorContext): Promise<number | null> {
    const outDir = resolve(context.root, options.outDir)

    const projectRoot = context.projectsConfigurations?.projects[context.projectName!].root

    const child = spawn(`cargo`, ["build", "-Z", "unstable-options", "--color=always", `--out-dir=${outDir}`], {
        cwd: options.cwd ? options.cwd : projectRoot,
    })

    child.stdout.setEncoding("utf8")
    child.stdout.on("data", (data) => {
        //Here is where the output goes
        logger.log(data)
    })

    child.stderr.setEncoding("utf8")
    child.stderr.on("data", (data) => {
        //Here is where the error output goes
        logger.error(data)
    })

    const exitCode = await new Promise<number | null>((resolve, reject) => {
        child.on("close", (code, signal) => {
            console.log("close", code, signal)
            resolve(code)
        })

        child.on("error", (error) => {
            console.error(error)
            reject(-1)
        })
    })

    return exitCode
}

export default async function buildExecutor(
    options: BuildOptions,
    context: ExecutorContext
): Promise<{ success: boolean }> {
    logger.info(`Executing "cargo build"...`)

    try {
        const ret = await doBuild(options, context)

        if (ret === 1) {
            return { success: false }
        }

        return { success: true }
    } catch (e) {
        console.error("error:", e)
        return { success: false }
    }
}
