import { CreateDependencies, CreateNodesV2, DependencyType, StaticDependency, validateDependency } from "@nx/devkit";
import { runBuildtool } from "./command";
import { inspect } from "util";
import { dirname } from "path";

export const createNodesV2: CreateNodesV2 = [
    "**/*/BUILD.hcl",
    (indexPathList, conf, ctx) => {
        console.log("createNodesV2", indexPathList);
        const output = runBuildtool(indexPathList);
        if (!output) return [];
        console.log("createNodesV2", output[output.length-1][1].projects["tools/buildtool"]);
        return output;

        return indexPathList.map((indexPath) => {
            const root = dirname(indexPath);
            return [
                /* This is used by Nx to track which matching file was used by the plugin
                 * It is shown in the project detail web view. */
                indexPath,
                {
                    projects: {
                        /* This will add a project to the Nx graph for the detected library. */
                        [root]: {
                            name: root,
                            //root,
                            projectType: "application",
                            targets: {
                                abcd: {
                                    executor: "nx-midl:echo",
                                    options: {
                                        command: "ls -a",
                                    },
                                },
                            },
                        },
                    },
                },
            ];
        });
    },
];

type Metadata = {
    projectDeps?: string[];
};

/*export const createDependencies: CreateDependencies = (opts, ctx) => {
    const projects = Object.values(ctx.projects);
    const results: StaticDependency[] = [];

    for (const project of projects) {
        const deps = project.metadata ? (project.metadata as Metadata).projectDeps : undefined;

        if (project.metadata && deps) {
            for (const dep of deps) {
                const newDependency: StaticDependency = {
                    type: DependencyType.static,
                    source: project.name!,
                    target: dep,
                    //sourceFile: cargoTomlPath,
                };

                validateDependency(newDependency, ctx);
                results.push(newDependency);
            }
        }
    }

    return [];
};
*/
