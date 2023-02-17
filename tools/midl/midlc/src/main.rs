//! The MIDL Schema AST.

#![deny(rust_2018_idioms, unsafe_code)]

mod ast;
mod database;
mod diagnotics;
mod error;
mod parse;
mod source_file;

use clap::{arg, Command};

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[macro_use]
extern crate pest_derive;

use crate::database::{Libraries, ParserDatabase};
use crate::diagnotics::Diagnostics;
use crate::parse::parse_source;
use crate::source_file::SourceFile;

fn cli() -> Command {
    Command::new("git")
        .about("A fictional versioning CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("compile")
                .about("adds things")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Stuff to add").value_parser(clap::value_parser!(PathBuf))),
        )
}

fn main() -> std::io::Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("compile", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            let sources = paths
                .iter()
                .map(|path| {
                    let midl_source: SourceFile = std::fs::read_to_string(path).unwrap().into();
                    midl_source
                })
                .collect::<Vec<_>>();

            println!("Compiling {:?} files", sources);

            let mut diagnostics = Diagnostics::new();

            let mut db = ParserDatabase::new(sources);

            db.compile(&mut diagnostics);

            let ir_str = serde_json::to_string(db.get_ir())?;

            let mut file = File::create("./ir.json")?;
            file.write_all(ir_str.as_bytes())?;
            file.flush()?;

            Ok(())
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
