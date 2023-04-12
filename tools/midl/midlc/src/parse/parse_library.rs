use crate::database::ParsingContext;
use crate::diagnotics::DiagnosticsError;

use super::ast;
use super::helpers::Pair;
use super::parse_compound_identifier;

pub(crate) fn parse_library_declaration(pair: &Pair<'_>, ctx: &mut ParsingContext<'_>) {
    let span = ast::Span::from_pest(pair.as_span(), ctx.source_id);
    let name_pair = pair.clone().into_inner().next().unwrap();
    let new_name = parse_compound_identifier(&name_pair, ctx);

    let mut name = ctx.library.name.borrow_mut();
    let mut arbitrary_name_span = ctx.library.arbitrary_name_span.borrow_mut();

    let cloned_name = name.clone();

    if name.is_none() {
        *name = Some(new_name);
        *arbitrary_name_span = Some(span);
    } else {
        if cloned_name != Some(new_name.clone()) {
            ctx.diagnostics.push_error(DiagnosticsError::new(
                format!(
                    "ErrFilesDisagreeOnLibraryName {:?} vs {:?}",
                    cloned_name.unwrap(),
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
