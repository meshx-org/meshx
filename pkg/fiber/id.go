package fiber

var last uint32 = 0

func NewId() uint32 {
	last++
	return last
}
