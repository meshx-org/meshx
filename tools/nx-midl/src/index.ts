import {
    CreateDependencies,
    CreateNodesV2,
    DependencyType,
    ImplicitDependency,
    ProjectConfiguration,
    StaticDependency,
    readJsonFile,
    validateDependency,
} from "@nx/devkit";
import { existsSync, readFileSync } from "fs";
import { sync as globSync } from "glob";

import { dirname, join } from "path";

type MIDLConfig = {
    name: string;
    compilerOptions: any;
    bindings: string[];
    include: string[];
    exclude: string[];
    references: { path: string }[];
};

export const name = "midl-discover";

export function generateBindingProject(root: string, lang: string): ProjectConfiguration {
    return {
        root: `dist/${root}/${lang}`,
        name: `dist/${root}/${lang}`,
        projectType: "library",
        implicitDependencies: ["tools/nx-midl", root],
        targets: {
            build: {
                dependsOn: ["^build"],
                executor: "./dist/tools/nx-midl:midlgen",
                outputs: [],
                inputs: [`{workspaceRoot}/dist/${root}/ir.midl.json`],
                options: {
                    binding: "rust",
                    outDir: `dist/${root}`,
                },
            },
        },
        tags: ["midl:binding", `lang:${lang}`],
    };
}

export const createNodesV2: CreateNodesV2 = [
    "**/msx.json",
    (indexPathList) => {
        return indexPathList.map((indexPath, idx) => {
            const config = readJsonFile(indexPath) as MIDLConfig;
            const root = dirname(indexPath);

            const inputs: string[] = [];
            let allFiles: string[] = [];

            for (const include of config.include) {
                const jsfiles = globSync(`${root}/${include}`, {
                    ignore: config.exclude.map((excl) => `${root}/${excl}`),
                });

                jsfiles.forEach((file) => {
                    inputs.push(`{workspaceRoot}/${file}`);
                });

                allFiles = [...allFiles, ...jsfiles];
            }

            const projects: Record<string, ProjectConfiguration> = {};

            projects[root] = {
                root: root,
                name: root,
                sourceRoot: root,
                projectType: "library",
                targets: {
                    build: {
                        dependsOn: ["^build"],
                        executor: "./dist/tools/nx-midl:midlc",
                        inputs,
                        outputs: [`{workspaceRoot}/dist/${root}/ir.midl.json`],
                        options: {
                            outDir: `dist/${root}`,
                            midlJson: "./msx.json",
                            srcs: allFiles,
                        },
                    },
                },
                tags: ["midl:ir", "lang:json"],
            };

            for (const binding of config.bindings) {
                projects[`dist/${root}/${binding}`] = generateBindingProject(root, binding);
            }

            return [
                indexPath,
                {
                    projects,
                },
            ];
        });
    },
];

/*export const createNodes: CreateNodes = [
    "*msx.json",
    (projectConfigurationFile, opts, context) => {
        const config = readJsonFile(projectConfigurationFile) as MIDLConfig;
        const root = dirname(projectConfigurationFile);
        const projects: Record<string, ProjectConfiguration> = {};

        const inputs: string[] = [];
        let allFiles: string[] = [];

        for (const include of config.include) {
            const jsfiles = sync(`${root}/${include}`, { ignore: config.exclude.map((excl) => `${root}/${excl}`) });

            jsfiles.forEach((file) => {
                inputs.push(`{workspaceRoot}/${file}`);
            });

            allFiles = [...allFiles, ...jsfiles];
        }

        projects[root] = {
            root: root,
            name: root,
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
            tags: ["midlc"],
        };

        for (const binding of config.bindings) {
            projects[`dist/${root}/${binding}`] = {
                root: `dist/${root}/${binding}`,
                name: `dist/${root}/${binding}`,
                projectType: "library",
                implicitDependencies: ["tools/nx-midl", root],
                targets: {
                    build: {
                        dependsOn: ["^build"],
                        executor: "./dist/tools/nx-midl:midlgen",
                        outputs: [],
                        inputs: [`{workspaceRoot}/dist/${root}/ir.json`],
                        options: {
                            binding: "rust",
                            outDir: `dist/${root}`,
                        },
                    },
                },
                tags: ["lang:midl"],
            };
        }

        return {
            projects,
        };
    },
];*/

export const createDependencies: CreateDependencies = (opts, ctx) => {
    const nxProjects = Object.values(ctx.projects);
    const results: any[] = [];

    for (const project of nxProjects) {
        const msxJsonPath = join(project.root, "msx.json");

        if (existsSync(msxJsonPath)) {
            const file = readFileSync(msxJsonPath).toString("utf-8");
            const json = JSON.parse(file) as MIDLConfig;

            for (const reference of json.references) {
                const newDependency: StaticDependency = {
                    source: project.name!,
                    target: join(project.root, reference.path),
                    sourceFile: msxJsonPath,
                    type: DependencyType.static,
                };

                validateDependency(newDependency, ctx);
                results.push(newDependency);
            }

            const midlgenDep: ImplicitDependency = {
                type: DependencyType.implicit,
                source: project.name!,
                target: "tools/nx-midl",
            };
            validateDependency(midlgenDep, ctx);
            results.push(midlgenDep);

            //validateDependency(newDependency, ctx);
            //results.push();

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

    return results;
};
