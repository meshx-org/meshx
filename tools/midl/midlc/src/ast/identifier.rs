use std::ops::Deref;

use super::{Span, WithSpan};

#[derive(Debug, Clone, Eq)]
pub struct Identifier {
    pub value: String,
    pub span: Span,
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Deref for Identifier {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        &self
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundIdentifier {
    pub components: Vec<Identifier>,
}

impl<T: pest::RuleType> From<pest::iterators::Pair<'_, T>> for CompoundIdentifier {
    fn from(pair: pest::iterators::Pair<'_, T>) -> Self {
        let components = pair
            .into_inner()
            .into_iter()
            .map(|id| id.into())
            .collect::<Vec<Identifier>>();

        CompoundIdentifier { components }
    }
}
