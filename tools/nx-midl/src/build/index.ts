import { logger, readJsonFile, ExecutorContext } from "@nrwl/devkit"
import { exec } from "child_process"
import * as path from "path"

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface Options {
    srcs: string[]
    outDir: string
    language: "ts" | "rust"
    midlJson?: string
    cwd?: string
    midlcPath?: string
}

interface LibraryDependency {
    path: string
}

interface MIDLJson {
    name: string
    dependencies: Record<string, LibraryDependency>
    files: string[]
}

async function buildIR(options: Options, context: ExecutorContext): Promise<void> {
    const outDir = path.resolve(context.root, options.outDir)
    const projectRoot = context.projectsConfigurations?.projects[context.projectName!].root
    const midlJsonPath = path.resolve(projectRoot!, options.midlJson || "midl.json")
    const midlJson = readJsonFile<MIDLJson>(midlJsonPath)

    // Reqursively resolve all dependancy paths from the midl.json files
    const resolveDependencies = (dependencies: Record<string, LibraryDependency>): Record<string, string[]> => {
        let libraries: Record<string, string[]> = {}
        for (const [name, dependency] of Object.entries(dependencies)) {
            const files: string[] = []
            const dependencyPath = path.resolve(projectRoot!, dependency.path)
            const dependencyJson = readJsonFile<MIDLJson>(path.resolve(dependencyPath, "midl.json"))

            files.push(...dependencyJson.files.map((file) => path.resolve(dependencyPath, file)))
            libraries = { ...libraries, ...resolveDependencies(dependencyJson.dependencies) }
            libraries[name] = files
        }
        return libraries
    }

    const files = resolveDependencies(midlJson.dependencies)
    files[midlJson.name] = [...midlJson.files.map((file) => path.resolve(projectRoot!, file))]

    let filesFlags = ""
    Object.entries(files).forEach(([name, files]) => {
        filesFlags += `--files ${files.join(" ")} `
    })

    const command = `${context.root}/dist/tools/midl/midlc/midlc compile -n ${midlJson.name} -o=${outDir}/ir.json ${filesFlags}`

    console.log(command)

    return new Promise((resolve, reject) => {
        exec(
            command,
            {
                env: {
                    CLICOLOR_FORCE: "1",
                    RUST_LOG: "debug",
                },
                cwd: options.cwd ? options.cwd : projectRoot,
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

async function buildLibrary(options: Options, context: ExecutorContext): Promise<void> {
    const projectRoot = context.projectsConfigurations?.projects[context.projectName!].root
    const cwd = options.cwd ? options.cwd : projectRoot

    const outDir = path.resolve(context.root, options.outDir)

    return new Promise((resolve, reject) => {
        exec(
            `${context.root}/dist/tools/midl/midlgen_${options.language}/midlgen_${
                options.language
            } --json ${path.resolve(outDir, "ir.json")} --out ${path.resolve(outDir, "file.rs")}`,
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
