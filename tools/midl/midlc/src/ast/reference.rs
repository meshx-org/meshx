use super::CompoundIdentifier;

// A key identifies a family of elements with a particular name. Unlike
// RawSourced, the roles of each component have been decided, and the library
// has been resolved. Unlike RawSynthetic, the key is stable across
// decomposition, i.e. we can choose it before decomposing the AST, and then
// use it for lookups after decomposing.
struct Key {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reference {
    state: ReferenceState,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReferenceState {
    RawSourced { identifier: CompoundIdentifier },
    RawSynthetic { target: Target },
    Key,
    Contextual,
    Target,
    Failed,
}

impl Reference {
    pub fn new_sourced(identifier: CompoundIdentifier) -> Self {
        Reference {
            state: ReferenceState::RawSourced { identifier },
        }
    }

    pub fn new_synthetic(target: Target) -> Self {
        Reference {
            state: ReferenceState::RawSynthetic { target },
        }
    }

    fn set_key(key: Key) {}
    pub fn mark_contextual() {}
    pub fn resolve_to(target: Target) {}
    pub fn mark_failed() {}
}
