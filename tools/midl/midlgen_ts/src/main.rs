use handlebars::Handlebars;

use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

mod ir;

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

fn main() -> Result<(), GeneratorError> {
    let args = Args::parse();

    let root = serde_json::from_str::<midlgen::Root>(contents.as_str())?;
    println!("{:?}", root);

    let generator = Generator::new();
    generator.generate_fidl(root, "./OUTPUT.rs".to_owned())?;

    let root = ir::compile(root);

    Ok(())
}
