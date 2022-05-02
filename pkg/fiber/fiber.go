package fiber

import (
	"context"
	"log"
	"os"
	"sync"
)

type Kernel struct {
	syscallContext context.Context
	processes      map[uint32]ProcessObject
	processesWg    sync.WaitGroup

	natives map[string]NativeFn

	klog *log.Logger
}

func NewKernel(natives map[string]NativeFn) (Kernel, error) {

	kernel := Kernel{
		syscallContext: context.Background(),
		processes:      map[uint32]ProcessObject{},
		processesWg:    sync.WaitGroup{},
		natives:        natives,
		klog:           log.New(os.Stdout, "[klog] ", log.Lmicroseconds|log.Lmsgprefix),
	}

	return kernel, nil
}

// Starts the kernel with a boot process and returns the proc handle
func (kern *Kernel) RootProcess(executable Executable) error {
	program, _ := NewProgram()

	//err := program.Load(executable)


	client, _, _ := NewChannel()
	proc, _ := NewProcess("init", program.Handle().Load())

	return proc.Start(client.Handle())
}

func (kern *Kernel) Wait() {
	kern.processesWg.Wait()
}
