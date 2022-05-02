package main

import (
	"strings"
	"testing"
)

func TestManifestMeta(t *testing.T) {
	m := &Manifest{
		Paths: map[string]string{
			"meta/package":  "",
			"meta/contents": "",
			"alpha":         "",
			"beta":          "",
		},
	}

	if got, want := len(m.Meta()), 2; got != want {
		t.Errorf("got %d, want %d", got, want)
	}

	for k := range m.Meta() {
		if !strings.HasPrefix(k, "meta/") {
			t.Errorf("found non-meta file in metas: %q", k)
		}
	}
}

func TestManifestContent(t *testing.T) {
	m := &Manifest{
		Paths: map[string]string{
			"meta/package":  "",
			"meta/contents": "",
			"alpha":         "",
			"beta":          "",
		},
	}
	if got, want := len(m.Meta()), 2; got != want {
		t.Errorf("got %d, want %d", got, want)
	}
	for k := range m.Content() {
		if strings.HasPrefix(k, "meta/") {
			t.Errorf("found meta file in contents: %q", k)
		}
	}
}
