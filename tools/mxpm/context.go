package main

type PkgContext struct {
	Name    string
	Version string

	OutputDir string
	Cwd       string

	// the manifest is memoized lazily, on the first call to Manifest()
	manifest *Manifest
}

// Manifest initializes and returns the configured manifest. The manifest may be
// modified during the build process to add/remove files.
func (ctx *PkgContext) Manifest() (*Manifest, error) {
	var err error

	if ctx.manifest == nil {
		ctx.manifest, err = NewManifest(ctx.Cwd)
	}

	return ctx.manifest, err
}
