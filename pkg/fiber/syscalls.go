//go:build !wasm

package fiber

import (
	"context"
	"fmt"
	"time"
	"unsafe"
	"go.uber.org/zap"
)

func (kern *Kernel) SysHandleDuplicate(handle Handle, out *Handle) {}

func (kern *Kernel) SysHandleReplace(handle Handle, out *Handle) {}

func (kern *Kernel) SysHandleClose(handle Handle) {
	zap.L().Info("syscall HandleClose", zap.Any("handle", handle))
}

func (kern *Kernel) SysChannelCreate(out0 *Handle, out1 *Handle) {
	*out0 = 2
	*out1 = 3

	zap.L().Info("syscall ChannelCreate", zap.Any("out0", out0), zap.Any("out1", out1))
}

func (kern *Kernel) SysChannelWrite(handle Handle, bytes unsafe.Pointer, handles *Handle, numBytes uint32, numHandles uint32) {

	zap.L().Info(
		"syscall ChannelWrite",
		zap.Any("handle", handle),
		zap.Any("bytes", bytes),
		zap.Any("handles", handles),
		zap.Uint32("numBytes", numBytes),
		zap.Uint32("numHandles", numHandles))
}

func (kern *Kernel) SysChannelRead(handle Handle, bytes unsafe.Pointer, handles *Handle, numBytes uint32, numHandles uint32, actualBytes *uint32, actualHandles *uint32) {
	fmt.Println("syscall ChannelRead", handle, bytes, handles, numBytes, numHandles)
}

func (kern *Kernel) SysProcessCreate(name string, program ProgramInterface, out *Handle) {
	GKernelMut.Lock()
	defer GKernelMut.Unlock()

	hid := NewId()

	*out = Handle(hid)
	kern.processes[hid] = ProcessObject{program: program}

	zap.L().Info("syscall ProcessCreate", zap.Any("out", out), zap.String("name", name))
}

func (kern *Kernel) SysProcessStart(procHandle Handle, bootstrap *Handle) {
	kern.wg.Add(1)

	zap.L().Info(
		"syscall ProcessStart",
		zap.Any("procHandle", procHandle),
		zap.Any("bootstrap", bootstrap),
	)

	go func() {
		defer kern.wg.Done()

		ctx := context.Background()
		ctx, _ = context.WithCancel(ctx)
		ctx, _ = context.WithTimeout(ctx, time.Second)

		kern.processes[uint32(procHandle)].program.Start(ctx, bootstrap)

		print("done")
	}()
}
