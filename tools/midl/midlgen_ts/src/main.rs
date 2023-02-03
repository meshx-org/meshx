use handlebars::Handlebars;

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

fn main() -> Result<(), GeneratorError> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();

    let root = serde_json::from_str::<midlgen::Root>(buffer.as_str())?;
    let root = ir::compile(root);

    Ok(())
}
