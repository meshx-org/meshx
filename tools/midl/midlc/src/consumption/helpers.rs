use super::parser::Rule;
use crate::{
    ast, compiler::ParsingContext, consumption::consume_value::consume_numeric_literal, diagnotics::{DiagnosticsError, Error},
};

pub type Pair<'a> = pest::iterators::Pair<'a, Rule>;

pub fn consume_ordinal64(pair: Pair<'_>, ctx: &mut ParsingContext<'_>) -> Result<ast::RawOrdinal64, DiagnosticsError> {
    debug_assert!(pair.as_rule() == Rule::ordinal);

    let value_token = pair.into_inner().next().unwrap();
    let value_span = value_token.as_span();

    let span = ast::Span::from_pest(value_span, ctx.source_id);
    let value = consume_numeric_literal(value_token, ctx);

    let value = value.parse::<u64>().expect("unparsable number should not be lexed.");

    if value > std::u64::MAX {
        return Err(Error::OrdinalOutOfBound { span }.into())
    }

    let ordinal = value as u64;
    if ordinal == 0 {
        return Err(Error::OrdinalsMustStartAtOne { span }.into());
    }


    /*if !MaybeConsumeToken(OfKind(Token::Kind::kNumericLiteral)) {
        return Fail(ErrMissingOrdinalBeforeMember);
    }

    let data = scope.GetSourceElement().span().data();
    let errno = 0;
    assert!(errno == 0, "unparsable number should not be lexed.");
    if value > std::u32::MAX {
        return Fail(ErrOrdinalOutOfBound);
    }

    let ordinal = value as u32;
    if ordinal == 0 {
        return Fail(ErrOrdinalsMustStartAtOne);
    }

    return ordinal;*/

    Ok(ast::RawOrdinal64 { value: ordinal, span })
}

#[track_caller]
pub fn consume_catch_all(token: &Pair<'_>, kind: &str) {
    match token.as_rule() {
        Rule::empty_lines | Rule::trailing_comment | Rule::comment_block => {}
        x => unreachable!(
            "Encountered impossible {} during parsing: {:?} {:?}",
            kind,
            &x,
            token.clone().tokens()
        ),
    }
}
