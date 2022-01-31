package fiber

type Handle uint32

type Channel Handle
type Process Handle

const HandleInvalid = Handle(0)

func (c *Channel) Handle() *Handle {
	return (*Handle)(c)
}

func (p *Process) Handle() *Handle {
	return (*Handle)(p)
}

