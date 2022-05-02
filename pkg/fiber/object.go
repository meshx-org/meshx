package fiber

type ProcessObject struct {
	htable  map[uint32]interface{}
	program Handle
}
