#![deny(unsafe_code, missing_docs)]
#![allow(clippy::derive_partial_eq_without_eq)]

mod context;
mod libraries;
// mod names;
mod typespace;
mod type_resolver;
mod compile_step;
mod resolve_step;

use pest::Parser;
use std::cell::RefCell;
use std::rc::Rc;

use crate::consumption::{MIDLParser, Rule};
use crate::source_file::{SourceFile, SourceId};
use crate::{ast, diagnotics::Diagnostics};
pub use type_resolver::TypeResolver;

pub(crate) use context::{Context, ParsingContext};
pub(crate) use libraries::Libraries;

use self::compile_step::CompileStep;
use self::resolve_step::ResolveStep;

/// Like ast::Declarations, but with const pointers rather than unique_ptr.
#[derive(Debug, Default)]
pub struct Declarations {
    // aliases: Vec<Rc<RefCell<Alias>>>,
    // bits: Vec<Rc<RefCell<Bits>>>,
    pub builtins: Vec<ast::Declaration>,
    pub consts: Vec<ast::Declaration>,
    pub enums: Vec<ast::Declaration>,
    // new_types: Vec<Rc<RefCell<NewType>>>,
    pub protocols: Vec<ast::Declaration>,
    pub resources: Vec<ast::Declaration>,
    // services: Vec<Rc<RefCell<ast::Service>>>,
    pub structs: Vec<ast::Declaration>,
    // tables: Vec<Rc<RefCell<ast::Table>>>,
    pub unions: Vec<ast::Declaration>,
    // overlays: Vec<Rc<RefCell<Overlay>>>,
}

/// A library dependency together with its filtered declarations.
#[derive(Debug)]
struct Dependency {
    library: Rc<ast::Library>,
    declarations: Declarations,
}

/// A compilation is the result of compiling a library and all its transitive
/// dependencies. All fidlc output should be a function of the compilation
/// (roughly speaking; of course everything is reachable via pointers into the
/// AST, but we should avoid any further processing/traversals).
#[derive(Debug)]
pub struct Compilation {
    /// Filtered from library->declarations.
    pub declarations: Declarations,

    /// The target library name and attributes. Note, we purposely do not store a
    /// Library* to avoid accidentally reaching into its unfiltered decls.
    pub library_name: Vec<String>,

    /// Filtered from structs used as method payloads in protocols that come from
    /// an external library via composition.
    pub external_structs: Vec<Rc<RefCell<ast::Struct>>>,

    /// Filtered from library->declaration_order.
    pub declaration_order: Vec<ast::Declaration>,

    /// Filtered from library->dependencies, and also includes indirect
    /// dependencies that come from protocol composition, i.e. what would need to
    /// be imported if the composed methods were copied and pasted.
    pub direct_and_composed_dependencies: Vec<Dependency>,

    /// Versions that were selected for this compilation.
    pub version_selection: u32, // VersionSelection;
}

/// gathered during schema validation. Each validation step enriches the
/// database with information that can be used to work with the schema, without
/// changing the AST. Instantiating with `Compiler::new()` will perform a
/// number of validations and make sure the schema makes sense, but it cannot
/// fail. In case the schema is invalid, diagnostics will be created and the
/// resolved information will be incomplete.
///
/// Validations are carried out in the following order:
///
/// - The AST is walked a first time to resolve names: to each relevant
///   identifier, we attach an ID that can be used to reference the
///   corresponding item (model, enum, field, ...)
/// - The AST is walked a second time to resolve types. For each field and each
///   type alias, we look at the type identifier and resolve what it refers to.
/// - The AST is walked a third time to validate attributes on models and
///   fields.
/// - Global validations are then performed on the mostly validated schema.
///   Currently only index name collisions.
pub(crate) struct Compiler {
    // interner: interner::StringInterner,
    // names: Names,
    // types: Types,
    // relations: Relations,
    pub(crate) all_libraries: Rc<RefCell<Libraries>>,
    pub(crate) library: Rc<ast::Library>,
}

impl Compiler {
    /// See the docs on [Compiler](/struct.Compiler.html).
    pub(crate) fn new(all_libraries: Rc<RefCell<Libraries>>) -> Option<Self> {

        /*
        // Second pass: resolve top-level items and field types.
        types::resolve_types(&mut ctx);

        // Return early on type resolution errors.
        if ctx.diagnostics.has_errors() {
            return ParserDatabase {
                ast,
                file,
                interner,
                names,
                types,
                relations,
            };
        }

        // Third pass: validate model and field attributes. All these
        // validations should be _order independent_ and only rely on
        // information from previous steps, not from other attributes.
        attributes::resolve_attributes(&mut ctx);

        // Fourth step: relation inference
        relations::infer_relations(&mut ctx);

        */

        let library = ast::Library::default();

        // let builtin = ast::Builtin::default();
        // library.elements.push(ast::Element::Builtin(&builtin));

        let library = Rc::from(library);

        Some(Compiler { all_libraries, library })
    }

    /// Consumes a source file. Must be called once for each file in the library.
    pub fn consume_file(&self, source_id: SourceId, source: &SourceFile<'_>) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();

        let mut ctx = ParsingContext::new(
            self.library.clone(),
            self.all_libraries.clone(),
            &mut diagnostics,
            source_id,
        );

        let pairs = MIDLParser::parse(Rule::library, source.as_str()).unwrap();

        crate::consumption::consume_source(pairs, &mut ctx);

        diagnostics
    }

    /// Compiles the library. Must be called once after consuming all files. On
    /// success, inserts the new library into all_libraries and returns true.
    pub fn compile<'d>(&self, diagnostics: &'d mut Diagnostics) -> bool {
        let mut ctx = Context::new(self.library.clone(), self.all_libraries.clone(), diagnostics);

        // names::verify_names(&mut ctx);

        //if !AvailabilityStep::new(&mut ctx).run() {
        //    return false;
        //}

        if !ResolveStep::new(&mut ctx).run() {
            return false;
        }

        if !CompileStep::new(&mut ctx).run() {
            return false;
        }

        if !self.all_libraries.borrow_mut().insert(self.library.clone()) {
            return false;
        }

        true
    }
}
