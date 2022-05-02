package fiber

import "context"

type NativeFn func(args ...interface{})

type NativeProgram interface {
	Start(ctx context.Context, bootstrap *Handle) error
}

type Handle uint32
type Program Handle
type Process Handle
type Channel Handle

const HandleInvalid = Handle(0)

func (p *Program) Handle() *Handle {
	return (*Handle)(p)
}

func (p *Process) Handle() *Handle {
	return (*Handle)(p)
}

func (c *Channel) Handle() *Handle {
	return (*Handle)(c)
}