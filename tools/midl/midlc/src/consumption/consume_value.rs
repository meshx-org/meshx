use crate::compiler::ParsingContext;
use crate::diagnotics::DiagnosticsError;

use super::ast;
use super::ast::Span;
use super::helpers::Pair;
use super::Rule;

pub(crate) fn consume_literal(token: Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::Literal {
    assert!(token.as_rule() == Rule::literal);

    let value_token = token.into_inner().next().unwrap();

    match value_token.as_rule() {
        Rule::string_literal => {
            let span = Span::from_pest(value_token.as_span(), ctx.source_id);
            let value = consume_string_literal(value_token, ctx);
            ast::Literal::StringValue(value, span)
        }
        Rule::numeric_literal => {
            let span = Span::from_pest(value_token.as_span(), ctx.source_id);
           
            ast::Literal::NumericValue("".to_string(), span)
        }
        _ => unreachable!(),
    }
}

pub(crate) fn consume_string_literal(token: Pair<'_>, ctx: &mut ParsingContext<'_>) -> String {
    assert!(token.as_rule() == Rule::string_literal);

    let contents = token.clone().into_inner().next().unwrap();
    let contents_str = contents.as_str();

    // This will overallocate a bit for strings with escaped characters, but it
    // shouldn't make a dramatic difference.
    let mut out = String::with_capacity(contents_str.len());
    let mut chars = contents_str.char_indices();

    // https://datatracker.ietf.org/doc/html/rfc8259#section-7
    while let Some((start, c)) = chars.next() {
        match c {
            '\\' => match chars.next().unwrap() {
                (_, '"') => {
                    out.push('"');
                }
                (_, '\\') => {
                    out.push('\\');
                }
                (_, '/') => {
                    out.push('/');
                }
                (_, 'b') => {
                    out.push('\u{0008}');
                }
                (_, 'f') => {
                    out.push('\u{000C}');
                }
                (_, 'n') => {
                    out.push('\n');
                }
                (_, 'r') => {
                    out.push('\r');
                }
                (_, 't') => {
                    out.push('\t');
                }
                (_, 'u') => {
                    let (advance, char) =
                        try_consume_unicode_codepoint(&contents_str[start..], contents.as_span().start() + start, ctx);

                    if let Some(char) = char {
                        out.push(char);
                    }

                    for _ in 0..advance.saturating_sub(2) {
                        chars.next().unwrap();
                    }
                }
                (_, c) => {
                    let span = contents.as_span();
                    let mut final_span = Span::from_pest(span, ctx.source_id);
                    final_span.start += start;
                    final_span.end = final_span.start + 1 + c.len_utf8();
                    ctx.diagnostics.push_error(DiagnosticsError::new_static(
                        r#"Unknown escape sequence. If the value is a windows-style path, `\` must be escaped as `\\`."#,
                        final_span
                    ));
                }
            },
            other => out.push(other),
        }
    }

    out
}

/// https://datatracker.ietf.org/doc/html/rfc8259#section-7
///
/// Returns the parsed character and how much input (in bytes) was consumed.
fn try_consume_unicode_codepoint(
    slice: &str,
    slice_offset: usize,
    ctx: &mut ParsingContext<'_>,
) -> (usize, Option<char>) {
    let unicode_sequence_error = |consumed| {
        let span = ast::Span::new(
            slice_offset,
            (slice_offset + slice.len()).min(slice_offset + consumed),
            ctx.source_id,
        );

        DiagnosticsError::new_static("Invalid unicode escape sequence.", span)
    };

    match consume_codepoint(slice) {
        (consumed, None) => {
            ctx.diagnostics.push_error(unicode_sequence_error(consumed.max(2)));
            (consumed, None)
        }
        (consumed_first_codepoint, Some(first_codepoint)) => {
            // Check if the first codepoint is a valid UTF-8 codepoint. UTF-16 surrogate sequences
            // are not valid UTF-8, so we can do this safely.
            if let Some(c) = char::from_u32(first_codepoint.into()) {
                return (6, Some(c));
            }

            // If that doesn't work, try parsing a second codepoint, and treat the first one as a
            // UTF-16 surrogate pair.
            match consume_codepoint(&slice[6..]) {
                (_, None) => {
                    ctx.diagnostics
                        .push_error(unicode_sequence_error(consumed_first_codepoint));
                    (consumed_first_codepoint, None)
                }
                (consumed_second_codepoint, Some(second_codepoint)) => {
                    // UTF-16 surrogate with
                    let char = match char::decode_utf16([first_codepoint, second_codepoint].into_iter()).next() {
                        Some(Ok(c)) => Some(c),
                        _ => {
                            ctx.diagnostics.push_error(unicode_sequence_error(
                                consumed_first_codepoint + consumed_second_codepoint,
                            ));
                            None
                        }
                    };

                    (consumed_first_codepoint * 2, char)
                }
            }
        }
    }
}

fn consume_codepoint(slice: &str) -> (usize, Option<u16>) {
    if slice.len() < 4 || !slice.starts_with("\\u") {
        return (0, None);
    }

    let mut chars = slice[2..].chars();
    let mut codepoint = 0u16;

    // four nibbles (4 bit integers)
    for i in 0u8..4 {
        let nibble_offset = 3 - i;
        match chars.next().and_then(|c| c.to_digit(16)) {
            Some(nibble) => {
                codepoint += (nibble as u16) << (nibble_offset * 4);
            }
            None => return (2 + i as usize, None),
        }
    }

    (6, Some(codepoint))
}
