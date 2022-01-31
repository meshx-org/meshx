package main

import (
	"os"

	"github.com/meshx-org/meshx/pkg/fiber"
	"go.uber.org/zap"
)

func main() {
	logger, _ := zap.NewDevelopment()
	zap.ReplaceGlobals(logger)

	kernel, err := fiber.NewKernel()
	fiber.GKernel = &kernel

	if err != nil {
		os.Exit(1)
	}

	err = kernel.BootProcess(InitProcess{counter: 0})

	if err != nil {
		os.Exit(1)
	}

	kernel.Wait()
	// proc := realm.NewProcess()
	// h0, h1 := realm.NewChannel()

	// realm.HandleExport()
	// realm.HandleImport()

	// realm.ProcessStart(proc)

	// fiber, err := fiber.NewFiberClient()
}
