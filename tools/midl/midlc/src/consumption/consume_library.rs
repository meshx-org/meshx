use crate::compiler::ParsingContext;
use crate::diagnotics::DiagnosticsError;

use super::ast;
use super::consume_compound_identifier;
use super::helpers::Pair;

pub(crate) fn consume_library_declaration(pair: &Pair<'_>, ctx: &mut ParsingContext<'_>) {
    let span = ast::Span::from_pest(pair.as_span(), ctx.source_id);
    let name_pair = pair.clone().into_inner().next().unwrap();
    let new_name = consume_compound_identifier(&name_pair, ctx).to_vec();

    let library_name = ctx.library.name.clone();

    if ctx.library.name.get().is_none() {
        ctx.library.name.set(new_name).expect("empty library name");
        ctx.library.arbitrary_name_span.replace(Some(span));
    } else {
        if library_name.get() != Some(&new_name.clone()) {
            ctx.diagnostics.push_error(DiagnosticsError::new(
                format!(
                    "ErrFilesDisagreeOnLibraryName {:?} vs {:?}",
                    library_name,
                    new_name
                ),
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
