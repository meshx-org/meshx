package main

import (
	"context"
	"fmt"
	"os"
	"time"

	"github.com/meshx-org/meshx/pkg/fiber"
)

func initProcess(args ...interface{}) {
	proc := InitProcess{counter: 0}

	ctx := context.Background()
	ctx, _ = context.WithCancel(ctx)
	ctx, _ = context.WithTimeout(ctx, time.Microsecond*100)

	proc.Start(ctx, args[0].(*fiber.Handle))
}

func secondProcess(args ...interface{}) {
	proc := SecondProc{}

	ctx := context.Background()
	ctx, _ = context.WithCancel(ctx)
	ctx, _ = context.WithTimeout(ctx, time.Microsecond*100)

	proc.Start(ctx, args[0].(*fiber.Handle))

}

/*
pop data(function name) -> native -> pop data (wasm module) -> wasm -> pop data -> quickjs
*/

func main() {
	natives := map[string]fiber.NativeFn{
		"proc/init":   initProcess,
		"proc/second": secondProcess,
		"proc/wasm": func(args ...interface{}) {
			fmt.Println("wasm_instance")

		},
	}

	kernel, err := fiber.NewKernel(natives)
	fiber.GKernel = &kernel

	if err != nil {
		os.Exit(1)
	}

	err = kernel.RootProcess()

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
