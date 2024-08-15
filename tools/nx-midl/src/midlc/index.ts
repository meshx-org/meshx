import { logger, ExecutorContext } from "@nx/devkit";
import { exec } from "child_process";
import * as path from "path";

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface Options {
    language: any;
    outDir: string;
    srcs: string[];
    name: string;
    cwd?: string;
    midlcPath?: string;
}

interface LibraryDependency {
    path: string;
}

interface MIDLJson {
    files: any;
    name: string;
    references: LibraryDependency[];
    include: string[];
    exclude: string[];
}

async function buildIR(options: Options, context: ExecutorContext): Promise<void> {
    // const deps = context.projectGraph?.dependencies[context.projectName!]

    // for (const dep of deps!) {
    //     const dependentNode = context.projectGraph!.nodes[dep.target]
    //
    //     dependentNode.data.root
    // }
    //

    const outDir = path.resolve(context.root, options.outDir);
    const root = context.projectsConfigurations!.projects[context.projectName!].root;

    //const midlJsonPath = path.resolve(root!, options.midlJson || "msx.json")
    //const config = readJsonFile<MIDLJson>(midlJsonPath)

    // Reqursively resolve all dependancy paths from the midl.json files
    /*const resolveDependencies = (dependencies: LibraryDependency[]): Record<string, string[]> => {
        let libraries: Record<string, string[]> = {};

        for (const dependency of dependencies) {
            const files: string[] = [];
            const dependencyPath = path.resolve(root!, dependency.path);
            const dependencyJson = readJsonFile<MIDLJson>(path.resolve(dependencyPath, "msx.json"));

            const midlFiles = resolveFiles(dependencyPath, dependencyJson);

            files.push(...midlFiles.map((file: any) => path.resolve(dependencyPath, file)));
            libraries = { ...libraries, ...resolveDependencies(dependencyJson.references) };
            libraries[dependencyJson.name] = files;
        }

        return libraries;
    };*/

    const files: Record<string, string[]> = {}; // resolveDependencies(config.references)

    files[options.name] = options.srcs.map(p => path.resolve(context.cwd, p))

    let filesFlags = "";
    Object.entries(files).forEach(([name, files]) => {
        filesFlags += `--files ${files.join(" ")} `;
    });

    const command = `cargo run -p midlc -- compile -n ${options.name} -o=${outDir}/ir.midl.json ${filesFlags}`;
    return new Promise((resolve, reject) => {
        exec(
            command,
            
            {
                env: {
                    CLICOLOR_FORCE: "1",
                    RUST_LOG: "debug",
                    RUST_BACKTRACE: "full",
                },
                shell: "/bin/zsh",
                cwd: options.cwd ? options.cwd : root,
            },
            (err, stdout, stderr) => {
                if (err) reject(err);

                logger.log(stdout);
                logger.log(stderr)
                resolve();
            }
        );
    });
}

async function buildLibrary(options: Options, context: ExecutorContext): Promise<void> {
    const projectRoot = context.projectsConfigurations?.projects[context.projectName!].root;
    const cwd = options.cwd ? options.cwd : projectRoot;

    const outDir = path.resolve(context.root, options.outDir);

    return new Promise((resolve, reject) => {
        exec(
            `${context.root}/dist/tools/midl/midlgen_${options.language}/midlgen_${
                options.language
            } --json ${path.resolve(outDir, "ir.json")} --out ${path.resolve(outDir, "file.rs")}`,
            {
                cwd,
            },
            (err, stdout, stderr) => {
                if (err) reject(err);

                logger.log(stdout);
                logger.log(stderr);
                resolve();
            }
        );
    });
}

export default async function executor(options: Options, context: ExecutorContext): Promise<{ success: boolean }> {
    try {
        await buildIR(options, context);

        //logger.info(`Executing "midlgen_ts"...`)
        //await buildLibrary(options, context)

        return { success: true };
    } catch (e) {
        logger.error(`error: ${e}`);
        return { success: false };
    }
}
