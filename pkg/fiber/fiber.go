package fiber

import "sync"

type KernelObject interface{}

type Kernel struct {
	objects map[uint32]KernelObject

	processes map[uint32]ProcessObject

	wg sync.WaitGroup
}

func NewKernel() (Kernel, error) {
	return Kernel{
		objects:   map[uint32]KernelObject{},
		processes: map[uint32]ProcessObject{},
		wg:        sync.WaitGroup{},
	}, nil
}

// Starts the kernel with a boot process and returns the proc handle
func (kern Kernel) BootProcess(root ProgramInterface) error {

	client, _, _ := NewChannel()
	proc, _ := NewProcess("init", root)

	return proc.Start(client.Handle())
}

func (kern *Kernel) Wait() {
	kern.wg.Wait()
}
