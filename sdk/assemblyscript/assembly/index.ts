import "wasi"

import {Console} from "as-wasi"

Console.log("Hello, World!")

// The entry file of your WebAssembly module.
export function main(): i32 { 

  return 0
}