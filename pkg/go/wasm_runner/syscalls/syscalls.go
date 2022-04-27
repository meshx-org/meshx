package syscalls

import (
	wt "github.com/bytecodealliance/wasmtime-go"

	"github.com/meshx-org/meshx/pkg/wasm_runner/syscalls/process"
	"github.com/meshx-org/meshx/pkg/wasm_runner/syscalls/wasi"
)



func Register(linker wt.Linker, namespaceFilter []string) {
	process.Register(linker, namespaceFilter)
	wasi.Register(linker, namespaceFilter)
}
