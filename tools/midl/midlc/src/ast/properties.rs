#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nullability {
    Nullable,
    Nonnullable,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Strictness {
    Flexible,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Resourceness {
    Value,
    Resource,
}
