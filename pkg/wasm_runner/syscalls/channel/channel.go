package channel


import wt "github.com/bytecodealliance/wasmtime-go"

// meshx::sycall::channel_create(out0 u32, out1 u32)
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
