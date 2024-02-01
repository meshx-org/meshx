import { logger, ExecutorContext } from "@nrwl/devkit"
import * as path from "path"
import { exec } from "child_process"
import { promises as fs } from "fs"

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface Options {
    todo?: string
    outDir: string
    packageName: string
    cwd?: string
}

function runCmd(command: string, cwd: string) {
    return new Promise<void>((resolve, reject) => {
        exec(
            command,
            {
                env: {
                    CLICOLOR_FORCE: "1",
                    RUST_LOG: "info",
                },
                cwd,
            },
            (err, stdout, stderr) => {
                if (err) reject(err)

                logger.log(stdout)

                resolve()
            }
        )
    })
}

async function packageBuild(options: Options, context: ExecutorContext): Promise<void> {
    const outDir = path.resolve(context.root, options.outDir)
    const root = context.projectsConfigurations?.projects[context.projectName!].root
    const absoluteRoot = path.resolve(root!)
    const cwd = options.cwd ?? root!

    await fs.mkdir(outDir, { recursive: true })

    const buildCmd = `${context.root}/dist/sys/pkg/bin/package-tool/package-tool package build ${absoluteRoot}/package.fini -o ${outDir} --api-level 1 --published-name test`
    await runCmd(buildCmd, cwd)

    const archiveCmd = `${context.root}/dist/sys/pkg/bin/package-tool/package-tool package archive create ${outDir}/package_manifest.json -o ${outDir}/package.far`
    await runCmd(archiveCmd, cwd)
}

export default async function executor(options: Options, context: ExecutorContext): Promise<{ success: boolean }> {
    try {
        await packageBuild(options, context)

        return { success: true }
    } catch (e) {
        logger.error(`error: ${e}`)
        return { success: false }
    }
}
