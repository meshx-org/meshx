import { logger, ExecutorContext } from "@nx/devkit";

export type Options = {
    contents: string;
    path: string;
};

export default async function executor(options: Options, context: ExecutorContext): Promise<{ success: boolean }> {
    try {
        logger.log("gen");
        return { success: true };
    } catch (e) {
        logger.error(`error: ${e}`);
        return { success: false };
    }
}
