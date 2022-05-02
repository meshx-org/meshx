package main

import (
	"archive/tar"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
)

func build(srcFile string, num int) {
	file, err := os.Open(srcFile)
	check(err)
	defer file.Close()

	tarReader := tar.NewReader(file)

	i := 0
	for {
		header, err := tarReader.Next()

		if err == io.EOF {
			break
		}

		check(err)

		name := header.Name

		switch header.Typeflag {
		case tar.TypeDir:
			continue
		case tar.TypeReg:
			fmt.Println("(", i, ")", "Name: ", name)
			if i == num {
				fmt.Println(" --- ")
				io.Copy(os.Stdout, tarReader)
				fmt.Println(" --- ")
				os.Exit(0)
			}
		default:
			fmt.Printf("%s : %c %s %s\n",
				"Yikes! Unable to figure out type",
				header.Typeflag,
				"in file",
				name,
			)
		}

		i++
	}
}

// computedOutputs are files that are produced by the `build` composite command
// that must be excluded from the depfile
var computedOutputs = map[string]struct{}{
	"meta/contents": {},
}

var (
	fCwd    string
	rootCmd = &cobra.Command{
		Use:   "build",
		Short: "MXPM is a package manager for MeshX packages",
		Long:  ``,

		Run: func(cmd *cobra.Command, args []string) {

			file := filepath.Join(fCwd, "test.tar")
			fmt.Println(file)

			build(file, 1)
		},
	}
)

func check(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func main() {
	cwd, err := os.Getwd()
	check(err)

	rootCmd.PersistentFlags().StringVar(&fCwd, "cwd", cwd, "the current working directory")

	rootCmd.AddCommand(syncCmd)

	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
