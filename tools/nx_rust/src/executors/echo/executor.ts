export default async function runExecutor(options: any) {
    console.log("Executor ran for Echo", options)
    return {
        success: true,
    }
}
