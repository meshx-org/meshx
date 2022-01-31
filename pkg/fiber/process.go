package fiber

import "context"

type ProgramInterface interface {
	Start(ctx context.Context, bootstrap *Handle) error
}

type ProcessObject struct {
	htable  map[uint32]interface{}
	program ProgramInterface
}
