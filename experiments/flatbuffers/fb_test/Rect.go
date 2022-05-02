// Code generated by the FlatBuffers compiler. DO NOT EDIT.

package fb_test

import (
	flatbuffers "github.com/google/flatbuffers/go"
)

type Rect struct {
	_tab flatbuffers.Table
}

func GetRootAsRect(buf []byte, offset flatbuffers.UOffsetT) *Rect {
	n := flatbuffers.GetUOffsetT(buf[offset:])
	x := &Rect{}
	x.Init(buf, n+offset)
	return x
}

func GetSizePrefixedRootAsRect(buf []byte, offset flatbuffers.UOffsetT) *Rect {
	n := flatbuffers.GetUOffsetT(buf[offset+flatbuffers.SizeUint32:])
	x := &Rect{}
	x.Init(buf, n+offset+flatbuffers.SizeUint32)
	return x
}

func (rcv *Rect) Init(buf []byte, i flatbuffers.UOffsetT) {
	rcv._tab.Bytes = buf
	rcv._tab.Pos = i
}

func (rcv *Rect) Table() flatbuffers.Table {
	return rcv._tab
}

func (rcv *Rect) Width() uint32 {
	o := flatbuffers.UOffsetT(rcv._tab.Offset(4))
	if o != 0 {
		return rcv._tab.GetUint32(o + rcv._tab.Pos)
	}
	return 0
}

func (rcv *Rect) MutateWidth(n uint32) bool {
	return rcv._tab.MutateUint32Slot(4, n)
}

func (rcv *Rect) Height() uint32 {
	o := flatbuffers.UOffsetT(rcv._tab.Offset(6))
	if o != 0 {
		return rcv._tab.GetUint32(o + rcv._tab.Pos)
	}
	return 0
}

func (rcv *Rect) MutateHeight(n uint32) bool {
	return rcv._tab.MutateUint32Slot(6, n)
}

func RectStart(builder *flatbuffers.Builder) {
	builder.StartObject(2)
}
func RectAddWidth(builder *flatbuffers.Builder, width uint32) {
	builder.PrependUint32Slot(0, width, 0)
}
func RectAddHeight(builder *flatbuffers.Builder, height uint32) {
	builder.PrependUint32Slot(1, height, 0)
}
func RectEnd(builder *flatbuffers.Builder) flatbuffers.UOffsetT {
	return builder.EndObject()
}
