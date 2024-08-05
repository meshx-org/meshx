type Listener<T> = (data: T) => void
type ErrorListener = (error: any) => void
type DoneListener = () => void

export class BroadcastStreamController<T> {
    private listeners: Listener<T>[] = []
    private errorListeners: ErrorListener[] = []
    private doneListeners: DoneListener[] = []
    private isClosed = false
    private isFiring = false

    private readableStream: ReadableStream<T>

    constructor() {
        this.readableStream = new ReadableStream<T>({
            start: (controller) => {
                this.addListener((data) => controller.enqueue(data))
                this.addErrorListener((error) => controller.error(error))
                this.addDoneListener(() => controller.close())
            },
        })
    }

    // Method to add a listener for data
    public addListener(listener: Listener<T>): void {
        if (this.isClosed) {
            throw new Error("Cannot add listener to a closed BroadcastStreamController")
        }
        this.listeners.push(listener)
    }

    // Method to add a listener for errors
    public addErrorListener(listener: ErrorListener): void {
        if (this.isClosed) {
            throw new Error("Cannot add listener to a closed BroadcastStreamController")
        }
        this.errorListeners.push(listener)
    }

    // Method to add a listener for done
    public addDoneListener(listener: DoneListener): void {
        if (this.isClosed) {
            throw new Error("Cannot add listener to a closed BroadcastStreamController")
        }
        this.doneListeners.push(listener)
    }

    // Method to broadcast data to all listeners
    public add(data: T): void {
        if (this.isClosed) {
            throw new Error("Cannot broadcast data to a closed BroadcastStreamController")
        }
        this.isFiring = true
        for (const listener of this.listeners) {
            listener(data)
        }
        this.isFiring = false
    }

    // Method to broadcast an error to all listeners
    public addError<E extends Error>(error: E): void {
        if (this.isClosed) {
            throw new Error("Cannot broadcast error to a closed BroadcastStreamController")
        }
        this.isFiring = true
        for (const listener of this.errorListeners) {
            listener(error)
        }
        this.isFiring = false
    }

    // Method to close the stream
    public close(): void {
        if (this.isClosed) {
            throw new Error("Cannot broadcast done to a closed BroadcastStreamController")
        }
        this.isClosed = true
        this.isFiring = true
        for (const listener of this.doneListeners) {
            listener()
        }
        this.isFiring = false
        // Clear all listeners after broadcasting done
        this.listeners = []
        this.errorListeners = []
        this.doneListeners = []
    }

    // Getter to return the readable stream
    public get stream(): ReadableStream<T> {
        return this.readableStream
    }
}
