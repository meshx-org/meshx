use std::str::FromStr;

use crate::diagnotics::Diagnostics;
use crate::diagnotics::DiagnosticsError;
use ast::Reference;

use super::ast;
use super::helpers::Pair;
use super::parse_compound_identifier;
use super::Rule;

pub(crate) fn get_collection_subtype(
    pair: &Pair<'_>,
    diagnostics: &mut Diagnostics,
) -> Result<ast::Type, DiagnosticsError> {
    let pair_span = pair.as_span();
    let layout_parameters = pair.clone().into_inner().next();

    // figure out subtype here then return an ast::TypeConstructor at the end
    match layout_parameters {
        Some(layout_parameters) => {
            let mut inner = layout_parameters.into_inner();
            let first_param = inner.next().unwrap();
            parse_type_constructor(first_param, diagnostics)
        }
        None => Err(DiagnosticsError::new_validation_error(
            "This type cosntructor is invalid. It is missing a subtype parameter.",
            pair_span.into(),
        )),
    }
}

pub(crate) fn parse_type_constructor(
    pair: Pair<'_>,
    diagnostics: &mut Diagnostics,
) -> Result<ast::Type, DiagnosticsError> {
    debug_assert!(pair.as_rule() == Rule::type_definition);

    let pair_span = pair.as_span();

    if let Some(current) = pair.clone().into_inner().next() {
        match current.as_rule() {
            Rule::compound_identifier => {
                let identifier = parse_compound_identifier(&current);
                Ok(ast::Type::IdentifierType {
                    reference: Reference::new_sourced(identifier),
                })
            }
            Rule::array_type => Ok(ast::Type::ArrayType {
                element_type: get_collection_subtype(&pair, diagnostics)?.into(),
            }),
            Rule::vector_type => Ok(ast::Type::VectorType {
                element_type: get_collection_subtype(&pair, diagnostics)?.into(),
            }),
            Rule::string_type => Ok(ast::Type::StringType { nullable: false }),
            Rule::primitive_type => Ok(ast::Type::PrimitiveType {
                nullable: false,
                subtype: match ast::PrimitiveSubtype::from_str(pair.as_str()) {
                    Ok(subtype) => subtype,
                    Err(message) => return Err(DiagnosticsError::new_validation_error(message, pair_span.into())),
                },
            }),
            _ => unreachable!(
                "Encountered impossible type during parsing: {:?}",
                current.clone().as_rule()
            ),
        }
    } else {
        unreachable!("Encountered impossible type during parsing: {:?}", pair.as_rule())
    }
}
