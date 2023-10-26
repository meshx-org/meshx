use crate::ast::CompoundIdentifier;
use crate::compiler::ParsingContext;
use crate::diagnotics::DiagnosticsError;

use super::{ast, consume_compound_identifier, consume_identifier};
use super::{helpers::Pair, Rule};

pub(crate) fn consume_import(using_directive: &Pair<'_>, ctx: &mut ParsingContext<'_>) {
    let pair_span = using_directive.as_span();
    let mut using_path: Option<ast::CompoundIdentifier> = None;
    let mut import_alias: Option<ast::CompoundIdentifier> = None;

    for current in using_directive.clone().into_inner() {
        match current.as_rule() {
            Rule::compound_identifier => {
                using_path = Some(consume_compound_identifier(&current, ctx));
            }
            Rule::import_alias => {
                let identifier = current.into_inner().next().unwrap();
                let id = consume_identifier(&identifier, ctx);
                import_alias = Some(CompoundIdentifier {
                    components: vec![id],
                    span: ast::Span::from_pest(identifier.as_span(), ctx.source_id),
                });
            }
            _ => panic!("Unexpected rule: {:?}", current.as_rule()),
        }
    }

    let using_path = using_path.expect("using_path is None");

    let dep_library = {
        let lock = ctx.all_libraries.borrow();
        lock.lookup(using_path.to_vec())
    };

    if dep_library.is_none() {
        let span = using_path.span.clone();
        ctx.diagnostics.push_error(DiagnosticsError::new_unknown_library(
            format!("{}", using_path).as_str(),
            span,
        ));
        return;
    }

    let result = ctx.library.dependencies.borrow_mut().register(
        &ast::Span::from_pest(pair_span, ctx.source_id),
        dep_library.unwrap(),
        import_alias.clone().map(|v| v.to_vec()),
    );

    match result {
        ast::RegisterResult::Success => {}

        ast::RegisterResult::Duplicate => {
            ctx.diagnostics.push_error(DiagnosticsError::new_duplicate_import(
                format!("ErrDuplicateLibraryImport: {:?}", using_path).as_str(),
                ast::Span::from_pest(pair_span, ctx.source_id),
            ));
            return;
        }

        ast::RegisterResult::Collision => {
            if import_alias.is_some() {
                ctx.diagnostics
                    .push_error(DiagnosticsError::new_conflicting_aliased_import(
                        format!("ErrConflictingLibraryImportAlias: {}", using_path).as_str(),
                        ast::Span::from_pest(pair_span, ctx.source_id),
                    ));

                return;
            }

            ctx.diagnostics.push_error(DiagnosticsError::new_conflicting_import(
                format!("ErrConflictingLibraryImport: {}", using_path).as_str(),
                ast::Span::from_pest(pair_span, ctx.source_id),
            ));

            return;
        }
    }
}
