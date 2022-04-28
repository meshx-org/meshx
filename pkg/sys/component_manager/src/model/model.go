package model

// Parameters for initializing a component model, particularly the root of the component
// instance tree.
type ModelParams struct {
	// TODO: Merge into RuntimeConfig
	// The URL of the root component.
	rootComponentUrl string
	/// The environment provided to the root.
	rootEnvironment Environment
	/// Global runtime configuration for the component_manager.
	runtimeConfig RuntimeConfig
	/// The instance at the top of the tree, representing component manager.
	topInstance ComponentManagerInstance
}

// The component model holds authoritative state about a tree of component instances, including
// each instance's identity, lifecycle, capabilities, and topological relationships.  It also
// provides operations for instantiating, destroying, querying, and controlling component
// instances at runtime.
type Model struct {
	// The instance at the top of the tree, i.e. the instance representing component manager
	// itself.
	topInstance ComponentManagerInstance

	// The instance representing the root component. Owned by `topInstance`, but cached here for
	// efficiency.
	root    ComponentInstance
	context ModelContext
}

func NewModel(params ModelParams) (Model, error) {
	context := NewModelContext(params.runtime_config)

	root := NewRootComponentInstance(
		params.root_environment,
		context,
		params.top_instance,
		params.root_component_url,
	)

	model := Model{
		root: root.clone(),
		context,
		top_instance: params.top_instance,
	}

	model.topInstance.init(root)

	return model, nil
}

func (self Model) Root() ComponentInstance {
	return self.root
}

func (self Model) TopInstance() ComponentManagerInstance {
	return self.topInstance
}

func (self Model) LookUp() (ComponentInstance, error) {
	// TODO: implement
	return nil, nil
}

func (self Model) Start() {
	// TODO: implement
}
