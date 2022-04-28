package model

// The ModelContext provides the API boundary between the Model and Realms. It
// defines what parts of the Model or authoritative state about the tree we
// want to share with Realms.
type ModelContext struct {
	policyChecker    GlobalPolicyChecker
	componentIdIndex ComponentIdIndex
	runtimeConfig    RuntimeConfig
}

func NewModelContext(runtimeConfig RuntimeConfig) ModelContext {
	componentIdIndex := NewComponentIdIndex(runtimeConfig.componentIdIndexPath)

	context := ModelContext{
		componentIdIndex: componentIdIndex,
		runtimeConfig:    runtimeConfig,
		policyChecker:    NewGlobalPolicyChecker(runtimeConfig),
	}

	return context
}
