import { Handle } from "./handle"

class HandleTransfer {
  constructor(public operation: number, public handle: Handle, public type: number, public rights: number) {}

  public toString(): String {
    return `HandleDisposition(operation=${this.operation}, handle=${this.handle}, type=${this.type}, rights=${this.rights})`
  }
}

export { HandleTransfer }
