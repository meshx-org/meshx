import { CreateNodesV2 } from "@nx/devkit";
import { runBuildtool } from "./command";

export const createNodesV2: CreateNodesV2 = [
    "**/*/BUILD.hcl",
    (indexPathList, conf, ctx) => {
        const output = runBuildtool(indexPathList);
        if (!output) return [];
        return output;
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
