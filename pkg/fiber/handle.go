package fiber

import (
	"sync/atomic"
	"unsafe"
)

func handlePtr(data []Handle) *Handle {
	if len(data) > 0 {
		return &data[0]
	}
	return nil
}

func bytesPtr(data []byte) unsafe.Pointer {
	if len(data) > 0 {
		return unsafe.Pointer(&data[0])
	}
	return nil
}

// Common Handle logic

func (h *Handle) Close() error {
	if handle := Handle(atomic.SwapUint32((*uint32)(h), uint32(0))); handle != 0 {
		// do syscall
		return nil
	}

	return nil
}

func (h *Handle) Load() Handle {
	return Handle(atomic.LoadUint32((*uint32)(h)))
}

/// Program Handle
func NewProgram() (Program, error) {
	var h Handle
	GKernel.SysProgramCreate(&h)
	return Program(h), nil
}

func (p *Program) CreateChild() error {
	// TODO
	return nil
}

func (p *Program) WriteSegment(segment []byte) error {
	GKernel.SysProgramWriteSegment(p.Handle().Load(), bytesPtr(segment), uint32(len(segment)))
	// TODO
	return nil
}

/// Process Handle

func NewProcess(name string, program Handle) (Process, error) {
	var h Handle
	GKernel.SysProcessCreate(name, program, &h)
	return Process(h), nil
}

func (p *Process) Start(bootstrap *Handle) error {
	GKernel.SysProcessStart(p.Handle().Load(), bootstrap)

	return nil
}

func (p *Process) Close() error {
	return p.Handle().Close()
}

/// Channel Handle

func NewChannel() (Channel, Channel, error) {
	var h0, h1 Handle
	GKernel.SysChannelCreate(&h0, &h1)
	return Channel(h0), Channel(h1), nil
}

func (c *Channel) Close() error {
	return c.Handle().Close()
}

func (c *Channel) Read(data []byte, handles []Handle, flags uint32) (numBytes, numHandles uint32, _ error) {
	numBytes = uint32(len(data))
	numHandles = uint32(len(handles))
	GKernel.SysChannelRead(c.Handle().Load(), bytesPtr(data), handlePtr(handles), numBytes, numHandles, &numBytes, &numHandles)
	return numBytes, numHandles, nil
}

func (c *Channel) Write(data []byte, handles []Handle, flags uint32) error {
	GKernel.SysChannelWrite(c.Handle().Load(), bytesPtr(data), handlePtr(handles), uint32(len(data)), uint32(len(handles)))
	return nil
}
