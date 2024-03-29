use super::helpers::consume_catch_all;
use super::{helpers::Pair, Rule};
use crate::ast;

pub(crate) fn consume_comment_block(token: Pair<'_>) -> Option<ast::Comment> {
    debug_assert!(token.as_rule() == Rule::comment_block);
    
    let mut lines = Vec::new();
    for comment in token.clone().into_inner() {
        match comment.as_rule() {
            Rule::doc_comment => lines.push(consume_doc_comment(comment)),
            Rule::comment | Rule::NEWLINE | Rule::WHITESPACE => {}
            _ => consume_catch_all(&comment, "comment block"),
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(ast::Comment { text: lines.join("\n") })
    }
}

pub(crate) fn consume_doc_comment(token: Pair<'_>) -> &str {
    let child = token.into_inner().next().unwrap();
    match child.as_rule() {
        Rule::doc_content => child.as_str().trim_start(),
        Rule::doc_comment => consume_doc_comment(child),
        x => unreachable!(
            "Encountered impossible doc comment during parsing: {:?}, {:?}",
            x,
            child.tokens()
        ),
    }
}

pub(crate) fn consume_trailing_comment(pair: Pair<'_>) -> Option<ast::Comment> {
    debug_assert_eq!(pair.as_rule(), Rule::trailing_comment);

    let mut lines = Vec::new();
    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::doc_comment => lines.push(consume_doc_comment(current)),
            Rule::comment | Rule::NEWLINE | Rule::WHITESPACE => {}
            _ => consume_catch_all(&current, "trailing comment"),
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(ast::Comment { text: lines.join("\n") })
    }
}
