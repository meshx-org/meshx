package main

import (
	"bufio"
	"context"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"sync"

	"github.com/spf13/cobra"
)

// MerkleRoot is the root hash of a merkle tree
type MerkleRoot [32]byte

// MetaContents maps file paths within a package to their content IDs
type MetaContents map[string]MerkleRoot

// String serializes the instance in the manifest file format, which could be
// parsed by ParseMetaContents.
func (meta MetaContents) String() string {
	paths := make([]string, 0, len(meta))
	for path, _ := range meta {
		paths = append(paths, path)
	}
	sort.Strings(paths)
	contentLines := make([]string, 0, len(meta))
	for _, path := range paths {
		root := meta[path]
		line := fmt.Sprintf("%s=%s\n", path, root)
		contentLines = append(contentLines, line)
	}
	return strings.Join(contentLines, "")
}

func Sync(ctx *PkgContext) error {
	var err error

	metadir := filepath.Join(fCwd, "meta")
	os.MkdirAll(metadir, os.ModePerm)

	manifest, err := ctx.Manifest()
	check(err)

	contentsPath := filepath.Join(metadir, "contents")
	pkgContents := manifest.Content()

	// s

	// manifestLines is a channel containing unpacked manifest paths
	var manifestLines = make(chan struct{ src, dest string }, len(pkgContents))
	go func() {
		for dest, src := range pkgContents {
			manifestLines <- struct{ src, dest string }{src, dest}
		}
		close(manifestLines)
	}()

	// contentCollector receives entries to include in contents
	type contentEntry struct {
		path string
		root MerkleRoot
	}

	var contentCollector = make(chan contentEntry, len(pkgContents))
	var errors = make(chan error)
	// w is a group that is done when contentCollector is fully populated
	var w sync.WaitGroup
	for i := runtime.NumCPU(); i > 0; i-- {
		w.Add(1)
		go func() {
			defer w.Done()
			for in := range manifestLines {
				var tree merkle.Tree
				cf, err := os.Open(in.src)
				if err != nil {
					errors <- fmt.Errorf("build.Update: open %s for %s: %s", in.src, in.dest, err)
					return
				}
				_, err = tree.ReadFrom(bufio.NewReader(cf))
				cf.Close()
				if err != nil {
					errors <- err
					return
				}
				var root MerkleRoot
				copy(root[:], tree.Root())
				contentCollector <- contentEntry{in.dest, root}
			}
		}()
	}

	// s

	contents := MetaContents{}

	go func() {
		for entry := range contentCollector {
			contents[entry.path] = entry.root
		}
		close(done)
	}()

	manifest.Paths["meta/contents"] = contentsPath

	return ioutil.WriteFile(contentsPath, []byte(contents.String()), os.ModePerm)
}

/*func BlobInfo(ctx context.Context) ([]PackageBlobInfo, error) {
	manifest, err := c.Manifest()

	if err != nil {
		return nil, err
	}

	var result []PackageBlobInfo
	// Include a meta FAR entry first. If blobs.sizes becomes the new root
	// blob for a package, targets need to know which unnamed blob is the
	// meta FAR.
	{
		merkleBytes, err := ioutil.ReadFile(c.MetaFARMerkle())
		if err != nil {
			return nil, err
		}
		merkle, err := DecodeMerkleRoot(merkleBytes)
		if err != nil {
			return nil, err
		}
		info, err := os.Stat(c.MetaFAR())
		if err != nil {
			return nil, err
		}
		result = append(result, PackageBlobInfo{
			SourcePath: c.MetaFAR(),
			Path:       "meta/",
			Merkle:     merkle,
			Size:       uint64(info.Size()),
		})
	}
	contentsPath := filepath.Join(c.OutputDir, "meta", "contents")
	contents, err := LoadMetaContents(contentsPath)
	if err != nil {
		return nil, err
	}
	contentsKeys := make([]string, 0, len(contents))
	for k := range contents {
		contentsKeys = append(contentsKeys, k)
	}
	sort.Strings(contentsKeys)
	for _, path := range contentsKeys {
		merkle := contents[path]
		info, err := os.Stat(manifest.Paths[path])
		if err != nil {
			return nil, err
		}
		result = append(result, PackageBlobInfo{
			SourcePath: manifest.Paths[path],
			Path:       path,
			Merkle:     merkle,
			Size:       uint64(info.Size()),
		})
	}
	return result, nil
}*/

/*func OutputManifest(ctx context.Context) (*PackageManifest, error) {
	p, err := Package()
	if err != nil {
		return nil, err
	}
	blobs, err := BlobInfo()
	if err != nil {
		return nil, err
	}
	return &PackageManifest{
		Version: "1",
		Package: p,
		Blobs:   blobs,
	}, err
}*/

var (
	syncCmd = &cobra.Command{
		Use:   "build",
		Short: "MXPM is a package manager for MeshX packages",
		Run: func(cmd *cobra.Command, args []string) {
			ctx := context.WithValue(cmd.Context(), "metaPath", filepath.Join(fCwd, "meta"))
			Sync(ctx)
		},
	}
)
