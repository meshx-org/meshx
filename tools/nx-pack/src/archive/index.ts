import { logger, ExecutorContext } from "@nx/devkit"
import * as path from "path"
import { exec } from "child_process"

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface Options {
    outDir: string
    packageManifestPath: string
}

async function createArchive(options: Options, context: ExecutorContext): Promise<void> {
    const outDir = path.resolve(context.root, options.outDir)
    const root = context.projectsConfigurations?.projects[context.projectName!].root
    const packageManifestPath = path.resolve(options.packageManifestPath)

    const command = `${context.root}/dist/sys/pkg/bin/package-tool/package-tool package archive create ${packageManifestPath} -o ${outDir}/package.far`

    return new Promise((resolve, reject) => {
        exec(
            command,
            {
                env: {
                    CLICOLOR_FORCE: "1",
                    RUST_LOG: "info",
                },
                cwd: root,
            },
            (err, stdout, stderr) => {
                if (err) reject(err)

                logger.log(stdout)
                //logger.log(stderr)
                resolve()
            }
        )
    })
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
