import { execSync, spawn, StdioOptions } from "child_process";

export interface Package {
    name: string;
    version: string;
    id: string;
    license: string;
    license_file: string;
    description: string;
    source: any;
    dependencies: Dependency[];
    //targets: Target[];
    //features: Features;
    //manifest_path: string;
    //metadata: Metadata;
    /**
     * From the docs:
     * "List of registries to which this package may be published.
     * Publishing is unrestricted if null, and forbidden if an empty array."
     *
     * Additional observation:
     * false can be used by the end user but it will be converted to an empty
     * array in the cargo metadata output.
     */
    publish: string[] | null;
    authors: string[];
    categories: string[];
    default_run: any;
    rust_version: string;
    keywords: string[];
    readme: string;
    repository: string;
    homepage: string;
    documentation: string;
    edition: string;
    links: any;
}

export interface Dependency {
    name: string;
    source: string;
    req: string;
    kind: any;
    rename: any;
    optional: boolean;
    uses_default_features: boolean;
    features: any[];
    target: string;
    path: string;
    registry: any;
}

interface CargoRun {
    success: boolean;
    output: string;
}

interface RunCargoOptions {
    stdio: StdioOptions;
    cwd?: string;
    env: NodeJS.ProcessEnv | undefined;
}

export function cargoCommandSync(args = "", options?: Partial<RunCargoOptions>): CargoRun {
    const normalizedOptions: RunCargoOptions = {
        cwd: options?.cwd,
        stdio: options?.stdio ?? "inherit",
        env: {
            ...process.env,
            ...options?.env,
        },
    };

    try {
        return {
            output: execSync(`cargo ${args}`, {
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

export interface CargoMetadata {
    packages: Package[];
    workspace_members: string[];
    resolve: Resolve;
    target_directory: string;
    version: number;
    workspace_root: string;
    metadata: Metadata2;
}

export interface Metadata2 {
    docs: Docs2;
}

export interface Docs2 {
    rs: Rs2;
}

export interface Rs2 {
    "all-features": boolean;
}

export interface Resolve {
    nodes: Node[];
    root: string;
}

export interface Node {
    id: string;
    dependencies: string[];
    deps: Dep[];
    features: string[];
}

export interface Dep {
    name: string;
    pkg: string;
    dep_kinds: DepKind[];
}

export interface DepKind {
    kind: any;
    target: string;
}

export function cargoMetadata(cwd: string): CargoMetadata | null {
    const output = cargoCommandSync("metadata --format-version=1", {
        stdio: "pipe",
        cwd,
    });

    if (!output.success) {
        return null;
    }

    return JSON.parse(output.output) as CargoMetadata;
}
