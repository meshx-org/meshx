import { logger, ExecutorContext } from "@nrwl/devkit"
import * as path from "path"
import { promises as fs } from "fs"

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface Options {
    todo?: string
    outDir: string
    packageName: string
    cwd?: string
}

async function createArchive(options: Options, context: ExecutorContext): Promise<void> {
    const outDir = path.resolve(context.root, options.outDir)
    const root = context.projectsConfigurations?.projects[context.projectName!].root

    await fs.mkdir(`${outDir}/meta`, { recursive: true })

    // Generate meta/package file.
    const pkg = { name: options.packageName }
    await fs.writeFile(`${outDir}/meta/package`, JSON.stringify(pkg))

    // const command = `${context.root}/dist/tools/mxpm/mxpm pack --out=${outDir} --dir=${directory} `

    /* return new Promise((resolve, reject) => {
        exec(
            command,
            {
                env: {
                    CLICOLOR_FORCE: "1",
                    RUST_LOG: "info",
                },
                cwd: options.cwd ? options.cwd : root,
            },
            (err, stdout, stderr) => {
                if (err) reject(err)

                logger.log(stdout)
                //logger.log(stderr)
                resolve()
            }
        )
    })*/
}

export default async function executor(options: Options, context: ExecutorContext): Promise<{ success: boolean }> {
    try {
        await createArchive(options, context)

        return { success: true }
    } catch (e) {
        logger.error(`error: ${e}`)
        return { success: false }
    }
}
