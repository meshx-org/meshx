package types

type AttributeArg struct {
	Name  Identifier `json:"name"`
	Value Constant   `json:"value"`
}

type Attribute struct {
	Name Identifier     `json:"name"`
	Args []AttributeArg `json:"arguments,omitempty"`
}

func (el Attribute) LookupArg(name Identifier) (AttributeArg, bool) {
	for _, a := range el.Args {
		if a.Name == name {
			return a, true
		}
	}
	return AttributeArg{}, false
}

func (el Attribute) HasArg(name Identifier) bool {
	_, ok := el.LookupArg(name)
	return ok
}

// Attributes represents a list of attributes. It conveniently implements the
// `Annotated` protocol, such that it can be embedded into other node structs
// which are annotated.
type Attributes struct {
	Attributes []Attribute `json:"maybe_attributes,omitempty"`
}

func (el Attributes) LookupAttribute(name Identifier) (Attribute, bool) {
	for _, a := range el.Attributes {
		if ToSnakeCase(string(a.Name)) == ToSnakeCase(string(name)) {
			return a, true
		}
	}
	return Attribute{}, false
}

func (el Attributes) HasAttribute(name Identifier) bool {
	_, ok := el.LookupAttribute(name)
	return ok
}
