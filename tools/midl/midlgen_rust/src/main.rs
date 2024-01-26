// Copyright 2024 MeshX Authors. All rights reserved.

mod compile;
mod types;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use handlebars::Handlebars;

use thiserror::Error;

use midlgen::helpers::{CommentsHelper, IfCondHelper, PrintfHelper};
use midlgen::ir;
use serde_json::Value;

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

handlebars::handlebars_helper!(doc_comments: |*args| {

    //let ir: Vec<ir::Attribute> = serde_json::from_value(args[0].clone()).unwrap();
     

    vec![] as Vec<String>
});

handlebars::handlebars_helper!(reverse: |arg0: Value| {
    let array = arg0.clone();
    let mut array = array.as_array().unwrap().clone();
    array.reverse();
    Value::Array(array)
});

handlebars::handlebars_helper!(unknown_value_for_tmpl: |*args| {
    String::from("")
});

impl Generator {
    fn new() -> Self {
        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);

        let protocol_declaration_tpl = include_str!("./templates/protocol.hbs");

        registry.register_helper("if_cond", Box::new(IfCondHelper {}));
        registry.register_helper("printf", Box::new(PrintfHelper {}));
        registry.register_helper("reverse", Box::new(reverse));
        registry.register_helper("doc_comments", Box::new(doc_comments));
        registry.register_helper("unknown_value_for_tmpl", Box::new(unknown_value_for_tmpl));

        registry
            .register_template_string("GenerateSourceFile", include_str!("./templates/sourcefile.hbs"))
            .unwrap();

        registry
            .register_partial("Const", include_str!("./templates/const.hbs"))
            .unwrap();

        registry
            .register_partial("Enum", include_str!("./templates/enum.hbs"))
            .unwrap();

        registry
            .register_partial("Union", include_str!("./templates/union.hbs"))
            .unwrap();

        registry
            .register_partial("UnionInternal", include_str!("./templates/union_internal.hbs"))
            .unwrap();

        registry
            .register_partial("Table", include_str!("./templates/table.hbs"))
            .unwrap();

        registry
            .register_partial("TableInternal", include_str!("./templates/table_internal.hbs"))
            .unwrap();

        registry
            .register_partial("EnumInternal", include_str!("./templates/enum_internal.hbs"))
            .unwrap();

        registry
            .register_partial("Struct", include_str!("./templates/struct.hbs"))
            .unwrap();

        registry
            .register_partial("StructInternal", include_str!("./templates/struct_internal.hbs"))
            .unwrap();

        registry.register_partial("Protocol", protocol_declaration_tpl).unwrap();

        Generator { registry }
    }

    fn generate_midl(&self, ir: ir::Root, output_filename: PathBuf) -> Result<(), GeneratorError> {
        let tree = compile::compile(ir);
        return self.generate_file(output_filename, "GenerateSourceFile", tree);
    }

    fn generate_file(&self, filename: PathBuf, tmpl: &str, data: types::Root) -> Result<(), GeneratorError> {
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
#[command(name = "MIDL generator for rust")]
struct Args {
    #[arg(short, long)]
    json: String,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Set)]
    out: PathBuf,
}

fn main() -> Result<(), GeneratorError> {
    let args = Args::parse();
    
    let contents = std::fs::read_to_string(args.json).unwrap();

    let root = serde_json::from_str::<ir::Root>(contents.as_str())?;

    let generator = Generator::new();
    generator.generate_midl(root, args.out)?;

    Ok(())
}
