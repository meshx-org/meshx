mod compile;
mod types;

use handlebars::Handlebars;
use clap::Parser;
use thiserror::Error;
use midlgen::ir;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("render error")]
    TemplateRender(#[from] handlebars::RenderError),

    #[error("unknown generator error")]
    Unknown,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    json: String,
}

struct Generator {
    registry: Handlebars<'static>,
}

impl Generator {
    fn new() -> Self {
        let mut registry = Handlebars::new();
        // let generate_source_tpl = include_str!("./templates/sourcefile.hbs");
        let const_declaration_tpl = include_str!("./templates/const.hbs");
        // let enum_declaration_tpl = include_str!("./templates/enum.hbs");
        let struct_declaration_tpl = include_str!("./templates/struct.hbs");
        // let protocol_declaration_tpl = include_str!("./templates/protocol.hbs");

        registry
            .register_partial("ConstDeclaration", const_declaration_tpl)
            .unwrap();

        registry
            .register_partial("StructDeclaration", struct_declaration_tpl)
            .unwrap();

        Generator { registry }
    }

    fn generate_fidl(&self, _ir: ir::Root, _output_filename: String) -> Result<(), GeneratorError> {
        Ok(())
    }
}

fn main() -> Result<(), GeneratorError> {
    let args = Args::parse();
    let contents = std::fs::read_to_string(args.json).unwrap();

    let root = serde_json::from_str::<ir::Root>(contents.as_str())?;
    println!("{:?}", root);

    // let root = ir::compile(root);

    let generator = Generator::new();
    generator.generate_fidl(root, "./OUTPUT.rs".to_owned())?;

    Ok(())
}
