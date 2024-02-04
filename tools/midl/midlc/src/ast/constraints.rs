use std::{cell::RefCell, rc::Rc};

use petgraph::matrix_graph::Nullable;

use crate::compiler::TypeResolver;

use super::{Constant, ConstantValue, Nullability, Protocol, Resource};

trait ConstraintStorage<ValueType> {
    const DEFAULT: ValueType;

    fn resolve_constraint(resolver: &TypeResolver<'_, '_>, param: &Constant, resource: &Resource) -> bool;

    //bool ReportMergeFailure(Reporter* reporter, const Name& layout_name,
    //                      const Constant* param) const override;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NullabilityConstraint(RefCell<Nullability>);

impl ConstraintStorage<Nullability> for NullabilityConstraint {
    const DEFAULT: Nullability = Nullability::Nonnullable;

    fn resolve_constraint(resolver: &TypeResolver<'_, '_>, param: &Constant, resource: &Resource) -> bool {
        todo!()
    }
}

impl std::default::Default for NullabilityConstraint {
    fn default() -> Self {
        Self(RefCell::new(NullabilityConstraint::DEFAULT))
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProtocolConstraint(Option<Rc<RefCell<Protocol>>>);

impl ConstraintStorage<Option<Rc<RefCell<Protocol>>>> for ProtocolConstraint {
    const DEFAULT: Option<Rc<RefCell<Protocol>>> = None;

    fn resolve_constraint(resolver: &TypeResolver<'_, '_>, param: &Constant, resource: &Resource) -> bool {
        todo!()
    }
}

impl std::default::Default for ProtocolConstraint {
    fn default() -> Self {
        Self(ProtocolConstraint::DEFAULT)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandleSubtypeConstraint(u32);

impl ConstraintStorage<u32> for HandleSubtypeConstraint {
    const DEFAULT: u32 = 0;

    fn resolve_constraint(resolver: &TypeResolver<'_, '_>, param: &Constant, resource: &Resource) -> bool {
        todo!()
    }
}

impl std::default::Default for HandleSubtypeConstraint {
    fn default() -> Self {
        Self(HandleSubtypeConstraint::DEFAULT)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandleRightsConstraint(u32);

impl ConstraintStorage<u32> for HandleRightsConstraint {
    const DEFAULT: u32 = 0;

    fn resolve_constraint(resolver: &TypeResolver<'_, '_>, param: &Constant, resource: &Resource) -> bool {
        todo!()
    }
}

impl std::default::Default for HandleRightsConstraint {
    fn default() -> Self {
        Self(HandleRightsConstraint::DEFAULT)
    }
}

#[derive(Debug, Clone, Default)]
pub struct VectorConstraints(NullabilityConstraint, SizeConstraint);

impl VectorConstraints {
    pub fn nullabilty(&self) -> Nullability {
        self.0 .0.borrow().clone()
    }

    pub fn size(&self) -> &Option<ConstantValue> {
        &self.1 .0
    }

    pub fn new(size: Option<ConstantValue>, nullabitly: Nullability) -> Self {
        Self(NullabilityConstraint(RefCell::new(nullabitly)), SizeConstraint(size))
    }
}

#[derive(Debug, Clone, Default)]
pub struct IdentifierConstraints(NullabilityConstraint);

impl IdentifierConstraints {
    pub fn nullabilty(&self) -> Nullability {
        self.0 .0.borrow().clone()
    }

    pub fn set_nullabilty(&self, val: Nullability) {
        *self.0 .0.borrow_mut() = val;
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransportSideConstraints(NullabilityConstraint, ProtocolConstraint);

impl TransportSideConstraints {
    pub fn nullabilty(&self) -> Nullability {
        self.0 .0.borrow().clone()
    }

    pub fn protocol(&self) -> Option<Rc<RefCell<Protocol>>> {
        self.1 .0.clone()
    }
}

pub type HandleConstraints = (HandleSubtypeConstraint, HandleRightsConstraint, NullabilityConstraint);

impl NullabilityTrait for HandleConstraints {
    fn nullability(&self) -> Nullability {
        self.2 .0.borrow().clone()
    }
}

pub trait NullabilityTrait {
    fn nullability(&self) -> Nullability;
}
