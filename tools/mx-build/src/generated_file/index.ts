import { logger, ExecutorContext } from "@nx/devkit";
import * as fs from "fs";
import { dirname } from "path";

export type Options = {
    contents: string;
    path: string;
};

export default async function executor(options: Options, context: ExecutorContext): Promise<{ success: boolean }> {
    try {
        const parent = dirname(options.path);
        fs.mkdirSync(parent, { recursive: true });
        fs.writeFileSync(options.path, options.contents);
        logger.log(options);
        return { success: true };
    } catch (e) {
        logger.error(`error: ${e}`);
        return { success: false };
    }
}
