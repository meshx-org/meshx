import { Handle } from "./handle"
import { FiberStatus } from "./system"

/// A base class for classes that wrap Handles.
class HandleWrapper {
  #handle: Handle | null

  constructor(handle: Handle | null) {
    this.#handle = handle
  }

  public get handle(): Handle {
    return this.#handle
  }

  public get isValid(): bool {
    return this.#handle?.isValid ?? false
  }

  public close(): void {
    this.#handle!.close()
    this.#handle = null
  }

  public passHandle(): Handle | null {
    const result: Handle | null = this.#handle
    this.#handle = null
    return result
  }

  // @override
  // bool operator ==(Object other) =>
  //     (other is _HandleWrapper) && handle == other.handle;

  // @override
  // int get hashCode => handle.hashCode;

  public toString(): string {
    return `${this.constructor.name}(${this.#handle})`
  }
}

/// A base class for classes that wrap a pair of Handles.
abstract class HandleWrapperPair<T> {
  #first: T
  #second: T

  constructor(first: T, second: T) {
    this.#first = first
    this.#second = second
  }
}

export { HandleWrapper, HandleWrapperPair }
