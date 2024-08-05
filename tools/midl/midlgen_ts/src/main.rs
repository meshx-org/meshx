mod compile;
mod types;

use clap::Parser;
use midlgen::ir;
use std::io::Write;
use std::process::Command;
use std::{fs::File, path::PathBuf};
use tera::{Context, Tera};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("render error")]
    TemplateError(#[from] tera::Error),

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
    tera: Tera,
}

static LIBRARY_TEMPLATE: &'static str = include_str!("templates/library.jinja");
static CONTST_TEMPLATE: &'static str = include_str!("templates/const.jinja");
static ENUM_TEMPLATE: &'static str = include_str!("templates/enum.jinja");
static UNION_TEMPLATE: &'static str = include_str!("templates/union.jinja");
static STRUCT_TEMPLATE: &'static str = include_str!("templates/struct.jinja");
static PROTOCOL_TEMPLATE: &'static str = include_str!("templates/protocol.jinja");
static PACKAGE_JSON_TEMPLATE: &'static str = include_str!("templates/package.json.jinja");

impl Generator {
    fn new() -> Self {
        let mut tera = Tera::default();
        tera.add_raw_template("library.jinja", &LIBRARY_TEMPLATE);
        tera.add_raw_template("const.jinja", &CONTST_TEMPLATE);
        tera.add_raw_template("enum.jinja", &ENUM_TEMPLATE);
        tera.add_raw_template("union.jinja", &UNION_TEMPLATE);
        tera.add_raw_template("struct.jinja", &STRUCT_TEMPLATE);
        tera.add_raw_template("protocol.jinja", &PROTOCOL_TEMPLATE);
        tera.add_raw_template("package.json.jinja", &PACKAGE_JSON_TEMPLATE);

        Generator { tera }
    }

    fn generate_midl(&self, ir: ir::Root) -> Result<(), GeneratorError> {
        let tree = compile::compile(ir);
        return self.generate_file(tree);
    }

    fn generate_file(&self, data: types::Root) -> Result<(), GeneratorError> {
        //let path = PathBuf::from(filename.clone());
        //let dir = path.parent().unwrap();
        //std::fs::create_dir_all(dir)?;
        std::fs::create_dir_all(&data.library_name)?;

        let file = File::create(format!("./{}/index.ts", &data.library_name))?;
        let context = Context::from_serialize(&data)?;
        self.tera.render_to("library.jinja", &context, file)?;

        let package_json = File::create(format!("./{}/package.json", &data.library_name))?;
        self.tera.render_to("package.json.jinja", &context, package_json)?;

        let result = Command::new("swc")
            .args([
                format!("./{}/index.ts", &data.library_name).as_str(),
                "--no-swcrc",
                "-o",
                format!("./{}/index.js", &data.library_name).as_str(),
                "--source-map-target",
                format!("./{}/index.map.js", &data.library_name).as_str(),
                "-C", 
                "jsc.parser.syntax=typescript"
            ])
            .output()
            .expect("failed to execute process");

        println!("{:?}", result);

        // TODO: formating
        // let formatted, err = gen.formatter.Format(generated)
        // if err != nil {
        //     return fmt.Errorf("Error formatting source: %w", err)
        //0 }

        // return WriteFileIfChanged(filename, formatted)

        Ok(())
    }
}

fn main() -> Result<(), GeneratorError> {
    let args = Args::parse();
    let contents = std::fs::read_to_string(args.json).unwrap();

    let root = serde_json::from_str::<ir::Root>(contents.as_str())?;

    let generator = Generator::new();
    generator.generate_midl(root)?;

    Ok(())
}
