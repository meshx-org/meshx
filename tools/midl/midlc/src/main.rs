//! The MIDL Schema AST.

#![deny(rust_2018_idioms, unsafe_code)]
#![feature(map_try_insert)]
#![feature(iter_next_chunk)]
#![feature(option_result_contains)]
#![feature(extend_one)]

mod ast;
mod database;
mod diagnotics;
mod error;
mod parse;
mod source_file;

use clap::{ArgAction, Command};

use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

#[macro_use]
extern crate pest_derive;

use crate::database::{Libraries, ParserDatabase};
use crate::diagnotics::pretty_print_error_text;
use crate::error::ErrorColorer;
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
                .arg(
                    clap::Arg::new("OUT")
                        .long("out")
                        .short('o')
                        .value_parser(clap::value_parser!(PathBuf))
                        .action(ArgAction::Set),
                )
                .arg(
                    clap::Arg::new("FILES")
                        .long("files")
                        .value_parser(clap::value_parser!(PathBuf))
                        .value_delimiter(' ')
                        .num_args(1..)
                        .action(ArgAction::Append),
                ),
        )
}

fn main() -> std::io::Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("compile", sub_matches)) => {
            let paths = sub_matches.get_occurrences::<PathBuf>("FILES").unwrap();
            let output = sub_matches.get_one::<PathBuf>("OUT").unwrap();

            let mut stdout = Box::new(std::io::stdout()) as Box<dyn Write>;
            let mut sources = vec![];

            paths.collect::<Vec<_>>().into_iter().for_each(|path| {
                let lib_sources = path
                    .map(|path| {
                        println!("read file: {}", path.file_name().unwrap().to_str().unwrap());

                        let source_file = SourceFile::new(path.as_path());

                        match source_file {
                            Ok(source_file) => source_file,
                            Err(e) => {
                                pretty_print_error_text(
                                    &mut stdout,
                                    format!("couldn't read {}: {}", path.file_name().unwrap().to_str().unwrap(), e)
                                        .as_str(),
                                    &ErrorColorer {},
                                );
                                std::process::exit(1);
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                let source_files = SourceFiles::from(lib_sources);
                sources.push(source_files);
            });

            let all_libraries = Rc::new(RefCell::from(Libraries::new()));
            let mut success = true;

            for source_files in sources {
                log::info!(
                    "Compiling files: {:#?}",
                    source_files.files.iter().map(|f| f.filename()).collect::<Vec<_>>()
                );

                let compiler = ParserDatabase::new(all_libraries.clone()).unwrap();

                for (source_id, source) in source_files.iter() {
                    log::info!("Parsing {:?}", source.filename());

                    let diagnostics = compiler.parse_file(source_id, source);

                    if diagnostics.has_errors() {
                        success = false;
                        diagnostics.errors().iter().for_each(|e| {
                            let source = &source_files[e.span().source];
                            e.pretty_print(&mut stdout, source.filename(), source.as_str()).unwrap();
                        });
                    }
                }

                let diagnostics = compiler.compile();

                if diagnostics.has_errors() {
                    success = false;
                    diagnostics.errors().iter().for_each(|e| {
                        let source = &source_files[e.span().source];
                        let mut stdout = Box::new(std::io::stdout()) as Box<dyn Write>;
                        e.pretty_print(&mut stdout, source.filename(), source.as_str()).unwrap();
                    });
                }
            }

            // println!("Success: {:#?}", all_libraries);

            if !success {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Compilation failed"));
            }

            let ir = midlgen::ir::Root {
                name: "S".to_owned(),
                documentation: None,
                attributes: vec![],

                table_declarations: vec![],
                const_declarations: vec![],
                enum_declarations: vec![],
                struct_declarations: vec![],
                protocol_declarations: vec![],
                union_declarations: vec![],
                bits_declarations: vec![],
            };

            println!("IR: {:#?}", output.clone().display());

            let ir_str = serde_json::to_string(&ir)?;
            std::fs::create_dir_all(output.clone().parent().unwrap())?;

            let mut file = File::create(output)?;
            file.write_all(ir_str.as_bytes())?;
            file.flush()?;

            Ok(())
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
