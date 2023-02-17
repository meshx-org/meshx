use crate::ast::WithSpan;
use crate::diagnotics::{Diagnostics, DiagnosticsError};

use super::helpers::Pair;
use super::{ast, parse_compound_identifier};

pub(crate) fn parse_import(using_directive: &Pair<'_>, diagnostics: &mut Diagnostics) {
}

/* pub(crate) fn parse_import(using_directive: &Pair<'_>, diagnostics: &mut Diagnostics) {
    let span = using_directive.as_span();
    let using_path = using_directive.clone().into_inner().next().unwrap();
    let using_path = parse_compound_identifier(&using_path);
    let alias = using_directive
        .clone()
        .into_inner()
        .next()
        .map(|pair| pair.as_str().to_string());

    let library_name: Vec<ast::Identifier>;
    for component in using_path.components.into_iter() {
        library_name.push(component);
    }

    let dep_library = all_libraries.lookup(library_name);
    if dep_library.is_none() {
        diagnostics.push_error(DiagnosticsError::new_static(
            format!("ErrUnknownLibrary: {:?}", library_name).as_str(),
            using_path.components[0].span(),
        ));
        return;
    }

    let result = library.dependencies.register(span, dep_library, alias);

    match (result) {
        ast::RegisterResult::Success => {}

        ast::RegisterResult::Duplicate => {
            diagnostics.push_error(DiagnosticsError::new_static(
                format!("ErrDuplicateLibraryImport: {:?}", library_name).as_str(),
                ast::Span::from(span),
            ));
            return;
        }

        ast::RegisterResult::Collision => {
            if alias.is_some() {
                diagnostics.push_error(DiagnosticsError::new_static(
                    format!("ErrConflictingLibraryImportAlias: {:?}", library_name).as_str(),
                    ast::Span::from(span),
                ));

                return;
            }

            diagnostics.push_error(DiagnosticsError::new_static(
                format!("ErrConflictingLibraryImport: {:?}", library_name).as_str(),
                ast::Span::from(span),
            ));

            return;
        }
    }
}*/
