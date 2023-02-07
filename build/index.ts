import glob from "glob"
import { promises } from "fs"

interface BuildConfig {
    name: string
    targets: any[]
}

interface TargetNode {
    deps: TargetNode[]
    config: BuildConfig
}

interface TargetTree {
    root: TargetNode
}

const rawBuildFiles = new Promise<string[]>((resolve, reject) => {
    glob("**/build.json", { ignore: ["node_modules/**/*", ".git"] }, function (error, files) {
        if (error) reject(error)
        else resolve(files)
    })
})

async function readJsonFile<T>(path: string): Promise<T> {
    const fileContent = await promises.readFile(path, "utf8")
    const parsed = JSON.parse(fileContent) as T
    return parsed
}

const buildFiles: BuildConfig[] = []

async function main(target: string) {
    for (const buildFile of await rawBuildFiles) {
        const parsed = await readJsonFile<BuildConfig>(buildFile)
        buildFiles.push(parsed)
    }

    console.log(buildFiles)
}

main("//all").then(console.log)
