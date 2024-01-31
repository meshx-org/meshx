import { logger, ExecutorContext, Tr } from "@nrwl/devkit"
import { exec } from "child_process"
import * as path from "path"

export type Options = {
    binding: string
    cwd?: string
    outDir: string
}

async function buildLibrary(options: Options, context: ExecutorContext): Promise<void> {
    const projectRoot = context.projectsConfigurations?.projects[context.projectName!].root
    const cwd = options.cwd ? options.cwd : projectRoot

    const outDir = path.resolve(context.root, options.outDir)

    return new Promise((resolve, reject) => {
        exec(
            `${context.root}/dist/tools/midl/midlgen_${options.binding}/midlgen_${
                options.binding
            } --json ${path.resolve(outDir, "ir.json")} --out ${path.resolve(outDir, "mod.rs")}`,
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
        await buildLibrary(options, context)
        return { success: true }
    } catch (e) {
        logger.error(`error: ${e}`)
        return { success: false }
    }
}
