import { logger, readJsonFile, ExecutorContext } from "@nrwl/devkit"
import { exec } from "child_process"
import { sync } from "glob"
import * as path from "path"

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface Options {
    language: any
    files: string[]
    outDir: string
    midlJson?: string
    cwd?: string
    midlcPath?: string
}

interface LibraryDependency {
    path: string
}

interface MIDLJson {
    files: any
    name: string
    references: LibraryDependency[]
    include: string[]
    exclude: string[]
}

function resolveFiles(basePath: string, config: MIDLJson) {
    const midlFiles = sync(`${basePath}/${config.include}`, {
        ignore: config.exclude.map((excl) => `${basePath}/${excl}`),
    })

    return midlFiles
}

async function buildIR(options: Options, context: ExecutorContext): Promise<void> {
    // const deps = context.projectGraph?.dependencies[context.projectName!]

    // for (const dep of deps!) {
    //     const dependentNode = context.projectGraph!.nodes[dep.target]
    //
    //     dependentNode.data.root
    // }
    //

    //console.log("midlc", deps)

    const outDir = path.resolve(context.root, options.outDir)
    const root = context.projectsConfigurations?.projects[context.projectName!].root

    const midlJsonPath = path.resolve(root!, options.midlJson || "msx.json")
    const config = readJsonFile<MIDLJson>(midlJsonPath)

    // Reqursively resolve all dependancy paths from the midl.json files
    const resolveDependencies = (dependencies: LibraryDependency[]): Record<string, string[]> => {
        let libraries: Record<string, string[]> = {}

        for (const dependency of dependencies) {
            const files: string[] = []
            const dependencyPath = path.resolve(root!, dependency.path)
            const dependencyJson = readJsonFile<MIDLJson>(path.resolve(dependencyPath, "msx.json"))

            const midlFiles = resolveFiles(dependencyPath, dependencyJson)

            files.push(...midlFiles.map((file: any) => path.resolve(dependencyPath, file)))
            libraries = { ...libraries, ...resolveDependencies(dependencyJson.references) }
            libraries[dependencyJson.name] = files
        }

        return libraries
    }

    const files = resolveDependencies(config.references)

    files[config.name] = resolveFiles(path.resolve(root!), config)

    let filesFlags = ""
    Object.entries(files).forEach(([name, files]) => {
        filesFlags += `--files ${files.join(" ")} `
    })

    const command = `${context.root}/dist/tools/midl/midlc/midlc compile -n ${config.name} -o=${outDir}/ir.json ${filesFlags}`

    return new Promise((resolve, reject) => {
        exec(
            command,
            {
                env: {
                    CLICOLOR_FORCE: "1",
                    RUST_LOG: "info",
                    RUST_BACKTRACE: "1"
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
        await buildIR(options, context)

        //logger.info(`Executing "midlgen_ts"...`)
        //await buildLibrary(options, context)

        return { success: true }
    } catch (e) {
        logger.error(`error: ${e}`)
        return { success: false }
    }
}
