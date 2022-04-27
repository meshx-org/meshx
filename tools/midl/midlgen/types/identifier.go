package types

import (
	"fmt"
	"strings"
)

type Identifier string

type LibraryIdentifier []Identifier

type CompoundIdentifier struct {
	Library LibraryIdentifier
	Name    Identifier
	Member  Identifier
}

type EncodedLibraryIdentifier string

type EncodedCompoundIdentifier string

func (li LibraryIdentifier) Encode() EncodedLibraryIdentifier {
	ss := make([]string, len(li))
	for i, s := range li {
		ss[i] = string(s)
	}
	return EncodedLibraryIdentifier(strings.Join(ss, "."))
}

func (ci CompoundIdentifier) EncodeDecl() EncodedCompoundIdentifier {
	return EncodedCompoundIdentifier(string(ci.Library.Encode()) + "/" + string(ci.Name))
}

func (ci CompoundIdentifier) Encode() EncodedCompoundIdentifier {
	if ci.Member != "" {
		return EncodedCompoundIdentifier(fmt.Sprintf("%s.%s", ci.EncodeDecl(), ci.Member))
	}
	return ci.EncodeDecl()
}

func (eli EncodedLibraryIdentifier) Parts() LibraryIdentifier {
	return ParseLibraryName(eli)
}

func (eli EncodedLibraryIdentifier) PartsReversed() []string {
	parts := eli.Parts()
	partsReversed := make([]string, len(parts))
	for i, part := range parts {
		partsReversed[len(parts)-i-1] = string(part)
	}

	return partsReversed
}

func (eci EncodedCompoundIdentifier) Parts() CompoundIdentifier {
	return ParseCompoundIdentifier(eci)
}

func (eci EncodedCompoundIdentifier) LibraryName() EncodedLibraryIdentifier {
	parts := strings.SplitN(string(eci), "/", 2)
	raw_library := ""
	if len(parts) == 2 {
		raw_library = parts[0]
	}
	return EncodedLibraryIdentifier(raw_library)
}

func ParseLibraryName(eli EncodedLibraryIdentifier) LibraryIdentifier {
	raw_parts := strings.Split(string(eli), ".")
	parts := make([]Identifier, len(raw_parts))
	for i, raw_part := range raw_parts {
		parts[i] = Identifier(raw_part)
	}
	return LibraryIdentifier(parts)
}

func ParseCompoundIdentifier(eci EncodedCompoundIdentifier) CompoundIdentifier {
	parts := strings.SplitN(string(eci), "/", 2)
	raw_library := ""
	raw_name := parts[0]
	if len(parts) == 2 {
		raw_library = parts[0]
		raw_name = parts[1]
	}
	library := ParseLibraryName(EncodedLibraryIdentifier(raw_library))
	name_parts := strings.SplitN(raw_name, ".", 2)
	name := Identifier(name_parts[0])
	member := Identifier("")
	if len(name_parts) == 2 {
		member = Identifier(name_parts[1])
	}
	return CompoundIdentifier{library, name, member}
}