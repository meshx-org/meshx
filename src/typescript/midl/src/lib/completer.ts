export class Completer<T> {
    public readonly promise: Promise<T>
    public isCompleted: boolean

    public complete!: (value: PromiseLike<T> | T) => void
    public completeError!: (reason?: any) => void

    constructor() {
        this.isCompleted = false
        this.promise = new Promise<T>((resolve, reject) => {
            this.complete = (v) => {
                resolve(v)
                this.isCompleted = true
            }
            this.completeError = (v) => {
                reject(v)
                this.isCompleted = true
            }
        })
    }
}
