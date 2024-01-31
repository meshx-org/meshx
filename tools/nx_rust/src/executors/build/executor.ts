import { logger, ExecutorContext } from "@nrwl/devkit"
import { resolve } from "path"

/**
 * Workaround for NX's lack of ESM support
 *
 * See the following issues:
 * - {@link https://github.com/nrwl/nx/issues/16776 require() of ES Module executor.js ... not supported with custom plugin using es2015 module #16776 }
 * - {@link https://github.com/nrwl/nx/issues/15682 ESM Support for Nx Plugins #15682}
 */
function requireEsm<T>(module: string): Promise<T> {
    return import(module)
}

export interface BuildOptions {
    outDir: string
    cwd?: string
}

async function doBuild(options: BuildOptions, context: ExecutorContext): Promise<number | null> {
    const outDir = resolve(context.root, options.outDir)

    const { execa } = await (Function('return import("execa")')() as Promise<typeof import("execa")>)

    const projectRoot = context.projectsConfigurations?.projects[context.projectName!].root
    
    const { stdout, stderr, exitCode } = await execa(
        `cargo`,
        ["build", "-Z", "unstable-options", "--color=always", `--out-dir=${outDir}`],
        {
            cwd: options.cwd ? options.cwd : projectRoot,
        }
    )

    logger.log(stdout, stderr)

    return exitCode
}

export default async function buildExecutor(
    options: BuildOptions,
    context: ExecutorContext
): Promise<{ success: boolean }> {
    try {
        const ret = await doBuild(options, context)

        if (ret === 1) {
            return { success: false }
        }

        return { success: true }
    } catch (e) {
        console.error("error:", (e as any).stderr)
        return { success: false }
    }
}
