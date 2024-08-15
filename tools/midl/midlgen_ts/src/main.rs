// TODO: remove this once lib is semi stable (i.e outputs usable code)
#![allow(unused)] 

mod compile;
mod types;

use clap::Parser;
use midlgen::ir;
use sheller::run;
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

    #[arg(short, long, action = clap::ArgAction::Set)]
    out: PathBuf,
}

struct Generator {
    out_dir: PathBuf,
    tera: Tera,
}

static LIBRARY_TEMPLATE: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/library.jinja"));
static CONTST_TEMPLATE: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/const.jinja"));
static ENUM_TEMPLATE: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/enum.jinja"));
static UNION_TEMPLATE: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/union.jinja"));
static STRUCT_TEMPLATE: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/struct.jinja"));
static PROTOCOL_TEMPLATE: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/protocol.jinja"));
static PACKAGE_JSON_TEMPLATE: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/package.json.jinja"));
static TSCONFIG_JSON_TEMPLATE: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/tsconfig.json.jinja"));

impl Generator {
    fn new(out_dir: PathBuf) -> Result<Self, GeneratorError> {
        let mut tera = Tera::default();

        tera.add_raw_template("const.jinja", &CONTST_TEMPLATE)?;
        tera.add_raw_template("enum.jinja", &ENUM_TEMPLATE)?;
        tera.add_raw_template("union.jinja", &UNION_TEMPLATE)?;
        tera.add_raw_template("struct.jinja", &STRUCT_TEMPLATE)?;
        tera.add_raw_template("protocol.jinja", &PROTOCOL_TEMPLATE)?;
        tera.add_raw_template("package.json.jinja", &PACKAGE_JSON_TEMPLATE)?;
        tera.add_raw_template("tsconfig.json.jinja", &TSCONFIG_JSON_TEMPLATE)?;

        tera.add_raw_template("library.jinja", &LIBRARY_TEMPLATE)?;

        Ok(Generator { out_dir, tera })
    }

    fn generate_midl(&self, ir: ir::Root) -> Result<(), GeneratorError> {
        let tree = compile::compile(ir);
        self.generate_file(tree)
    }

    fn generate_file(&self, data: types::Root) -> Result<(), GeneratorError> {
        //let path = PathBuf::from(filename.clone());
        //let dir = path.parent().unwrap();
        //std::fs::create_dir_all(dir)?;
        std::fs::create_dir_all(format!("{}/{}", self.out_dir.display(), &data.library_name))?;

        println!(
            "{}",
            format!("{}/{}/index.ts", self.out_dir.display(), &data.library_name)
        );
        let file = File::create(format!("{}/{}/index.ts", self.out_dir.display(), &data.library_name))?;
        let context = Context::from_serialize(&data)?;
        self.tera.render_to("library.jinja", &context, file)?;

        let package_json = File::create(format!(
            "{}/{}/package.json",
            self.out_dir.display(),
            &data.library_name
        ))?;
        self.tera.render_to("package.json.jinja", &context, package_json)?;

        let tsconfig_json = File::create(format!(
            "{}/{}/tsconfig.json",
            self.out_dir.display(),
            &data.library_name
        ))?;
        self.tera.render_to("tsconfig.json.jinja", &context, tsconfig_json)?;

        //let result = run!("tsc -p {}/{}", self.out_dir.display(), &data.library_name);

        //println!("tsc {}/{}", self.out_dir.display(), &data.library_name);
        //println!("status {:?}", result.status.to_string());

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
    let contents = std::fs::read_to_string(args.json)?;

    let root = serde_json::from_str::<ir::Root>(contents.as_str())?;

    let generator = Generator::new(args.out)?;
    generator.generate_midl(root)?;

    Ok(())
}
