package main

import (
	"context"
	"log"
	"os"
	"time"

	"github.com/meshx-org/meshx/pkg/fiber"
)

type SecondProc struct {
}

func (process SecondProc) Start(ctx context.Context, boot *fiber.Handle) error {
	logger := log.New(os.Stdout, "[init] ", log.Lmicroseconds|log.Lmsgprefix)

	logger.Printf("Started SecondProc")
	time.Sleep(1000)
	logger.Printf("Finished SecondProc")
	return nil
}
