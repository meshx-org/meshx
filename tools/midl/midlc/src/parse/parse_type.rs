use std::str::FromStr;

use super::helpers::Pair;
use super::ast;
use super::Rule;
use super::ParserError;

pub(crate) fn get_collection_subtype(type_pair: &Pair<'_>) -> Result<ast::Type, ParserError> {
    let layout_parameters = type_pair.clone().into_inner().next();

    if let Some(pair) = layout_parameters {
        let first_param = pair.into_inner().next().unwrap();
        parse_type_constructor(&first_param)
    } else {
        Err(ParserError::MissingTypeParameter)
    }
}

pub(crate) fn parse_type_constructor(type_pair: &Pair<'_>) -> Result<ast::Type, ParserError> {
    println!("{:?}", type_pair);

    match type_pair.as_rule() {
        Rule::array_type => Ok(ast::Type::Array {
            element_type: get_collection_subtype(&type_pair)?.into(),
        }),
        Rule::vector_type => Ok(ast::Type::Vector {
            element_type: get_collection_subtype(&type_pair)?.into(),
        }),
        Rule::string_type => Ok(ast::Type::String { nullable: false }),
        Rule::primitive_type => {
            let raw = type_pair.as_str();

            Ok(ast::Type::Primitive {
                nullable: false,
                subtype: ast::PrimitiveSubtype::from_str(raw)?,
            })
        }
        _ => panic!("unhandled type"),
    }
}
