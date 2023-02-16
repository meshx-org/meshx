mod compile;
mod types;

use std::fs::File;
use std::io::Write;
use std::path::{PathBuf};

use clap::Parser;
use handlebars::Handlebars;

use thiserror::Error;

use midlgen::ir;
use midlgen::helpers::{IfCondHelper, PrintfHelper};

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("render error")]
    TemplateRender(#[from] handlebars::RenderError),

    #[error("template error")]
    Template,

    #[error("unknown generator error")]
    Unknown,
}

struct Generator {
    registry: Handlebars<'static>,
}

impl Generator {
    fn new() -> Self {
        let mut registry = Handlebars::new();
        let generate_source_tpl = include_str!("./templates/sourcefile.hbs");
        let const_declaration_tpl = include_str!("./templates/const.hbs");
        let enum_declaration_tpl = include_str!("./templates/enum.hbs");
        let struct_declaration_tpl = include_str!("./templates/struct.hbs");
        let protocol_declaration_tpl = include_str!("./templates/protocol.hbs");

        registry.register_helper("if_cond", Box::new(IfCondHelper {}));
        registry.register_helper("printf", Box::new(PrintfHelper {}));

        registry
            .register_template_string("GenerateSourceFile", generate_source_tpl)
            .unwrap();

        registry
            .register_partial("ConstDeclaration", const_declaration_tpl)
            .unwrap();

        registry
            .register_partial("EnumDeclaration", enum_declaration_tpl)
            .unwrap();

        registry
            .register_partial("StructDeclaration", struct_declaration_tpl)
            .unwrap();

        registry
            .register_partial("ProtocolDeclaration", protocol_declaration_tpl)
            .unwrap();

        Generator { registry }
    }

    fn generate_fidl(&self, ir: ir::Root, output_filename: String) -> Result<(), GeneratorError> {
        let tree = compile::compile(ir);
        return self.generate_file(output_filename, "GenerateSourceFile", tree);
    }

    fn generate_file(&self, filename: String, tmpl: &str, data: types::Root) -> Result<(), GeneratorError> {
        let path = PathBuf::from(filename.clone());
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir)?;

        let result = self.execute_template(tmpl, data)?;

        let mut file = File::create(filename)?;
        file.write_all(result.as_bytes())?;

        // TODO: formating
        // let formatted, err = gen.formatter.Format(generated)
        // if err != nil {
        //     return fmt.Errorf("Error formatting source: %w", err)
        //0 }

        // return WriteFileIfChanged(filename, formatted)

        Ok(())
    }

    fn execute_template(&self, tmpl: &str, root: types::Root) -> Result<String, GeneratorError> {
        let result = self.registry.render(tmpl, &root)?;
        Ok(result)
    }
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    json: String,
}

fn main() -> Result<(), GeneratorError> {
    let args = Args::parse();
    let contents = std::fs::read_to_string(args.json).unwrap();

    let root = serde_json::from_str::<ir::Root>(contents.as_str())?;

    let generator = Generator::new();
    generator.generate_fidl(root, "./OUTPUT.rs".to_owned())?;

    Ok(())
}
