import { CreateNodesV2, ProjectConfiguration, CreateDependencies, validateDependency, RawProjectGraphDependency, StaticDependency, DependencyType } from "@nx/devkit";
import * as path from "path";
import * as fs from "node:fs";
import { load } from "js-toml";

type CargoToml = {
    package: { edition: "2021"; name: string; version: string };
    dependencies: Record<string, any>;
};

function createLibProject(dir: string, cargoJson: CargoToml): ProjectConfiguration {
    return {
        name: cargoJson.package.name,
        sourceRoot: path.join(dir, "src"),
        root: dir,
        projectType: "library",
        tags: ["cargo", "lang:rs"],
        targets: {
            build: {
                executor: "@nxrs/cargo:build",
                options: {
                    toolchain: "nightly",
                    release: false,
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
        name: cargoJson.package.name,
        sourceRoot: path.join(dir, "src"),
        root: dir,
        projectType: "application",
        tags: ["cargo", "lang:rs"],
        targets: {
            build: {
                executor: "@nxrs/cargo:build",
                options: {
                    toolchain: "nightly",
                    release: false,
                },
                configurations: {
                    production: {
                        release: true,
                    },
                },
            },
            run: {
                executor: "@nxrs/cargo:build",
                options: {
                    toolchain: "nightly",
                    release: false,
                    run: true,
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

export const createDependencies: CreateDependencies = (opts, ctx) => {
    const cargoTomlMap = new Map();
    const nxProjects = Object.values(ctx.projects);
    console.log(nxProjects);
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
    (indexPathList, _) => {
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
                projects[cargoJson.package.name + "-lib"] = createLibProject(dir, cargoJson);
            }

            if (isBin) {
                projects[cargoJson.package.name + "-bin"] = createBinProject(dir, cargoJson);
            }

            return [
                /* This is used by Nx to track which matching file was used by the plugin
                 * It is shown in the project detail web view. */
                indexPath,
                {
                    projects,
                },
            ];
        });
    },
];

/**
 
                        ['test']: {
                            name: projectName,
                            sourceRoot: projectRoot,
                            projectType: "library",
                        },

 */
