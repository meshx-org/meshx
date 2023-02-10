//! The MIDL Schema AST.

#![deny(rust_2018_idioms, unsafe_code)]

mod ast;
mod diagnotics;
mod error;
mod parse;

use clap::{arg, ArgAction, Command};
use midlgen::{EncodedLibraryIdentifier, Library, Root};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[macro_use]
extern crate pest_derive;

use crate::diagnotics::Diagnostics;
use crate::parse::{parse, MIDLParser};

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

            let midl_source = std::fs::read_to_string(paths[0])?;

            let mut diagnotics = Diagnostics::new();
            let pairs = MIDLParser::parse(parse::Rule::library, &midl_source.as_str()).unwrap();
            let ast = parse(pairs, &mut diagnotics);
            println!("{:?}", ast);

            let ir = Root {
                library: midlgen::Library {
                    name: "S".to_owned(),
                    doc: None,
                },
                r#const: vec![],
                r#enum: vec![],
                r#struct: vec![],
                r#protocol: vec![],
            };

            // let value2: Root = serde_json::from_value(value.clone())?;
            // println!("{:#?}", value2);

            let ir_str = serde_json::to_string(&ir)?;

            let mut file = File::create("./ir.json")?;
            file.write_all(ir_str.as_bytes())?;
            file.flush()?;

            Ok(())
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
