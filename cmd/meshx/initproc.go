package main

import (
	"context"
	"fmt"
	"log"

	"github.com/meshx-org/meshx/pkg/fiber"
	"go.uber.org/zap"
)

type InitProcess struct {
	counter uint32
}

func (process InitProcess) Start(ctx context.Context, bootstrap *fiber.Handle) error {
	zap.L().Sugar().Infof("Started InitProcess %v", bootstrap)

	// program init phase

	_, clientEnd, err := fiber.NewChannel()
	childProcess, err := fiber.NewProcess("test", SecondProc{})

	if err != nil {
		fmt.Println("Failed to create new process")
		return err
	}

	childProcess.Start(clientEnd.Handle())

	// program loop

	for {
		select {
		case <-ctx.Done():
			zap.L().Info("Finished InitProcess")
			log.SetPrefix("syscall")
			log.Println("Finished InitProcess")
			return nil
		default:
			// client.Read()
			log.Printf("test")
		}
	}
}
