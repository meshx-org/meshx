//go:build !wasm

package fiber

import (
	"context"
	"time"
	"unsafe"
)

func (kern *Kernel) SysHandleDuplicate(handle Handle, out *Handle) {
	kern.klog.Printf("HandleDuplicate %#v %#v", handle, out)
}

func (kern *Kernel) SysHandleReplace(handle Handle, out *Handle) {
	kern.klog.Printf("HandleReplace %#v %#v", handle, out)
}

func (kern *Kernel) SysHandleClose(handle Handle) {
	kern.klog.Printf("HandleClose %#v", handle)
}

func (kern *Kernel) SysChannelCreate(out0 *Handle, out1 *Handle) {
	*out0 = 2
	*out1 = 3

	kern.klog.Printf("ChannelCreate %#v %#v", out0, out1)
}

func (kern *Kernel) SysChannelWrite(handle Handle, bytes unsafe.Pointer, handles *Handle, numBytes uint32, numHandles uint32) {

	kern.klog.Printf(
		"ChannelWrite %#v %#v %#v %#v %#v",
		handle,
		bytes,
		handles,
		numBytes,
		numHandles)
}

func (kern *Kernel) SysChannelRead(handle Handle, bytes unsafe.Pointer, handles *Handle, numBytes uint32, numHandles uint32, actualBytes *uint32, actualHandles *uint32) {
	kern.klog.Printf("ChannelRead(%#v %#v %#v %#v %#v)", handle, bytes, handles, numBytes, numHandles)
}

func (kern *Kernel) SysProcessCreate(name string, program Handle, out *Handle) {
	GKernelMut.Lock()
	defer GKernelMut.Unlock()

	hid := NewId()

	// TODO: create process context here

	*out = Handle(hid)
	kern.processes[hid] = ProcessObject{program: program}

	kern.klog.Printf("ProcessCreate %#v %#v", out, name)
}

func (kern *Kernel) SysProcessPopSegment(handle Handle, bytes unsafe.Pointer, bytesRead *uint32) {
	kern.klog.Printf("ProcessPopSegment %#v", bytes, bytesRead)
}

func (kern *Kernel) SysProcessStart(procHandle Handle, bootstrap *Handle) {
	kern.processesWg.Add(1)

	kern.klog.Printf("ProcessStart %#v %#v", procHandle, *bootstrap)

	// clone ProgramObject {}

	go func() {
		defer kern.processesWg.Done()

		ctx := context.Background()
		ctx, _ = context.WithCancel(ctx)
		ctx, _ = context.WithTimeout(ctx, time.Microsecond*100)

		// kern.processes[uint32(procHandle)].program.Start(ctx, bootstrap)

		kern.klog.Printf("process finished")
	}()
}

func (kern *Kernel) SysProgramCreate(out *Handle) {
	kern.klog.Printf("ProgramCreate %#v", out)
}

func (kern *Kernel) SysProgramCreateChild(handle Handle) {
	kern.klog.Printf("ProgramCreate %#v", handle)
}

func (kern *Kernel) SysProgramWriteSegment(handle Handle, bytes unsafe.Pointer, numBytes uint32) {
	kern.klog.Printf("ProgramWrite %#v", handle)
}
