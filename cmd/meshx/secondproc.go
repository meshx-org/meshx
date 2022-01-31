package main

import (
	"context"
	"time"

	"github.com/meshx-org/meshx/pkg/fiber"
	"go.uber.org/zap"
)

type SecondProc struct {
}

func (process SecondProc) Start(ctx context.Context, boot *fiber.Handle) error {

	zap.L().Info("Started SecondProc")
	time.Sleep(1000)
	zap.L().Info("Finished SecondProc")
	return nil
}
