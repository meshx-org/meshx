//! The MIDL Schema AST.

#![deny(rust_2018_idioms, unsafe_code)]
#![feature(map_try_insert)]
#![feature(try_blocks)]
#![feature(iter_next_chunk)]
#![feature(extend_one)]
#![feature(once_cell)]

#[macro_use]
extern crate pest_derive;

mod ast;
mod compiler;
mod consumption;
mod diagnotics;
mod error;
mod generator;
mod source_file;

use clap::{ArgAction, Command};
use core::panic;
use serde_json::Value;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Write, self};
use std::path::PathBuf;
use std::rc::Rc;

use crate::compiler::{Compiler, Libraries};
use crate::diagnotics::{pretty_print_error_text, Diagnostics};
use crate::error::ErrorColorer;
use crate::source_file::{SourceFile, SourceManager};

fn cli() -> Command {
    Command::new("midlc")
        .about("A MIDL file compiler")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("compile")
                .about("adds things")
                .arg_required_else_help(true)
                .arg(clap::Arg::new("NAME").long("name").short('n').action(ArgAction::Set))
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

struct ExperimentalFlags;

fn write(output: Value, file_path: &PathBuf) -> Result<(), io::Error> {
    let ir_str = serde_json::to_string(&output)?;
    std::fs::create_dir_all(file_path.clone().parent().unwrap())?;

    let mut file = File::create(file_path)?;
    file.write_all(ir_str.as_bytes())?;
    file.flush()?;

    Ok(())
}

fn compile(
    source_managers: &Vec<SourceManager<'_>>,
    output: &PathBuf,
    experimental_flags: ExperimentalFlags,
) -> Result<(), std::io::Error> {
    let mut success = true;
    let mut stderr = Box::new(std::io::stderr()) as Box<dyn Write>;
    let all_libraries = Rc::new(RefCell::from(Libraries::new()));

    for manager in source_managers {
        let compiler = Compiler::new(all_libraries.clone()).unwrap();

        log::info!(
            "Compiling files: {:#?}",
            manager.files.iter().map(|f| f.filename()).collect::<Vec<_>>()
        );

        for (source_id, source) in manager.iter() {
            log::info!("Parsing {:?}", source.filename());

            let diagnostics = compiler.consume_file(source_id, source);

            if diagnostics.has_errors() {
                success = false;
                diagnostics.errors().iter().for_each(|e| {
                    let source: &SourceFile<'_> = &manager[e.span().source];
                    e.pretty_print(&mut stderr, source.filename(), source.as_str()).unwrap();
                });
            }
        }

        let mut diagnostics = Diagnostics::new();

        if !compiler.compile(&mut diagnostics) {
            diagnostics.errors().iter().for_each(|e| {
                let source = &manager[e.span().source];
                e.pretty_print(&mut stderr, source.filename(), source.as_str()).unwrap();
            });

            panic!("failed to compile")
        }
    }

    if all_libraries.borrow().is_empty() {
        panic!("No library was produced.\n");
    }

    // log::debug!("all {:#?}", all_libraries);

    if !success {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Compilation failed"));
    }

    let compilation = all_libraries.borrow_mut().filter(0);

    // Verify that the produced library's name matches the expected name.
    /*std::string produced_name = fidlc::NameLibrary(compilation->library_name);
    if (!library_name.empty() && produced_name != library_name) {
        Fail("Generated library '%s' did not match --name argument: %s\n", produced_name.c_str(),
            library_name.c_str());
    }*/

    // We recompile dependencies, and only emit output for the target library.
    let generator = generator::JSONGenerator::new(compilation, experimental_flags);
    write(generator.produce(), output);

    Ok(())
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("compile", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME").unwrap();
            let paths = sub_matches.get_occurrences::<PathBuf>("FILES").unwrap();
            let output = sub_matches.get_one::<PathBuf>("OUT").unwrap();

            let mut stdout = Box::new(std::io::stdout()) as Box<dyn Write>;

            log::debug!("compiling {}", name);

            // Prepare source files.
            let mut source_managers = vec![];

            paths.collect::<Vec<_>>().into_iter().for_each(|path| {
                let lib_sources = path
                    .map(|path| {
                        log::trace!("read file: {}", path.file_name().unwrap().to_str().unwrap());

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

                let source_manager = SourceManager::from(lib_sources);
                source_managers.push(source_manager);
            });

            compile(&source_managers, output, ExperimentalFlags);

            Ok(())
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
