use super::{Span, WithSpan};

#[derive(Debug, Clone)]
pub struct Identifier {
    pub value: String,
    pub span: Span,
}

impl WithSpan for Identifier {
    fn span(&self) -> Span {
        self.span
    }
}

impl<T: pest::RuleType> From<pest::iterators::Pair<'_, T>> for Identifier {
    fn from(pair: pest::iterators::Pair<'_, T>) -> Self {
        Identifier {
            value: pair.as_str().to_owned(),
            span: pair.as_span().into(),
        }
    }
}

#[derive(Debug)]
pub struct CompoundIdentifier (pub Vec<Identifier>);
