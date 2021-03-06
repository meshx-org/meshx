// Code generated by the FlatBuffers compiler. DO NOT EDIT.

package fb_test

import "strconv"

type Color int8

const (
	ColorRed   Color = 1
	ColorGreen Color = 2
	ColorBlue  Color = 3
)

var EnumNamesColor = map[Color]string{
	ColorRed:   "Red",
	ColorGreen: "Green",
	ColorBlue:  "Blue",
}

var EnumValuesColor = map[string]Color{
	"Red":   ColorRed,
	"Green": ColorGreen,
	"Blue":  ColorBlue,
}

func (v Color) String() string {
	if s, ok := EnumNamesColor[v]; ok {
		return s
	}
	return "Color(" + strconv.FormatInt(int64(v), 10) + ")"
}
