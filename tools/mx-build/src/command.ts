import { CreateNodesResultV2 } from "@nx/devkit";
import { execSync, StdioOptions } from "child_process";

type CommandRun = {
    success: boolean;
    output: string;
};

type CommandCargoOptions = {
    stdio: StdioOptions;
    cwd?: string;
    env: NodeJS.ProcessEnv | undefined;
};

export function commandSync(command: string, args = "", options?: Partial<CommandCargoOptions>): CommandRun {
    const normalizedOptions: CommandCargoOptions = {
        cwd: options?.cwd,
        stdio: options?.stdio ?? "inherit",
        env: {
            ...process.env,
            ...options?.env,
        },
    };

    try {
        return {
            output: execSync(`${command} ${args}`, {
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
        return {
            output: e as string,
            success: false,
        };
    }
}

export function runBuildtool(inputs: readonly string[]): CreateNodesResultV2 | null {
    const output = commandSync("cargo", "run -p buildtool -- "+ inputs.map((input) => `--input ${input}`).join(" "), {
        stdio: "pipe",
    });

    if (!output.success) {
        return null;
    }

    return JSON.parse(output.output) as CreateNodesResultV2;
}
