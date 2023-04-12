use std::{cmp::Ordering, hash::Hasher, ops::Deref};

use super::{Span, WithSpan};

#[derive(Debug, Clone, Eq, PartialOrd)]
pub struct Identifier {
    pub value: String,
    pub span: Span,
}

impl std::fmt::Display for Identifier {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.value)
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl std::hash::Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
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

#[derive(Clone, Eq)]
pub struct CompoundIdentifier {
    pub components: Vec<Identifier>,
    pub span: Span,
}

impl PartialEq for CompoundIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.components == other.components
    }
}

impl PartialOrd for CompoundIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompoundIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.components.cmp(&other.components)
    }
}

impl std::hash::Hash for CompoundIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.components.hash(state);
    }
}

impl std::fmt::Debug for CompoundIdentifier {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<String> = self.components.iter().map(|c| c.value.to_string()).collect();
        write!(f, "CompoundIdentifier({:?})", ids.join("."))
    }
}

impl std::fmt::Display for CompoundIdentifier {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<String> = self.components.iter().map(|c| c.value.to_string()).collect();
        write!(f, "{}", ids.join("."))
    }
}
