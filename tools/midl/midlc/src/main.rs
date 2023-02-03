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
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command as Cmd, Stdio};

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
                .arg(
                    arg!(--lang)
                        .action(ArgAction::Set)
                        .required(true)
                        .long("lang")
                        .short('l')
                        .value_parser(["ts", "rust"])
                        .default_value("ts")
                        // .num_args(0..=1)
                        .require_equals(true),
                )
                .arg(arg!(<PATH> ... "Stuff to add").value_parser(clap::value_parser!(PathBuf))),
        )
}

trait FromIDL<T> {
    fn from_idl(value: &Value) -> T;
}

impl FromIDL<Library> for Library {
    fn from_idl(value: &Value) -> Library {
        let lib_name = value.get("name");
        let lib_doc = value.get("doc");

        if let None = lib_name {
            panic!("library.name can't be undefined")
        }

        Library {
            name: lib_name.unwrap().to_string(),
            doc: if lib_doc.is_some() {
                Some(lib_doc.unwrap().to_string())
            } else {
                None
            },
        }
    }
}

fn parse_midl_file(value: Value) -> Root {
    println!("{:#?}", value);

    let library_raw = value.get("library");
    if let None = library_raw {
        panic!("library can't be undefined")
    }

    let library_raw = library_raw.unwrap();

    let uses = value.as_object().expect("").get("use").unwrap();
    let uses_is_object = uses.is_object();
    let uses_is_array = uses.is_array();
    println!("{} {}", uses_is_object, uses_is_array);

    Root {
        name: EncodedLibraryIdentifier("".to_owned()),
        library: Library::from_idl(library_raw),
        r#const: vec![],
        r#enum: vec![],
        r#struct: vec![],
        r#protocol: vec![],
    }
}

static LIBRARY: &str = r#"
library test.test

@doc(added = "test")
const test :array(u8) = "test"

@db.test(added = "test")
struct test {
   test :u8 @test, /// this is a doc
   test1 :u8
}
"#;

fn main() -> std::io::Result<()> {
    let mut diagnotics = Diagnostics::new();
    let pairs = MIDLParser::parse(parse::Rule::library, LIBRARY).unwrap();
    let ast = parse(pairs, &mut diagnotics);
    println!("{:?}", ast);

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("compile", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            let hcl_source = std::fs::read_to_string(paths[0])?;
            let value = hcl::from_str(&hcl_source).unwrap();

            let ir = parse_midl_file(value);

            // let value2: Root = serde_json::from_value(value.clone())?;
            // println!("{:#?}", value2);

            let ir_str = serde_json::to_string(&ir)?;

            let mut child = Cmd::new("midlgen_rust")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to execute child");

            let child_stdin = child.stdin.as_mut().unwrap();
            child_stdin.write_all(ir_str.as_bytes())?;

            // Close stdin to finish and avoid indefinite blocking
            drop(child_stdin);

            let output = child.wait_with_output()?;
            println!("Compling paths = {:?} output = {:?}", paths, output);

            Ok(())
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
