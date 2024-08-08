import { logger, ExecutorContext } from "@nx/devkit"
import { exec } from "child_process"
import * as path from "path"

export type Options = {
    binding: string
    cwd?: string
    outDir: string
}

async function buildLibrary(options: Options, context: ExecutorContext): Promise<void> {
    console.dir(context.projectGraph?.nodes, { depth: null })
    

    const root = context.projectsConfigurations?.projects[context.projectName!].root
    
    const outDir = path.resolve(context.root, options.outDir)

    const binary = `${context.root}/dist/tools/midl/midlgen_${options.binding}/midlgen_${options.binding}`
    const command = `${binary} --json ${path.resolve(outDir, "ir.json")} --out ${path.resolve(outDir, "mod.rs")}`

    // console.log(command, options)
    return new Promise((resolve, reject) => {
        exec(
            command,
            {
                env: {
                    CLICOLOR_FORCE: "1",
                    RUST_LOG: "info",
                    RUST_BACKTRACE: "1"
                },
                cwd: context.root,
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
