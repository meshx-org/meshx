package fiber

type Segment struct {
	code  []byte
	entry []byte
}

type Executable struct {
	segments []Segment
}


