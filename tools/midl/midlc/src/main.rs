//! The MIDL Schema AST.

#![deny(rust_2018_idioms, unsafe_code)]
#![feature(map_try_insert)]

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

use crate::database::ParserDatabase;
use crate::diagnotics::Diagnostics;
use crate::parse::parse_source;
use crate::source_file::{SourceFile, SourceFiles};

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

            let files: Vec<SourceFile<'_>> = paths
                .iter()
                .map(|path| {
                    let contents = std::fs::read_to_string(path).unwrap();
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    let midl_source = SourceFile::new(filename, contents);
                    midl_source
                })
                .collect();

            println!("Compiling {:?} files", files);

            let source_files = SourceFiles::from(files);
            let db = ParserDatabase::new();
            let mut success = true;

            source_files.iter_sources().for_each(|(source_id, source)| {
                println!("Parsing {:?}", source.filename());



                let diagnostics = db.parse_file(source_id, source);

                if diagnostics.has_errors() {
                    success = false;
                    diagnostics.errors().iter().for_each(|e| {
                        let source = &source_files[e.span().source];
                        let mut stdout = Box::new(std::io::stdout()) as Box<dyn Write>;
                        e.pretty_print(&mut stdout, source.filename(), source.as_str()).unwrap();
                    });
                }
            });

            let diagnostics = db.compile();

            if diagnostics.has_errors() {
                success = false;
                diagnostics.errors().iter().for_each(|e| {
                    let source = &source_files[e.span().source];
                    let mut stdout = Box::new(std::io::stdout()) as Box<dyn Write>;
                    e.pretty_print(&mut stdout, source.filename(), source.as_str()).unwrap();
                });
            }

            if !success {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Compilation failed"));
            }

            let ir_str = serde_json::to_string(db.get_ir())?;

            let mut file = File::create("./ir.json")?;
            file.write_all(ir_str.as_bytes())?;
            file.flush()?;

            Ok(())
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
