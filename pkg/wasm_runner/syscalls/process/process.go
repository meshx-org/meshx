package process

import (
	wt "github.com/bytecodealliance/wasmtime-go"

	utils "github.com/meshx-org/meshx/pkg/wasm_runner/syscalls/utils"
)

func Register(linker wt.Linker, namespaceFilter []string) error {

	ty := wt.NewFuncType([]*wt.ValType{}, []*wt.ValType{})
	err := linker.FuncNew("meshx::process", "pid", ty, pid)

}

//% meshx::process::id(process_id: u64, u128_ptr: u32)
//%
//% Returns UUID of a process as u128_ptr.
//%
//% Traps:
//% * If the process ID doesn't exist.
//% * If **u128_ptr** is outside the memory space.
// fn pid(mut caller: Caller<InstanceState>, _process_id: u64, u128_ptr: u32) -> Result<(), Trap> {
// 	let memory = get_memory(&mut caller)?;
//
// 	let pid = caller.data().pid;
// 	let pid_bytes = pid.as_bytes();
//
// 	memory
// 			.write(&mut caller, u128_ptr as usize, pid_bytes)
// 			.or_trap("lunatic::process::id")?;
//
// 	Ok(())
// }

// meshx::process::pid(processId: u64, u128_ptr: u32)
//
// Returns UUID of a process as u128_ptr.
//
// Traps:
// * If the process ID doesn't exist. 
// * If **u128_ptr** is outside the memory space.
func pid(caller *wt.Caller, params []wt.Val) ([]wt.Val, *wt.Trap) {
	memory, err := utils.GetMemory(caller)

	if err != nil {
		return nil, utils.ErrorToTrap(err)
	}

	caller.Context()

	return make([]wt.Val, 0), nil
}
