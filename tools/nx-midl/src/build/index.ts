import { logger, ExecutorContext } from "@nrwl/devkit"
import { exec, spawn } from "child_process"
import * as path from "path"

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface Options {
    path: string
    outDir: string
    language: "ts" | "rust"
    cwd?: string
    midlcPath?: string
}

async function buildIR(options: Options, context: ExecutorContext): Promise<void> {
    const outDir = path.resolve(context.root, options.outDir)

    const projectRoot = context.projectsConfigurations?.projects[context.projectName!].root

    return new Promise((resolve, reject) => {
        exec(
            `${context.root}/dist/tools/midl/midlc/midlc compile ${options.path}`,
            {
                cwd: options.cwd ? options.cwd : projectRoot,
            },
            (err, stdout, stderr) => {
                if (err) reject(err)

                logger.log("stdout: " + stdout)
                logger.log("stderr: " + stderr)
                resolve()
            }
        )
    })
}

async function buildLibrary(options: Options, context: ExecutorContext): Promise<void> {
    const projectRoot = context.projectsConfigurations?.projects[context.projectName!].root
    const cwd = options.cwd ? options.cwd : projectRoot

    return new Promise((resolve, reject) => {
        exec(
            `${context.root}/dist/tools/midl/midlgen_${options.language}/midlgen_${
                options.language
            } --json ${path.resolve(cwd!, "ir.json")}`,
            {
                cwd,
            },
            (err, stdout, stderr) => {
                if (err) reject(err)

                logger.log(stdout)
                logger.log(stderr)
                resolve()
            }
        )
    })
}

export default async function executor(options: Options, context: ExecutorContext): Promise<{ success: boolean }> {
    try {
        logger.info(`Executing "midlc compile"...`)
        await buildIR(options, context)

        logger.info(`Executing "midlgen_ts"...`)
        await buildLibrary(options, context)

        return { success: true }
    } catch (e) {
        logger.error(`error: ${e}`)
        return { success: false }
    }
}
