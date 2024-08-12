import { ExecutorContext } from "@nx/devkit";
import * as fs from "fs/promises";

export type Options = {
    outputFileName: string;
};

export default async function executor(options: Options, context: ExecutorContext): Promise<{ success: boolean }> {
    // const dep_tree = context.taskGraph!.dependencies;

    const [_, ...other] = Object.values(context.taskGraph!.tasks);

    const mapping = other
        .flatMap((task) => {
            const { project, target } = task.target;
            const targetConf = context.projectsConfigurations!.projects[project].targets![target];
            return targetConf.metadata ? (targetConf.metadata.distribution_entries as any[]) ?? null : null;
        })
        .filter((v) => v !== null);

    const writes = mapping.map((map) => fs.appendFile(options.outputFileName, `${map.destination}=${map.source}`));
    await Promise.all(writes);

    return { success: true };
}
