package main

import (
	"context"
	"log"
	"os"

	"github.com/meshx-org/meshx/pkg/go/fiber"
)

type InitProcess struct {
	counter uint32
}

func (process InitProcess) Start(ctx context.Context, bootstrap *fiber.Handle) error {
	logger := log.New(os.Stdout, "[init] ", log.Lmicroseconds|log.Lmsgprefix)
	
	logger.Printf("Started InitProcess %#v", bootstrap)

	// program init phase
	program, err := fiber.NewProgram()
	childProcess, err := fiber.NewProcess("test", program.Handle().Load())

	if err != nil {
		logger.Printf("Failed to create new process")
		return err
	}

	_, clientEnd, err := fiber.NewChannel()

	childProcess.Start(clientEnd.Handle())

	// process loop

	for {
		select {
		case <-ctx.Done():
			logger.Printf("Finished InitProcess")
			return nil
		default:
			logger.Printf("test")
		}
	}
}
