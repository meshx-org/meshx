import { logger, ExecutorContext } from "@nx/devkit";
import { exec, execSync } from "child_process";
import * as path from "path";

export type Options = {
    binding: string;
    cwd?: string;
    outDir: string;
};

export function commandSync(command = "", options?: Partial<any>): any {
    const normalizedOptions: any = {
        cwd: options?.cwd,
        stdio: options?.stdio ?? "inherit",
        env: {
            ...process.env,
            ...options?.env,
        },
    };

    try {
        return {
            output: execSync(`${command}`, {
                encoding: "utf8",
                windowsHide: true,
                cwd: normalizedOptions.cwd,
                stdio: normalizedOptions.stdio,
                env: normalizedOptions.env,
                maxBuffer: 1024 * 1024 * 10,
            }),
            success: true,
        };
    } catch (e) {
        console.error(e);
        return {
            output: e as string,
            success: false,
        };
    }
}

async function buildLibrary(options: Options, context: ExecutorContext): Promise<void> {
    console.dir(context.projectGraph?.nodes, { depth: null });

    const root = context.projectsConfigurations!.projects[context.projectName!].root;

    const outDir = path.resolve(context.root, options.outDir);
    const irDir = path.resolve(path.join("dist", root));

    const binary = `cargo run -p midlgen_${options.binding} --`;
    const command = `${binary} --json ${path.resolve(irDir, "ir.midl.json")} --out ${path.resolve(outDir)}`;

    return commandSync(command, {
        env: {
            CLICOLOR_FORCE: "1",
            RUST_LOG: "info",
            RUST_BACKTRACE: "1",
        },
        shell: "/bin/zsh",
        cwd: context.root,
    });
}

export default async function executor(options: Options, context: ExecutorContext): Promise<{ success: boolean }> {
    try {
        await buildLibrary(options, context);
        return { success: true };
    } catch (e) {
        logger.error(`error: ${e}`);
        return { success: false };
    }
}
