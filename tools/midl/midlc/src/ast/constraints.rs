use crate::ast;
use crate::{compiler::TypeResolver, diagnotics::Diagnostics};
use std::any::Any;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstraintKind {
    Protocol,
    Nullability,
    HandleSubtype,
    HandleRights,
}

trait ConstraintStorage<ValueType>: Any
where
    ValueType: Clone,
{
    fn resolve_constraint(
        &mut self,
        resolver: &TypeResolver<'_, '_>,
        param: &ast::Constant,
        resource: Option<&ast::Resource>,
    ) -> bool;

    fn has_constraint(&self) -> bool;

    fn set_value(&mut self, val: ValueType);
    fn value(&self) -> &ValueType;

    fn report_merge_failure(&self, reporter: Rc<Diagnostics>, layout_name: &ast::Name) -> bool {
        false
    }

    //bool ReportMergeFailure(Reporter* reporter, const Name& layout_name,
    //                      const Constant* param) const override;
}

fn merge_constraint<ValueType>(
    reporter: Rc<Diagnostics>,
    layout_name: &ast::Name,
    base: &dyn ConstraintStorage<ValueType>,
    resolved: &dyn ConstraintStorage<ValueType>,
    out_merged: &mut dyn ConstraintStorage<ValueType>,
) -> bool
where
    ValueType: Clone + 'static,
{
    if base.has_constraint() && resolved.has_constraint() {
        return resolved.report_merge_failure(reporter, layout_name /*, resolved.raw_constraint*/);
    }

    if resolved.has_constraint() {
        out_merged.set_value(resolved.value().clone());
        //out_merged.raw_constraint = resolved.raw_constraint;
    } else {
        out_merged.set_value(base.value().clone());
        //out_merged.raw_constraint = base.raw_constraint;
    }

    true
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NullabilityConstraint(Option<RefCell<ast::Nullability>>);

impl ConstraintStorage<Option<RefCell<ast::Nullability>>> for NullabilityConstraint {
    fn resolve_constraint(
        &mut self,
        resolver: &TypeResolver<'_, '_>,
        param: &ast::Constant,
        resource: Option<&ast::Resource>,
    ) -> bool {
        if resolver.resolve_as_optional(param) {
            self.0 = Some(RefCell::new(ast::Nullability::Nullable));
            return true;
        }

        return false;
    }

    fn has_constraint(&self) -> bool {
        // self.0 != Self::default().0
        false
    }

    fn value(&self) -> &Option<RefCell<ast::Nullability>> {
        &self.0
    }

    fn set_value(&mut self, val: Option<RefCell<ast::Nullability>>) {
        self.0 = val;
    }
}

impl std::default::Default for NullabilityConstraint {
    fn default() -> Self {
        Self(Some(RefCell::new(ast::Nullability::Nonnullable)))
    }
}

#[derive(Debug, Clone)]
struct SizeConstraint(Option<ast::ConstantValue>);

impl ConstraintStorage<Option<ast::ConstantValue>> for SizeConstraint {
    fn resolve_constraint(
        &mut self,
        resolver: &TypeResolver<'_, '_>,
        param: &ast::Constant,
        resource: Option<&ast::Resource>,
    ) -> bool {
        todo!()
    }

    fn has_constraint(&self) -> bool {
        self.0.is_some()
    }

    fn value(&self) -> &Option<ast::ConstantValue> {
        todo!()
    }

    fn set_value(&mut self, val: Option<ast::ConstantValue>) {
        todo!()
    }
}

impl std::default::Default for SizeConstraint {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolConstraint(Option<Rc<RefCell<ast::Protocol>>>);

impl ConstraintStorage<Option<Rc<RefCell<ast::Protocol>>>> for ProtocolConstraint {
    fn resolve_constraint(
        &mut self,
        resolver: &TypeResolver<'_, '_>,
        param: &ast::Constant,
        resource: Option<&ast::Resource>,
    ) -> bool {
        match resolver.resolve_as_protocol(param) {
            Some(result) => {
                self.0 = Some(result);
                true
            }
            None => false,
        }
    }

    fn has_constraint(&self) -> bool {
        self.0.is_some()
    }

    fn value(&self) -> &Option<Rc<RefCell<ast::Protocol>>> {
        &self.0
    }

    fn set_value(&mut self, val: Option<Rc<RefCell<ast::Protocol>>>) {
        self.0 = val;
    }
}

impl std::default::Default for ProtocolConstraint {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandleSubtypeConstraint(Option<u32>);

impl ConstraintStorage<u32> for HandleSubtypeConstraint {
    fn resolve_constraint(
        &mut self,
        resolver: &TypeResolver<'_, '_>,
        param: &ast::Constant,
        resource: Option<&ast::Resource>,
    ) -> bool {
        todo!()
    }

    fn has_constraint(&self) -> bool {
        self.0.is_some()
    }

    fn value(&self) -> &u32 {
        todo!()
    }

    fn set_value(&mut self, val: u32) {
        todo!()
    }
}

impl std::default::Default for HandleSubtypeConstraint {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandleRightsConstraint(Option<u32>);

impl ConstraintStorage<u32> for HandleRightsConstraint {
    fn resolve_constraint(
        &mut self,
        resolver: &TypeResolver<'_, '_>,
        param: &ast::Constant,
        resource: Option<&ast::Resource>,
    ) -> bool {
        todo!()
    }

    fn has_constraint(&self) -> bool {
        self.0.is_some()
    }

    fn value(&self) -> &u32 {
        todo!()
    }

    fn set_value(&mut self, val: u32) {
        todo!()
    }
}

impl std::default::Default for HandleRightsConstraint {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Debug, Clone, Default)]
pub struct VectorConstraints(NullabilityConstraint, SizeConstraint);

impl VectorConstraints {
    pub fn size(&self) -> &Option<ast::ConstantValue> {
        &self.1 .0
    }

    pub fn new(size: Option<ast::ConstantValue>, nullabitly: ast::Nullability) -> Self {
        Self(
            NullabilityConstraint(Some(RefCell::new(nullabitly))),
            SizeConstraint(size),
        )
    }
}

impl NullabilityTrait for VectorConstraints {
    fn nullability(&self) -> ast::Nullability {
        self.0 .0.as_ref().unwrap().borrow().clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct IdentifierConstraints(NullabilityConstraint);

impl IdentifierConstraints {
    pub fn nullabilty(&self) -> ast::Nullability {
        self.0 .0.as_ref().unwrap().borrow().clone()
    }

    pub fn set_nullabilty(&self, val: ast::Nullability) {
        *self.0 .0.as_ref().unwrap().borrow_mut() = val;
    }
}

pub type TransportSideConstraints = (NullabilityConstraint, ProtocolConstraint);

impl NullabilityTrait for TransportSideConstraints {
    fn nullability(&self) -> ast::Nullability {
        self.0 .0.as_ref().unwrap().borrow().clone()
    }
}

impl ProtocolTrait for TransportSideConstraints {
    fn protocol(&self) -> Option<Rc<RefCell<ast::Protocol>>> {
        self.1 .0.clone()
    }
}

impl MergeConstraints for TransportSideConstraints {
    /// Merge resolved constraints onto base constraints.
    /// This is a recursive template that's applied to each constraint in order.
    fn merge_constraints(
        reporter: Rc<Diagnostics>,
        layout_name: &ast::Name,
        base: &Self,
        resolved: &Self,
        out_merged: &mut Self,
    ) -> bool {
        if !merge_constraint(
            reporter.clone(),
            layout_name,
            &base.0,
            resolved.0.borrow(),
            &mut out_merged.0,
        ) {
            return false;
        }

        if !merge_constraint(reporter, layout_name, &base.1, resolved.1.borrow(), &mut out_merged.1) {
            return false;
        }

        true
    }

    fn has_constraint(&self, kind: ConstraintKind) -> bool {
        match kind {
            ConstraintKind::Nullability => self.0.has_constraint(),
            ConstraintKind::Protocol => self.1.has_constraint(),
            _ => false,
        }
    }

    fn resolve_one_constraint(
        &mut self,
        constraint_index: usize,
        resolver: &TypeResolver<'_, '_>,
        param: &ast::Constant,
        resource: Option<&ast::Resource>,
    ) -> bool {
        match constraint_index {
            0 => return self.0.resolve_constraint(resolver, param, resource),
            1 => return self.1.resolve_constraint(resolver, param, resource),
            _ => false,
        }
    }

    fn constraints_count(&self) -> usize {
        2
    }
}

pub type HandleConstraints = (HandleSubtypeConstraint, HandleRightsConstraint, NullabilityConstraint);

impl NullabilityTrait for HandleConstraints {
    fn nullability(&self) -> ast::Nullability {
        self.2 .0.as_ref().unwrap().borrow().clone()
    }
}

pub trait NullabilityTrait {
    fn nullability(&self) -> ast::Nullability;
}

pub trait ProtocolTrait {
    fn protocol(&self) -> Option<Rc<RefCell<ast::Protocol>>>;
}

pub trait ResolveAndMerge: MergeConstraints
where
    Self: Debug + Default,
{
    fn resolve_and_merge_constraints(
        &mut self,
        resolver: &TypeResolver<'_, '_>,
        reporter: Rc<Diagnostics>,
        params_span: Option<ast::Span>,
        layout_name: &ast::Name,
        resource: Option<&ast::Resource>,
        params: &Vec<ast::Constant>,
        out_merged: &mut Self,
        // layout_invocation: Option<ast::LayoutInvocation>,
    ) -> bool {
        let mut resolved = Self::default();
        // static_assert(std::is_base_of_v<std::remove_reference_t<decltype(*this)>, M>);
        if !resolved.resolve_constraints(resolver, params, params_span, resource) {
            return false;
        }

        //if layout_invocation.is_some() {
        // resolved.populate_layout_invocation(layout_invocation);
        //}

        return Self::merge_constraints(reporter, layout_name, self, &resolved, out_merged);
    }

    fn resolve_constraints(
        &mut self,
        resolver: &TypeResolver<'_, '_>,
        params: &Vec<ast::Constant>,
        params_span: Option<ast::Span>,
        resource: Option<&ast::Resource>,
    ) -> bool {
        let num_params = params.len();
        let num_constraints = self.constraints_count();
        let mut constraint_index = 0;

        // For each param supplied...
        for param_index in 0..num_params {
            println!("1.");
            // Walk through the next constraint to see if one can resolve.
            while constraint_index < num_constraints
                && !self.resolve_one_constraint(constraint_index, resolver, params.get(param_index).unwrap(), resource)
            {
                
                constraint_index += 1;
            }

            if constraint_index == num_constraints {
                panic!("OnUnexpectedConstraint {:?}", params_span);
                //  // Ran out of constraint kinds trying to match this item.
                //  return OnUnexpectedConstraint(resolver, reporter, params_span, layout_name, resource,
                //                                num_constraints, params, param_index);
            }

            constraint_index += 1;
        }

        true
    }
}

impl<T> ResolveAndMerge for T where T: MergeConstraints + Debug + Default {}

pub trait MergeConstraints {
    fn merge_constraints(
        reporter: Rc<Diagnostics>,
        layout_name: &ast::Name,
        base: &Self,
        resolved: &Self,
        out_merged: &mut Self,
    ) -> bool;

    fn resolve_one_constraint(
        &mut self,
        constraint_index: usize,
        resolver: &TypeResolver<'_, '_>,
        param: &ast::Constant,
        resource: Option<&ast::Resource>,
    ) -> bool;

    fn constraints_count(&self) -> usize;
    fn has_constraint(&self, kind: ConstraintKind) -> bool;
}
