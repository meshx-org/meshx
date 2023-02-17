#![deny(unsafe_code, missing_docs)]
#![allow(clippy::derive_partial_eq_without_eq)]

//! See the docs on [ParserDatabase](./struct.ParserDatabase.html).
//!
//! ## Scope
//!
//! The ParserDatabase is tasked with gathering information about the schema. It is _connector
//! agnostic_: it gathers information and performs generic validations, leaving connector-specific
//! validations to later phases in datamodel core.
//!
//! ## Terminology
//!
//! Names:
//!
//! - _name_: the item name in the schema for datasources, generators, models, model fields,
//!   composite types, composite type fields, enums and enum variants. The `name:` argument for
//!   unique constraints, primary keys and relations.
//! - _mapped name_: the name inside an `@map()` or `@@map()` attribute of a model, field, enum or
//!   enum value. This is used to determine what the name of the Prisma schema item is in the
//!   database.
//! - _database name_: the name in the database, once both the name of the item and the mapped
//!   name have been taken into account. The logic is always the same: if a mapped name is defined,
//!   then the database name is the mapped name, otherwise it is the name of the item.
//! - _constraint name_: indexes, primary keys and defaults can have a constraint name. It can be
//!   defined with a `map:` argument or be a default, generated name if the `map:` argument is not
//!   provided. These usually require a datamodel connector to be defined./// ParserDatabase is a container for a Schema AST, together with information

mod context;
mod libraries;
mod names;
mod references;

use midlgen::ir;

use crate::source_file::{SourceFile, SourceId};
use crate::{ast, diagnotics::Diagnostics};

pub(crate) use context::{Context, ParsingContext};
pub(crate) use libraries::Libraries;

/// gathered during schema validation. Each validation step enriches the
/// database with information that can be used to work with the schema, without
/// changing the AST. Instantiating with `ParserDatabase::new()` will perform a
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

pub(crate) struct ParserDatabase<'lib> {
    // interner: interner::StringInterner,
    // names: Names,
    // types: Types,
    // relations: Relations,
    pub(crate) all_libraries: Libraries<'lib>,
    pub(crate) ir: midlgen::ir::Root,
}

impl<'lib> ParserDatabase<'lib> {
    /// See the docs on [ParserDatabase](/struct.ParserDatabase.html).
    pub(crate) fn new() -> Self {
        let mut all_libraries = Libraries::new();

        let ir = ir::Root {
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
        // println!("ir: {:?}", ir);

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

        ParserDatabase { ir, all_libraries }
    }

    pub fn get_ir(&'lib self) -> &ir::Root {
        &self.ir
    }

    pub fn parse_file(&'lib self, source_id: SourceId, source: &SourceFile<'_>) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();
        let mut ctx = ParsingContext::new(&self.all_libraries, &mut diagnostics, source_id);
        let ast = crate::parse_source(source.as_str(), &mut ctx);
        println!("ast: {:#?}", ast);

        diagnostics
    }

    pub fn compile(&'lib self) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();
        diagnostics
    }
}
