package main

import (
	"io"

	"github.com/djherbis/buffer"
	"github.com/djherbis/nio"
)

// BUG(r): The rule Title uses for word boundaries does not handle Unicode punctuation properly.
func BufferedPipe() (io.ReadWriteCloser, io.ReadWriteCloser) {
	b1 := buffer.New(32 * 1024)
	b2 := buffer.New(32 * 1024)

	r1, w1 := nio.Pipe(b1)
	r2, w2 := nio.Pipe(b2)

	return &pipe{r1, w2}, &pipe{r2, w1}
}

type pipe struct {
	*nio.PipeReader
	*nio.PipeWriter
}

func (p pipe) Close() error {
	
	return nil
}
