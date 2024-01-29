import {
    CreateDependencies,
    CreateNodes,
    CreateNodesContext,
    DependencyType,
    ImplicitDependency,
    ProjectConfiguration,
    StaticDependency,
    readJsonFile,
    validateDependency,
} from "@nx/devkit"
import { existsSync, readFileSync } from "fs"
import { sync } from "glob"

import { dirname, join } from "path"

type MIDLConfig = {
    name: string
    compilerOptions: any
    bindings: string[]
    include: string[]
    exclude: string[]
    references: { path: string }[]
}

type PluginOptions = unknown

export const name = "midl-discover"

export const createNodes: CreateNodes<PluginOptions> = [
    "**/msx.json",
    (projectConfigurationFile: string, opts, context: CreateNodesContext) => {
        const config = readJsonFile(projectConfigurationFile) as MIDLConfig
        const root = dirname(projectConfigurationFile)
        const projects: Record<string, ProjectConfiguration> = {}

        const inputs: string[] = []
        let allFiles: string[] = []

        for (const include of config.include) {
            const jsfiles = sync(`${root}/${include}`, { ignore: config.exclude.map((excl) => `${root}/${excl}`) })

            jsfiles.forEach((file) => {
                inputs.push(`{workspaceRoot}/${file}`)
            })

            allFiles = [...allFiles, ...jsfiles]
        }

        projects[root] = {
            root: root,
            sourceRoot: root,
            projectType: "library",
            implicitDependencies: ["tools/nx-midl"],
            targets: {
                build: {
                    dependsOn: ["^build"],
                    executor: "./dist/tools/nx-midl:midlc",
                    inputs,
                    outputs: [`{workspaceRoot}/dist/${root}/ir.json`],
                    options: {
                        outDir: `dist/${root}`,
                        midlJson: "./msx.json",
                        srcs: allFiles,
                    },
                },
            },
            tags: ["midl"],
        }

        for (const binding of config.bindings) {
            projects[`dist/${root}/${binding}`] = {
                root: `dist/${root}/${binding}`,
                projectType: "library",
                implicitDependencies: ["tools/nx-midl", root],
                targets: {
                    build: {
                        dependsOn: ["^build"],
                        inputs: [`{workspaceRoot}/dist/${root}/ir.json`],
                        outputs: [],
                        executor: "nx:run-commands",
                        options: {
                            command: "echo generate rust",
                        },
                    },
                },
                tags: ["midl"],
            }
        }

        return {
            projects,
        }
    },
]

export const createDependencies: CreateDependencies<PluginOptions> = (opts, ctx) => {
    const nxProjects = Object.values(ctx.projects)
    const results: any[] = []

    for (const project of nxProjects) {
        const maybeMsxJsonPath = join(project.root, "msx.json")

        if (existsSync(maybeMsxJsonPath)) {
            const file = readFileSync(maybeMsxJsonPath).toString("utf-8")
            const json = JSON.parse(file) as MIDLConfig

            for (const reference of json.references) {
                const newDependency: StaticDependency = {
                    source: project.name!,
                    target: join(project.root, reference.path),
                    sourceFile: maybeMsxJsonPath,
                    type: DependencyType.static,
                }

                validateDependency(newDependency, ctx)
                results.push(newDependency)
            }

            /*for (const binding of json.bindings) {
                const newDependency: ImplicitDependency = {
                    source: join("dist", project.root, binding),
                    target: project.name!,
                    type: DependencyType.implicit,
                }

                validateDependency(newDependency, ctx)
                results.push(newDependency)
            }*/
        }
    }

    return results
}
