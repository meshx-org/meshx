package main

import (
	"os"
	"path/filepath"
	"strings"
)

// Manifest describes the list of files that are to become the contents of a package
type Manifest struct {
	// Srcs is a set of manifests and/or directories that are the contents of the package
	Src string

	// Paths is the fully computed contents of a package in the form of "destination": "source"
	Paths map[string]string
}

// NewManifest initializes a manifest from the given paths. If a path is a
// directory, it is globbed and the manifest includes all unignored files under
// that directory.
func NewManifest(path string) (*Manifest, error) {
	manifest := &Manifest{
		Src:   path,
		Paths: make(map[string]string),
	}

	var newPaths map[string]string
	newPaths, err := walkRoot(path)

	if err != nil {
		return nil, err
	}

	for k, v := range newPaths {
		manifest.Paths[k] = v
	}

	return manifest, nil
}

// Package loads the package descriptor from the package listed in the manifest and returns it.
/*func (m *Manifest) Info() (*pkg.PackageInfo, error) {
	f, err := os.Open(m.Paths["meta/info"])
	if err != nil {
		return nil, fmt.Errorf("build.Manifest.Package: %s", err)
	}
	defer f.Close()

	var info pkg.PackageInfo
	
	if err := json.NewDecoder(f).Decode(&info); err != nil {
		return nil, fmt.Errorf("Decode() failed: %v", err)
	}

	if err := info.Validate(); err != nil {
		return nil, fmt.Errorf("Validate() failed: %v", err)
	}

	return &info, nil
}*/

// Meta provides the list of files from the manifest that are to be included in
// meta.tar.
func (manifest *Manifest) Meta() map[string]string {
	meta := map[string]string{}

	for dest, src := range manifest.Paths {
		if strings.HasPrefix(dest, "meta/") {
			meta[dest] = src
		}
	}

	return meta
}

// Content returns the list of files from the manifest that are not to be
// included in the meta.tar.
func (m *Manifest) Content() map[string]string {
	content := map[string]string{}

	for d, s := range m.Paths {
		if !strings.HasPrefix(d, "meta/") {
			content[d] = s
		}
	}

	return content
}

func walkRoot(root string) (map[string]string, error) {
	r := map[string]string{}

	err := filepath.Walk(root, func(source string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		dest, err := filepath.Rel(root, source)

		if err != nil {
			return err
		}

		if info.IsDir() {
			return nil
		}

		r[dest] = source

		return nil
	})

	return r, err
}
