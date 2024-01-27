use crate::compiler::TypeResolver;

use super::{Constant, ConstantValue, Nullability, Resource};

trait ConstraintStorage<ValueType> {
    const DEFAULT: ValueType;

    fn resolve_constraint(resolver: &TypeResolver<'_, '_>, param: &Constant, resource: &Resource) -> bool;

    //bool ReportMergeFailure(Reporter* reporter, const Name& layout_name,
    //                      const Constant* param) const override;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NullabilityConstraint(Nullability);

impl ConstraintStorage<Nullability> for NullabilityConstraint {
    const DEFAULT: Nullability = Nullability::Nonnullable;

    fn resolve_constraint(resolver: &TypeResolver<'_, '_>, param: &Constant, resource: &Resource) -> bool {
        todo!()
    }
}

impl std::default::Default for NullabilityConstraint {
    fn default() -> Self {
        Self(NullabilityConstraint::DEFAULT)
    }
}

#[derive(Debug, Clone)]
struct SizeConstraint(Option<ConstantValue>);

impl ConstraintStorage<Option<ConstantValue>> for SizeConstraint {
    const DEFAULT: Option<ConstantValue> = None;

    fn resolve_constraint(resolver: &TypeResolver<'_, '_>, param: &Constant, resource: &Resource) -> bool {
        todo!()
    }
}

impl std::default::Default for SizeConstraint {
    fn default() -> Self {
        Self(SizeConstraint::DEFAULT)
    }
}

#[derive(Debug, Clone, Default)]
pub struct VectorConstraints(NullabilityConstraint, SizeConstraint);

impl VectorConstraints {
    pub fn nullabilty(&self) -> Nullability {
        self.0 .0
    }

    pub fn size(&self) -> &Option<ConstantValue> {
        &self.1 .0
    }
}

#[derive(Debug, Clone, Default)]
pub struct IdentifierConstraints(NullabilityConstraint);

impl IdentifierConstraints {
    pub fn nullabilty(&self) -> Nullability {
        self.0 .0
    }
}
