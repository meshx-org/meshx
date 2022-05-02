package midlgen

import (
	"bytes"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"text/template"
)

type Generator struct {
	tmpls *template.Template
}

// NewGenerator creates a new fidlgen Generator, given a name, a system of Go
// .tmpl files dictating the generation (likely deriving from a go:embed
// directive), a formatter for the generated source, and a template function map.
func NewGenerator(name string, tmplFS fs.FS, funcs template.FuncMap) *Generator {
	gen := &Generator{
		template.New(name),
	}

	gen.tmpls.Funcs(funcs)

	// The text/template package does not make it easy for us to populate the
	// template parse tree from our abstracted filesystem. In order to do this,
	// we must manually walk the filesystem ourselves to pick out the .tmpl
	// files, and then pass those along to template.ParseFS() as exact globs.
	files, err := walkTemplateFiles(tmplFS)
	if err != nil {
		panic(err)
	}
	template.Must(gen.tmpls.ParseFS(tmplFS, files...))

	return gen
}

func walkTemplateFiles(tmplFS fs.FS) ([]string, error) {
	var tmpls []string

	err := fs.WalkDir(tmplFS, ".", func(path string, _ fs.DirEntry, err error) error {
		if err != nil {
			return nil
		}
		if filepath.Ext(path) == ".tmpl" {
			tmpls = append(tmpls, path)
		}
		return nil
	})

	if err != nil {
		return nil, err
	}
	return tmpls, nil
}

func (gen *Generator) ExecuteTemplate(tmpl string, data interface{}) ([]byte, error) {
	buf := new(bytes.Buffer)
	err := gen.tmpls.ExecuteTemplate(buf, tmpl, data)
	if err == nil {
		return buf.Bytes(), nil
	}
	return nil, err
}

func (gen *Generator) GenerateFile(filename string, tmpl string, data interface{}) error {
	err := os.MkdirAll(filepath.Dir(filename), os.ModePerm)
	if err != nil {
		return err
	}

	generated, err := gen.ExecuteTemplate(tmpl, data)
	if err != nil {
		return fmt.Errorf("Error generating content: %w", err)
	}

	// TODO(formater)
	//formatted, err := gen.formatter.Format(generated)
	// if err != nil {
	//	return fmt.Errorf("Error formatting source: %w", err)
	// }

	// overwrite
	if err := os.MkdirAll(filepath.Dir(filename), os.FileMode(0777)); err != nil {
		return err
	}
	return os.WriteFile(filename, generated, os.FileMode(0666))
}
