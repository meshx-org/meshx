use crate::compiler::ParsingContext;
use crate::diagnotics::DiagnosticsError;

use super::ast;
use super::consume_compound_identifier;
use super::helpers::Pair;
use super::Rule;

pub(crate) fn consume_library_declaration(pair: &Pair<'_>, ctx: &mut ParsingContext<'_>) {
    let span = ast::Span::from_pest(pair.as_span(), ctx.source_id);
    let mut name = None;

    for current in pair.clone().into_inner() {
        match current.as_rule() {
            Rule::compound_identifier => {
                name = Some(consume_compound_identifier(&current, ctx));
            }
            _ => {}
        }
    }

    let new_name = name.unwrap().to_vec();
    let library_name = ctx.library.name.clone();

    if ctx.library.name.get().is_none() {
        ctx.library.name.set(new_name).expect("empty library name");
        ctx.library.arbitrary_name_span.replace(Some(span));
    } else {
        if library_name.get() != Some(&new_name.clone()) {
            ctx.diagnostics.push_error(DiagnosticsError::new(
                format!("ErrFilesDisagreeOnLibraryName {:?} vs {:?}", library_name, new_name),
                span,
            ));
            return;
        }
        // Prefer setting arbitrary_name_span to a file which has attributes on the
        // library declaration, if any do, since it's conventional to put all
        // library attributes and the doc comment in a single file (overview.fidl).
        // if self.library.attributes.Empty() && source.library_decl.attributes {
        //    self.library.arbitrary_name_span = source.library_decl.span();
        // }
    }
}
