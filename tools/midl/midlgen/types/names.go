package types

import (
	"strings"
	"unicode"
	"unicode/utf8"
)

// Break a name into parts, discarding underscores but not changing case
func nameParts(name string) []string {
	var parts []string
	for _, namePart := range strings.Split(name, "_") {
		partStart := 0
		lastRune, _ := utf8.DecodeRuneInString(namePart)
		lastRuneStart := 0
		for i, curRune := range namePart {
			if i == 0 {
				continue
			}
			if unicode.IsUpper(curRune) && !unicode.IsUpper(lastRune) {
				parts = append(parts, namePart[partStart:i])
				partStart = i
			}
			if !(unicode.IsUpper(curRune) || unicode.IsDigit(curRune)) && unicode.IsUpper(lastRune) && partStart != lastRuneStart {
				parts = append(parts, namePart[partStart:lastRuneStart])
				partStart = lastRuneStart
			}
			lastRuneStart = i
			lastRune = curRune
		}
		parts = append(parts, namePart[partStart:])
	}
	return parts
}

// Convert text to snake_case style
func ToSnakeCase(name string) string {
	parts := nameParts(name)
	for i := range parts {
		parts[i] = strings.ToLower(parts[i])
	}
	return strings.Join(parts, "_")
}
