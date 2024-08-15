import {
    CreateNodesV2,
    ProjectConfiguration,
    CreateDependencies,
    validateDependency,
    StaticDependency,
    DependencyType,
    ProjectGraphExternalNode,
} from "@nx/devkit";
import * as path from "path";
import * as fs from "node:fs";
import { load } from "js-toml";

type CargoToml = {
    package: { edition: "2021"; name: string; version: string };
    dependencies: Record<string, any>;
};

function createLibProject(dir: string, cargoJson: CargoToml): ProjectConfiguration {
    return {
        name: dir,
        // sourceRoot: path.join(dir, "src"),
        root: dir,
        projectType: "library",
        tags: ["cargo", "lang:rs"],
        targets: {
            /*abcd2: {
                executor: "nx-midl:echo",
                options: {
                    command: "ls -a",
                },
            },*/
            build: {
                executor: "@nxrs/cargo:build",
                options: {
                    toolchain: "nightly",
                    release: false,
                    package: cargoJson.package.name,
                    outDir: path.join("dist", dir),
                },
                configurations: {
                    production: {
                        release: true,
                    },
                },
            },
            test: {
                executor: "@nxrs/cargo:test",
                options: {},
            },
            lint: {
                executor: "@nxrs/cargo:clippy",
                options: {
                    fix: false,
                    failOnWarnings: true,
                    noDeps: true,
                },
            },
        },
    };
}

function createBinProject(dir: string, cargoJson: CargoToml): ProjectConfiguration {
    return {
        name: dir,
        //sourceRoot: path.join(dir, "src"),
        root: dir,
        projectType: "application",
        tags: ["cargo", "lang:rs"],
        targets: {
            /*abcd2: {
                executor: "nx-midl:echo",
                options: {
                    command: "ls -a",
                },
            },*/
            build: {
                executor: "@nxrs/cargo:build",
                options: {
                    toolchain: "nightly",
                    release: false,
                    package: cargoJson.package.name,
                    outDir: path.join("dist", dir),
                },
                configurations: {
                    production: {
                        release: true,
                    },
                },
            },
            run: {
                executor: "nx:run-commands",
                options: {
                    command: `cargo +nightly run -p ${cargoJson.package.name}`,
                    cwd: dir,
                },
            },
            test: {
                executor: "@nxrs/cargo:test",
                options: {},
                inputs: [],
                metadata: {
                    technologies: ["typescript"],
                    description: `Runs cargo tests`,
                },
            },
            lint: {
                executor: "@nxrs/cargo:clippy",
                options: {
                    fix: false,
                    failOnWarnings: true,
                    noDeps: true,
                },
            },
        },
    };
}

type Dependency = {
    git?: string;
};

export function isExternal(packageOrDep: number | Dependency, workspaceRoot: string) {
    const isRegistry = typeof packageOrDep == "number";
    const isGit = typeof packageOrDep != "number" && typeof packageOrDep.git !== "undefined";

    return isRegistry || isGit;
}

export const createDependencies: CreateDependencies = (opts, ctx) => {
    const cargoTomlMap = new Map();
    const nxProjects = Object.values(ctx.projects);
    const results: any[] = [];

    for (const project of nxProjects) {
        if (project.tags?.includes("cargo")) {
            const cargoTomlPath = path.join(project.root, "Cargo.toml");

            if (fs.existsSync(cargoTomlPath)) {
                const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
                const cargoJson = load(cargoToml) as CargoToml;
                cargoTomlMap.set(cargoJson.package.name, project.name);
            }
        }
    }

    for (const project of nxProjects) {
        const cargoTomlPath = path.join(project.root, "Cargo.toml");
        if (fs.existsSync(cargoTomlPath)) {
            const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
            const cargoJson = load(cargoToml) as CargoToml;
            const deps = [...Object.keys(cargoJson.dependencies)];

            for (const dep of deps) {
                if (cargoTomlMap.has(dep)) {
                    const newDependency: StaticDependency = {
                        type: DependencyType.static,
                        source: project.name!,
                        target: cargoTomlMap.get(dep),
                        sourceFile: cargoTomlPath,
                    };

                    validateDependency(newDependency, ctx);
                    results.push(newDependency);
                }
            }
        }
    }

    return results;
};

export const createNodesV2: CreateNodesV2 = [
    /* This will look for all `index.ts` files that follow your file structure convention. */
    "**/*/Cargo.toml",
    (indexPathList, _, ctx) => {
        const cargoJsons = indexPathList.map((indexPath) => {
            const cargoToml = fs.readFileSync(indexPath, "utf8");
            const cargoJson = load(cargoToml) as CargoToml;
            return cargoJson;
        });

        return indexPathList.map((indexPath, idx) => {
            const { dir } = path.parse(indexPath);
            const cargoJson = cargoJsons[idx];

            const isLib = fs.existsSync(path.join(dir, "src", "lib.rs"));
            const isBin = fs.existsSync(path.join(dir, "src", "main.rs"));

            const externalNodes: Record<string, ProjectGraphExternalNode> = {};
            const projects: Record<string, ProjectConfiguration> = {};

            if ("workspace" in cargoJson) {
                return [
                    indexPath,
                    {
                        projects,
                    },
                ];
            }

            if (isLib) {
                projects[dir] = createLibProject(dir, cargoJson);
            }

            if (isBin) {
                projects[dir] = createBinProject(dir, cargoJson);
            }

            for (const [name, dep] of Object.entries(cargoJson.dependencies)) {
                if (!isExternal(dep, ctx.workspaceRoot)) {
                    const externalDepName = `cargo:${name}`;
                    if (!externalNodes?.[externalDepName]) {
                        externalNodes[externalDepName] = {
                            type: "cargo" as any,
                            name: externalDepName as any,
                            data: {
                                packageName: name,
                                version: "0.0.0",
                                //version: cargoPackageMap.get(name)?.version ?? "0.0.0",
                            },
                        };
                    }
                }
            }

            return [
                /* This is used by Nx to track which matching file was used by the plugin
                 * It is shown in the project detail web view. */
                indexPath,
                {
                    externalNodes,
                    projects,
                },
            ];
        });
    },
];
