// Code generated by the FlatBuffers compiler. DO NOT EDIT.

package fb_test

import "strconv"

type Shape byte

const (
	ShapeNONE   Shape = 0
	ShapeRect   Shape = 1
	ShapeCircle Shape = 2
)

var EnumNamesShape = map[Shape]string{
	ShapeNONE:   "NONE",
	ShapeRect:   "Rect",
	ShapeCircle: "Circle",
}

var EnumValuesShape = map[string]Shape{
	"NONE":   ShapeNONE,
	"Rect":   ShapeRect,
	"Circle": ShapeCircle,
}

func (v Shape) String() string {
	if s, ok := EnumNamesShape[v]; ok {
		return s
	}
	return "Shape(" + strconv.FormatInt(int64(v), 10) + ")"
}
