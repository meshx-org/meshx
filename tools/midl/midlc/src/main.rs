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
        library: Library::from_idl(library_raw),
        r#const: vec![],
        r#enum: vec![],
        r#struct: vec![],
        r#protocol: vec![],
    }
}

static HCL: &str = r#"
library {
  name = "meshx.sys"
}

use {
  lib = meshx.sys
  as  = sys
}

protocol Launcher {
  discoverable = true
  doc          = <<-EOT
  An interface for creating component instances.
  Typically obtained via `Environment.GetLauncher`.
  EOT

  method CreateComponent {
    doc = <<-EOT
    Creates a new instance of the component described by `launch_info`.

    The component instance is created in the `Environment`
    associated with this `Launcher`. When creating the component,
    the environment requests the environment services for this component from
    its `EnvironmentHost`.

    The `controller` can be used to control the lifecycle of the created
    component instance. If an `ComponentController`'s interface is
    requested, the component instance is killed when the interface is closed.
    EOT

    request {
      method launch_info {
        type = meshx.sys.LaunchInfo
      }
      method controller {
        optional = true
        type     = fx.server_end
        of {
          type = meshx.sys.ComponentController
        }
      }
    }
  }
}
"#;

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

            let value = hcl::from_str(&HCL).unwrap();

            let ir = parse_midl_file(value);

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
