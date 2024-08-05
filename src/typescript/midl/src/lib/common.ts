/// Calls close and prints the exception.
export function handleException(name: string, exception: Error, close: () => void): void {
    close()
    console.log("Exception handling method call $name: $exception")
}

/// Wraps work with common try/catch behaviour and timeline events.
export function performWithExceptionHandling(name: string, work: () => void, close: () => void): void {
    try {
        work()
    } catch (err) {
        handleException(name, err as Error, close)
        throw err
    }
}
