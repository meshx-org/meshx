package utils

import (
	"errors"
	"fmt"
	"strings"

	wt "github.com/bytecodealliance/wasmtime-go"
)

func NamespaceMatchesFilter(namespace string, name string, namespaceFilter []string) bool {
	fullName := fmt.Sprintf("%s::%s", namespace, name)

	// Allow if any of the allowed namespaces matches the beginning of the full name.
	for _, allowed := range namespaceFilter {
		if strings.HasPrefix(fullName, allowed) {
			return true
		}
	}

	return false
}

func GetMemory(caller *wt.Caller) (*wt.Memory, error) {
	extern := caller.GetExport("memory")

	if extern == nil {
		return nil, errors.New("no export `memory` found")
	}

	memory := extern.Memory()

	if memory == nil {
		return nil, errors.New("export `memory` is not a memory")
	}

	return memory, nil
}

func ErrorToTrap(err error) *wt.Trap {
	message := fmt.Sprintf("trap: %s\n", err.Error())
	return wt.NewTrap(message)
}
