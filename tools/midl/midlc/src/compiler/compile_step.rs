use anyhow::Result;

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{
    ast::{self, ConstantTrait, ConstantValue, Decl, PrimitiveSubtype, WithSpan},
    diagnotics::{DiagnosticsError, Error},
};

use super::{type_resolver::TypeResolver, typespace::Typespace, Context};

pub(crate) struct CompileStep<'ctx, 'd> {
    pub(super) ctx: &'ctx mut Context<'d>,

    /// Stack of decls being compiled. Used to trace back and print the cycle if a
    /// cycle is detected.
    decl_stack: RefCell<Vec<ast::Declaration>>,
}

struct ScopeInsertResult {
    previous_occurrence: Option<ast::Span>,
}

impl ScopeInsertResult {
    fn ok() -> Self {
        Self {
            previous_occurrence: None,
        }
    }

    fn new_failure_at(previous: ast::Span) -> Self {
        Self {
            previous_occurrence: Some(previous),
        }
    }

    fn is_ok(&self) -> bool {
        self.previous_occurrence.is_none()
    }

    fn previous_occurrence(&self) -> ast::Span {
        self.previous_occurrence.as_ref().unwrap().clone()
    }
}

type MemberValidator<T> = dyn FnMut(T, Vec<ast::Attribute>, ast::Span) -> Result<(), DiagnosticsError>;

#[derive(Debug)]
struct Scope<T> {
    scope: BTreeMap<T, ast::Span>,
}

impl<T> Scope<T> {
    fn new() -> Self {
        Self { scope: BTreeMap::new() }
    }

    fn insert(&mut self, t: T, span: ast::Span) -> ScopeInsertResult
    where
        T: Ord,
    {
        let last = self.scope.get_key_value(&t);

        if let Some((last, span)) = last {
            return ScopeInsertResult::new_failure_at(span.clone());
        }

        self.scope.insert(t, span);
        ScopeInsertResult::ok()
    }
}

type Ordinal64Scope = Scope<u64>;

fn find_first_non_dense_ordinal(scope: &Ordinal64Scope) -> Option<(u64, ast::Span)> {
    let mut last_ordinal_seen = 0;

    for (ordinal, loc) in scope.scope.iter() {
        let next_expected_ordinal = last_ordinal_seen + 1;

        if *ordinal != next_expected_ordinal {
            return Some((next_expected_ordinal, loc.clone()));
        }

        last_ordinal_seen = *ordinal;
    }

    None
}

impl<'ctx, 'd> CompileStep<'ctx, 'd>
where
    'ctx: 'd,
{
    pub fn new(ctx: &'ctx mut Context<'d>) -> Self {
        Self {
            ctx,
            decl_stack: RefCell::new(Vec::new()),
        }
    }

    fn typespace(&self) -> Rc<Typespace> {
        self.ctx.all_libraries.borrow().typespace()
    }

    pub(crate) fn run(&self) -> bool {
        let checkpoint = self.ctx.diagnostics.checkpoint();
        self.run_impl();
        checkpoint.no_new_errors()
    }

    fn run_impl(&self) {
        // CompileAttributeList(library()->attributes.get());
        for (_, decl) in self.ctx.library.declarations.borrow_mut().all.flat_iter_mut() {
            self.compile_decl(decl);
        }
    }

    pub(crate) fn compile_decl(&self, decl: &mut ast::Declaration) {
        {
            let mut decl_stack = self.decl_stack.borrow_mut();

            decl.set_compiling(true);
            decl_stack.push(decl.clone());
        }

        match decl {
            ast::Declaration::Const { decl } => self.compile_const(decl.clone()),
            ast::Declaration::Enum { decl } => self.compile_enum(decl.clone()),
            ast::Declaration::Bits { decl } => self.compile_bits(decl.clone()),
            ast::Declaration::Struct { decl } => self.compile_struct(decl.clone()),
            ast::Declaration::Union { decl } => self.compile_union(decl.clone()),
            ast::Declaration::Protocol { decl } => self.compile_protocol(decl.clone()),
            ast::Declaration::Alias { decl } => self.compile_alias(decl.clone()),
            ast::Declaration::Resource { decl } => self.compile_resource(decl.clone()),
            _ => todo!(),
        }

        {
            let mut decl_stack = self.decl_stack.borrow_mut();

            decl.set_compiling(false);
            decl.set_compiled(true);
            decl_stack.pop();
        }
    }

    pub(crate) fn get_decl_cycle(&self, decl: &mut ast::Declaration) -> Option<Vec<ast::Declaration>> {
        if !decl.compiled() && decl.compiling() {
            // Decl should already be in the stack somewhere because compiling is set to
            // true iff the decl is in the decl stack.
            let decl_pos = self.decl_stack.borrow().iter().position(|r| r == decl).unwrap();

            // Copy the part of the cycle we care about so Compiling guards can pop
            // normally when returning.
            let mut cycle = self.decl_stack.borrow()[..decl_pos].to_vec();

            // Add a second instance of the decl at the end of the list so it shows as
            // both the beginning and end of the cycle.
            cycle.push(decl.clone());

            return Some(cycle);
        }

        None
    }

    fn compile_attribute_list(&self, attrs: &ast::AttributeList) {}

    fn validate_members<MemberType>(&self, decl: ast::Declaration, validator: &mut MemberValidator<MemberType>) -> bool
    where
        MemberType: From<ConstantValue> + Copy + Ord,
    {
        //assert!(decl != nullptr);
        // let checkpoint = reporter()->Checkpoint();

        let mut value_scope: Scope<MemberType> = Scope::new();

        if let ast::Declaration::Enum { decl } = decl {
            for (_, member) in decl.borrow().iter_members() {
                let mut member = member.borrow_mut();
                // assert(member.value != nullptr, "member value is null");

                if !self.resolve_constant(&mut member.value, decl.borrow().subtype_ctor.r#type.clone()) {
                    panic!("ErrCouldNotResolveMember");
                    // reporter().Fail(ErrCouldNotResolveMember, member.name, decl.kind);
                    continue;
                }

                let value: MemberType = member.value.value().into();
                let value_result = value_scope.insert(value, member.name.clone());
                if !value_result.is_ok() {
                    let previous_span = value_result.previous_occurrence();
                    // We can log the error and then continue validating other members for other bugs
                    // reporter().Fail(ErrDuplicateMemberValue, member.name, decl.kind, member.name.data(), previous_span.data(), previous_span);
                }

                let result = validator(value, member.attributes.clone(), member.name.clone());

                if let Err(err) = result {
                    //reporter().Report(err);
                }
            }
        }

        // return checkpoint.NoNewErrors();
        true
    }

    fn validate_enum_members_and_calc_unknown_value<MemberType>(
        &self,
        decl: Rc<RefCell<ast::Enum>>,
        out_unknown_value: &mut MemberType,
    ) -> bool
    where
        MemberType: num::Bounded + From<ConstantValue> + Copy + Ord,
    {
        let enum_decl = decl.borrow();
        let default_unknown_value = <MemberType as num::Bounded>::max_value();
        let explicit_unknown_value: Option<MemberType> = None;

        for member in enum_decl.members.iter() {
            let mut member = member.borrow_mut();

            if !self.resolve_constant(&mut member.value, enum_decl.subtype_ctor.r#type.clone()) {
                // ValidateMembers will resolve each member and report errors.
                continue;
            }
            //if (member.attributes.Get("unknown") != nullptr) {
            //    if (explicit_unknown_value.has_value()) {
            //        return reporter().Fail(ErrUnknownAttributeOnMultipleEnumMembers, member.name);
            //    }
            //
            //    explicit_unknown_value = static_cast<const NumericConstantValue<MemberType>&>(member.value.value()).value;
            //}
        }

        let result = self.validate_members::<MemberType>(
            ast::Declaration::Enum { decl: decl.clone() },
            &mut |value, attrs, id| Ok(()),
        );

        if !result {
            return false;
        }

        *out_unknown_value = explicit_unknown_value.unwrap_or(default_unknown_value);
        return true;
    }

    fn compile_enum(&self, enum_declaration: Rc<RefCell<ast::Enum>>) {
        let subtype = {
            let mut enum_declaration = enum_declaration.borrow_mut();

            self.compile_attribute_list(&enum_declaration.attributes);
            //for member in enum_declaration.members {
            //    self.compile_attribute_list(member.attributes);
            //}

            self.compile_type_constructor(&mut enum_declaration.subtype_ctor);

            if enum_declaration.subtype_ctor.r#type.is_none() {
                return;
            }

            match enum_declaration.subtype_ctor.r#type.as_ref().unwrap().clone() {
                ast::Type::Primitive(primitive_type) => {
                    enum_declaration.r#type = Some(primitive_type.clone());
                    primitive_type.subtype
                }
                _ => {
                    //    reporter()->Fail(ErrEnumTypeMustBeIntegralPrimitive, enum_declaration->name.span().value(),
                    //                    enum_declaration->subtype_ctor->type);
                    return;
                }
            }
        };

        // Validate constants.
        match subtype {
            ast::PrimitiveSubtype::Int8 => {
                let mut unknown_value: i8 = 0;
                if self.validate_enum_members_and_calc_unknown_value::<i8>(enum_declaration.clone(), &mut unknown_value)
                {
                    enum_declaration.borrow_mut().unknown_value_signed = unknown_value as i64;
                }
            }
            PrimitiveSubtype::Int16 => {
                let mut unknown_value: i16 = 0;
                if self
                    .validate_enum_members_and_calc_unknown_value::<i16>(enum_declaration.clone(), &mut unknown_value)
                {
                    enum_declaration.borrow_mut().unknown_value_signed = unknown_value as i64;
                }
            }
            PrimitiveSubtype::Int32 => {
                let mut unknown_value: i32 = 0;
                if self
                    .validate_enum_members_and_calc_unknown_value::<i32>(enum_declaration.clone(), &mut unknown_value)
                {
                    enum_declaration.borrow_mut().unknown_value_signed = unknown_value as i64;
                }
            }
            PrimitiveSubtype::Int64 => {
                let mut unknown_value: i64 = 0;
                if self
                    .validate_enum_members_and_calc_unknown_value::<i64>(enum_declaration.clone(), &mut unknown_value)
                {
                    enum_declaration.borrow_mut().unknown_value_signed = unknown_value as i64;
                }
            }
            PrimitiveSubtype::Uint8 => {
                let mut unknown_value: u8 = 0;
                if self.validate_enum_members_and_calc_unknown_value::<u8>(enum_declaration.clone(), &mut unknown_value)
                {
                    enum_declaration.borrow_mut().unknown_value_unsigned = unknown_value as u64;
                }
            }
            PrimitiveSubtype::Uint16 => {
                let mut unknown_value: u16 = 0;
                if self
                    .validate_enum_members_and_calc_unknown_value::<u16>(enum_declaration.clone(), &mut unknown_value)
                {
                    enum_declaration.borrow_mut().unknown_value_unsigned = unknown_value as u64;
                }
            }
            PrimitiveSubtype::Uint32 => {
                let mut unknown_value: u32 = 0;
                if self
                    .validate_enum_members_and_calc_unknown_value::<u32>(enum_declaration.clone(), &mut unknown_value)
                {
                    enum_declaration.borrow_mut().unknown_value_unsigned = unknown_value as u64;
                }
            }
            PrimitiveSubtype::Uint64 => {
                let mut unknown_value: u64 = 0;
                if self
                    .validate_enum_members_and_calc_unknown_value::<u64>(enum_declaration.clone(), &mut unknown_value)
                {
                    enum_declaration.borrow_mut().unknown_value_unsigned = unknown_value as u64;
                }
            }
            PrimitiveSubtype::Bool | PrimitiveSubtype::Float32 | PrimitiveSubtype::Float64 => {
                //self.ctx.diagnostics.push_error(ErrEnumTypeMustBeIntegralPrimitive, enum_declaration.name,
                //            enum_declaration.subtype_ctor.r#type);
            }
        }
    }

    fn compile_union(&self, decl: Rc<RefCell<ast::Union>>) {
        let union_declaration: std::cell::Ref<'_, ast::Union> = decl.borrow();
        let mut ordinal_scope = Ordinal64Scope::new();
        // DeriveResourceness derive_resourceness(&union_declaration.resourceness);

        self.compile_attribute_list(&union_declaration.attributes);
        let mut contains_non_reserved_member = false;

        for member in union_declaration.members.iter() {
            let mut member = member.borrow_mut();

            self.compile_attribute_list(&member.attributes);

            let ordinal_result = ordinal_scope.insert(member.ordinal.value, member.ordinal.span.clone());
            if !ordinal_result.is_ok() {
                self.ctx.diagnostics.push_error(
                    Error::DuplicateUnionMemberOrdinal {
                        span: member.ordinal.span.clone(),
                        prev: ordinal_result.previous_occurrence(),
                    }
                    .into(),
                );
            }

            if member.maybe_used.as_ref().is_none() {
                continue;
            }

            contains_non_reserved_member = true;
            let member_used = member.maybe_used.as_mut().unwrap();

            self.compile_type_constructor(&mut member_used.type_ctor);

            if member_used.type_ctor.r#type.is_none() {
                continue;
            }

            if member_used.type_ctor.r#type.as_ref().unwrap().is_nullable() {
                // reporter()->Fail(ErrOptionalUnionMember, member_used.name);
                todo!()
            }

            // derive_resourceness.AddType(member_used.type_ctor->type);
        }

        if union_declaration.strictness == ast::Strictness::Strict && !contains_non_reserved_member {
            self.ctx.diagnostics.push_error(
                Error::StrictUnionMustHaveNonReservedMember {
                    span: union_declaration.name.span().unwrap(),
                }
                .into(),
            )
        }

        if let Some((ordinal, span)) = find_first_non_dense_ordinal(&ordinal_scope) {
            self.ctx
                .diagnostics
                .push_error(Error::NonDenseOrdinal { span, ordinal }.into());
        }
    }

    fn compile_alias(&self, decl: Rc<RefCell<ast::Alias>>) {
        let mut alias_declaration = decl.borrow_mut();

        self.compile_attribute_list(&alias_declaration.attributes);
        self.compile_type_constructor(&mut alias_declaration.partial_type_ctor);
    }

    fn validate_bits_members_and_calc_mask<T>(&self, decl: Rc<RefCell<ast::Bits>>, mask: &u64) -> bool {
        false
    }

    fn compile_bits(&self, decl: Rc<RefCell<ast::Bits>>) {
        let mut bits_declaration = decl.borrow_mut();

        self.compile_attribute_list(&bits_declaration.attributes);
        for member in bits_declaration.members.iter() {
            let member = member.borrow();
            self.compile_attribute_list(&member.attributes);
        }

        self.compile_type_constructor(&mut bits_declaration.subtype_ctor);
        if !bits_declaration.subtype_ctor.r#type.is_none() {
            return;
        }

        if !matches!(bits_declaration.subtype_ctor.r#type, Some(ast::Type::Primitive(_))) {
            panic!("ErrBitsTypeMustBeUnsignedIntegralPrimitive");
            //reporter()->Fail(ErrBitsTypeMustBeUnsignedIntegralPrimitive,
            //                bits_declaration->name.span().value(), bits_declaration->subtype_ctor->type);
            return;
        }

        // Validate constants.

        if let ast::Type::Primitive(primitive_type) = bits_declaration.subtype_ctor.r#type.as_ref().unwrap() {
            match primitive_type.subtype {
                PrimitiveSubtype::Uint8 => {
                    let mask = 0;
                    if !self.validate_bits_members_and_calc_mask::<u8>(decl.clone(), &mask) {
                            return;
                    }
                        
                    bits_declaration.mask = mask;
                },
                PrimitiveSubtype::Uint16 => {
                    let mask = 0;
                    if !self.validate_bits_members_and_calc_mask::<u16>(decl.clone(), &mask) {
                        return;
                    }
                        
                    bits_declaration.mask = mask;
                },
                PrimitiveSubtype::Uint32 => {
                    let mask= 0;
                    if !self.validate_bits_members_and_calc_mask::<u32>(decl.clone(), &mask) {
                        return;
                    }
                        
                    bits_declaration.mask = mask;
                },
                PrimitiveSubtype::Uint64 => {
                    let mask= 0;
                    if !self.validate_bits_members_and_calc_mask::<u64>(decl.clone(), &mask) {
                        return; 
                    }
                        
                    bits_declaration.mask = mask;
                },
                PrimitiveSubtype::Bool |
                PrimitiveSubtype::Int8 |
                PrimitiveSubtype::Int16 |
                PrimitiveSubtype::Int32 |
                PrimitiveSubtype::Int64 |
                //PrimitiveSubtype::ZxUchar:
                //PrimitiveSubtype::ZxUsize64:
                //PrimitiveSubtype::ZxUintptr64:
                PrimitiveSubtype::Float32 |
                PrimitiveSubtype::Float64 => {
                    panic!("ErrBitsTypeMustBeUnsignedIntegralPrimitive");
                    //reporter()->Fail(ErrBitsTypeMustBeUnsignedIntegralPrimitive,
                    //            bits_declaration->name.span().value(), bits_declaration->subtype_ctor->type);
                    return;
                }
            }
        }
    }

    fn compile_resource(&self, decl: Rc<RefCell<ast::Resource>>) {
        let mut resource_declaration = decl.borrow_mut();

        self.compile_attribute_list(&resource_declaration.attributes);
        self.compile_type_constructor(&mut resource_declaration.subtype_ctor);

        //if (resource_declaration.subtype_ctor.type.kind != Type::Kind::kPrimitive || static_cast<const PrimitiveType*>(resource_declaration->subtype_ctor->type)->subtype !=  PrimitiveSubtype::kUint32) {
        //reporter()->Fail(ErrResourceMustBeUint32Derived, resource_declaration->name.span().value(),
        //         resource_declaration->name);
        //}

        for property in resource_declaration.properties.iter_mut() {
            self.compile_attribute_list(&property.attributes);
            self.compile_type_constructor(&mut property.type_ctor);
        }

        // All properties have been compiled at this point, so we can reason about their types.
        let subtype_property = resource_declaration.lookup_property("subtype");

        if subtype_property.is_some() {
            // const Type* subtype_type = subtype_property->type_ctor->type;

            // If the |subtype_type is a |nullptr|, we are in a cycle, which means that the |subtype|
            // property could not possibly be an enum declaration.
            //if (subtype_type == nullptr || subtype_type->kind != Type::Kind::kIdentifier || static_cast<const IdentifierType*>(subtype_type)->type_decl->kind != Decl::Kind::kEnum) {
            //reporter()->Fail(ErrResourceSubtypePropertyMustReferToEnum, subtype_property->name,
            //            resource_declaration->name);
            //}
        } else {
            //reporter()->Fail(ErrResourceMissingSubtypeProperty, resource_declaration->name.span().value(),
            //                resource_declaration->name);
        }
    }

    fn compile_struct(&self, decl: Rc<RefCell<ast::Struct>>) {
        let struct_declaration = decl.borrow();
        // DeriveResourceness derive_resourceness(&struct_declaration->resourceness);

        self.compile_attribute_list(&struct_declaration.attributes);

        for member in struct_declaration.members.iter() {
            let mut member = member.borrow_mut();
            let r#type = member.type_ctor.r#type.clone();

            self.compile_attribute_list(&member.attributes);
            self.compile_type_constructor(&mut member.type_ctor);

            if r#type.is_none() {
                continue;
            }

            if member.maybe_default_value.is_some() {
                if !self.type_can_be_const(member.type_ctor.r#type.as_ref().unwrap()) {
                    //reporter().Fail(ErrInvalidStructMemberType, struct_declaration.name.span().value(),
                    //                NameIdentifier(member.name), default_value_type);
                } else if !self.resolve_constant(&mut member.maybe_default_value.as_mut().unwrap(), r#type) {
                    //reporter().Fail(ErrCouldNotResolveMemberDefault, member.name, NameIdentifier(member.name));
                }
            }

            // TODO: derive_resourceness.add_type(member.type_ctor.r#type);
        }
    }

    fn compile_const(&self, decl: Rc<RefCell<ast::Const>>) {
        let mut const_declaration = decl.borrow_mut();
        self.compile_attribute_list(&const_declaration.attributes);
        self.compile_type_constructor(&mut const_declaration.type_ctor);

        let const_type = const_declaration.type_ctor.r#type.clone();

        if const_declaration.type_ctor.r#type.is_none() {
            return;
        }

        if !self.type_can_be_const(const_type.as_ref().unwrap()) {
            panic!("ErrInvalidConstantType");
            //    self.ctx.diagnostics.fail(ErrInvalidConstantType, const_declaration.name.span().value(), const_type);
        } else if !self.resolve_constant(&mut const_declaration.value, const_type) {
            panic!("ErrCannotResolveConstantValue {}", const_declaration.name.span().unwrap().data);
            //    self.ctx.diagnostics.fail(ErrCannotResolveConstantValue, const_declaration.name.span().value());
        }
    }

    fn compile_protocol(&self, decl: Rc<RefCell<ast::Protocol>>) {
        let protocol_declaration = decl.borrow();
        self.compile_attribute_list(&protocol_declaration.attributes);

        for method in protocol_declaration.methods.iter() {
            let mut method = method.borrow_mut();

            if let Some(typ_ctor) = method.maybe_request.as_mut() {
                self.compile_type_constructor(typ_ctor);

                if let Some(ref r#type) = typ_ctor.r#type {
                    if let ast::Type::Identifier(id_type) = r#type {
                        let mut decl = id_type.decl.clone();
                        self.compile_decl(&mut decl);
                        //  self.check_no_default_members(decl);
                        //  self.check_payload_decl_kind(method.name, decl);
                    } else {
                        // reporter()->Fail(ErrInvalidMethodPayloadType, method.name, type);
                    }
                }
            }

            if let Some(typ_ctor) = method.maybe_response.as_mut() {
                self.compile_type_constructor(typ_ctor);

                if let Some(ref r#type) = typ_ctor.r#type {
                    if let ast::Type::Identifier(id_type) = r#type {
                        let mut decl = id_type.decl.clone();
                        self.compile_decl(&mut decl);
                        /*

                        if (method.HasResultUnion()) {
                            ZX_ASSERT(decl.kind == Decl::Kind::kUnion);
                            let result_union = static_cast<const Union*>(decl);
                            ZX_ASSERT(!result_union.members.empty());
                            ZX_ASSERT(result_union.members[0].maybe_used);
                            let success_variant_type = result_union->members[0].maybe_used->type_ctor->type;

                            if (success_variant_type) {
                                if (success_variant_type->kind != Type::Kind::kIdentifier) {
                                    reporter()->Fail(ErrInvalidMethodPayloadType, method.name, success_variant_type);
                                } else {
                                    let success_decl = static_cast<const IdentifierType*>(success_variant_type)->type_decl;
                                    CheckNoDefaultMembers(success_decl);
                                    CheckPayloadDeclKind(method.name, success_decl);
                                }
                            }

                            ValidateDomainErrorType(result_union);
                        } else {
                            CheckNoDefaultMembers(decl);
                            CheckPayloadDeclKind(method.name, decl);
                        }*/
                    } else {
                        //  reporter()->Fail(ErrInvalidMethodPayloadType, method.name, type);
                    }
                }
            }
        }
    }

    fn resolve_constant(&self, constant: &mut ast::Constant, opt_type: Option<ast::Type>) -> bool {
        if constant.compiled() {
            return constant.is_resolved();
        }
        constant.set_compiled(true);

        match constant {
            ast::Constant::BinaryOperator(constant) => {
                let binary_operator_constant = constant;

                if !self.resolve_constant(binary_operator_constant.lhs.as_mut(), opt_type.clone()) {
                    return false;
                }

                if !self.resolve_constant(binary_operator_constant.rhs.as_mut(), opt_type.clone()) {
                    return false;
                }

                match binary_operator_constant.op {
                    ast::ConstantOp::Or => {
                        return self.resolve_or_operator_constant(
                            binary_operator_constant,
                            opt_type,
                            &binary_operator_constant.lhs.value(),
                            &binary_operator_constant.rhs.value(),
                        );
                    }
                }
            }
            ast::Constant::Identifier(idenifier) => self.resolve_identifier_constant(idenifier, opt_type),
            ast::Constant::Literal(literal) => self.resolve_literal_constant(literal, opt_type),
        }
    }

    fn resolve_or_operator_constant(
        &self,
        constant: &ast::BinaryOperatorConstant,
        opt_type: Option<ast::Type>,
        left_operand: &ConstantValue,
        right_operand: &ConstantValue,
    ) -> bool {
        /*assert!(left_operand.kind == right_operand.kind, "left and right operands of or operator must be of the same kind");
        assert!(opt_type.is_some(), "type inference not implemented for or operator");

        let r#type = UnderlyingType(opt_type.value());
        if (r#type == nullptr) {
            return false;
        }
        if (r#type.kind != Type::Kind::kPrimitive) {
            return reporter().Fail(ErrOrOperatorOnNonPrimitiveValue, constant.span);
        }

        std::unique_ptr<ConstantValue> left_operand_u64;
        std::unique_ptr<ConstantValue> right_operand_u64;

        if (!left_operand.Convert(ConstantValue::Kind::kUint64, &left_operand_u64)) {
            return false;
        }

        if (!right_operand.Convert(ConstantValue::Kind::kUint64, &right_operand_u64)) {
            return false;
        }

        NumericConstantValue<uint64_t> result = *static_cast<NumericConstantValue<uint64_t>*>(left_operand_u64.get()) | *static_cast<NumericConstantValue<uint64_t>*>(right_operand_u64.get());
        std::unique_ptr<ConstantValue> converted_result;
        if (!result.Convert(ConstantValuePrimitiveKind(static_cast<const PrimitiveType*>(type)->subtype), &converted_result)) {
            return false;
        }

        constant.resolve_to(converted_result, r#type);
        return true;
        */

        return false;
    }

    fn constant_value_primitive_kind(&self, subtype: ast::PrimitiveSubtype) -> ast::ConstantValueKind {
        match subtype {
            PrimitiveSubtype::Bool => todo!(),
            PrimitiveSubtype::Int8 => todo!(),
            PrimitiveSubtype::Int16 => todo!(),
            PrimitiveSubtype::Int32 => todo!(),
            PrimitiveSubtype::Int64 => todo!(),
            PrimitiveSubtype::Uint8 => todo!(),
            PrimitiveSubtype::Uint16 => todo!(),
            PrimitiveSubtype::Uint32 => todo!(),
            PrimitiveSubtype::Uint64 => todo!(),
            PrimitiveSubtype::Float32 => todo!(),
            PrimitiveSubtype::Float64 => todo!(),
        }
    }

    fn resolve_identifier_constant(
        &self,
        identifier_constant: &mut ast::IdentifierConstant,
        opt_type: Option<ast::Type>,
    ) -> bool {
        if opt_type.is_some() {
            assert!(
                self.type_can_be_const(&opt_type.clone().unwrap()),
                "resolving identifier constant to non-const-able type"
            );
        }

        let reference = &identifier_constant.reference;
        log::warn!("const ref: {:?}", reference);
        let resolved = reference.resolved().unwrap();

        let mut parent = resolved.element_or_parent_decl();
        let target = resolved.element();
        self.compile_decl(&mut parent);

        let mut const_type: Option<ast::Type> = None;
        let mut const_val: Option<ast::ConstantValue> = None;

        match target {
            ast::Element::Builtin { .. } => {
                // TODO(https://fxbug.dev/99665): In some cases we want to return a more specific
                // error message from here, but right now we can't due to the way
                // TypeResolver::ResolveConstraintAs tries multiple interpretations.
                return false;
            }
            ast::Element::Const { inner } => {
                let const_decl = inner.borrow();

                if !const_decl.value.is_resolved() {
                    return false;
                }

                const_type = const_decl.type_ctor.r#type.clone();
                const_val = Some(const_decl.value.value());
            }
            ast::Element::EnumMember { inner } => {
                if let ast::Declaration::Bits { decl } = parent.clone() {
                    let parent = decl.borrow();

                    const_type = parent.subtype_ctor.r#type.clone();
                    let member = inner.borrow();
                    
                    if !member.value.is_resolved() {
                        return false;
                    }

                    const_val = Some(member.value.value());
                } else {
                    panic!("parent is not Enum declaration")
                }
            }
            ast::Element::BitsMember { inner } => {
                if let ast::Declaration::Bits { decl } = &parent {
                    let parent = decl.borrow();

                    const_type = parent.subtype_ctor.r#type.clone();
                    let member = inner.borrow();
                    
                    if !member.value.is_resolved() {
                        return false;
                    }

                    const_val = Some(member.value.value());
                } else {
                    panic!("parent is not Bits declaration")
                }
            }
            _ => {
                panic!("ErrExpectedValueButGotType {:?}", reference.resolved().unwrap().name());
                // return reporter().Fail(ErrExpectedValueButGotType, reference.span(), reference.resolved().name());
                return false;
            },
        }

        assert!(const_val.is_none(), "did not set const_val");
        assert!(const_type.is_none(), "did not set const_type");
        let const_val = const_val.unwrap();
        let const_type = const_type.unwrap();

        let mut resolved_val: ConstantValue = ConstantValue::Bool(false);
        let r#type = if opt_type.is_some() { opt_type.unwrap() } else { const_type.clone() };
        

        fn fail_cannot_convert() -> bool{
            panic!("ErrTypeCannotBeConvertedToType");
            // return reporter().Fail(ErrTypeCannotBeConvertedToType, reference.span(), identifier_constant, const_type, type);
            false
        }
  

        match &r#type {
            ast::Type::String(_) => {
                if !self.type_is_convertible_to(&const_type, &r#type) {
                    return fail_cannot_convert();
                }
                    
                if !const_val.convert(ast::ConstantValueKind::String, &mut resolved_val) {
                    return fail_cannot_convert();
                }
            },
            ast::Type::Primitive(primitive_type) => {
                if !const_val.convert(self.constant_value_primitive_kind(primitive_type.subtype), &mut resolved_val) {
                    return fail_cannot_convert();
                }
            },
            ast::Type::Identifier(identifier_type) => {
                let primitive_type: Rc<ast::PrimitiveType>;

                match identifier_type.decl.clone() {
                    ast::Declaration::Enum { decl } => {
                        let enum_decl = decl.borrow();
                        if enum_decl.subtype_ctor.r#type.is_none() {
                            return false;
                        }
                        
                        if let ast::Type::Primitive(typ) = enum_decl.subtype_ctor.r#type.as_ref().unwrap() {
                            primitive_type = typ.clone();
                        } else {
                            panic!("non primitive type")
                        }
                    }
                    ast::Declaration::Bits { decl } => {
                        let bits_decl = decl.borrow();
                        if bits_decl.subtype_ctor.r#type.is_none() {
                            return false;
                        }

                         if let ast::Type::Primitive(typ) = bits_decl.subtype_ctor.r#type.as_ref().unwrap() {
                            primitive_type = typ.clone();
                        } else {
                            panic!("non primitive type")
                        }
                    }
                    _ => panic!("identifier not of const-able type.")
                }

                fn fail_with_mismatched_type(type_name: &ast::Name)-> bool {
                    panic!("ErrMismatchedNameTypeAssignment");
                    false
                    //return reporter()->Fail(ErrMismatchedNameTypeAssignment, identifier_constant->span,
                    //            identifier_type->type_decl->name, type_name);
                }

                match &parent.clone() {
                    ast::Declaration::Const { ref decl } => {
                        if const_type.name() != identifier_type.decl.name() {
                            return fail_with_mismatched_type(&const_type.name());
                        }
                    }
                    ast::Declaration::Bits{ .. } |
                    ast::Declaration::Enum{ .. } => {
                        if parent.name() != identifier_type.decl.name() {
                            return fail_with_mismatched_type(&parent.name());
                        }
                    }
                    _=> panic!("identifier not of const-able type.")
                }

                if !const_val.convert(self.constant_value_primitive_kind(primitive_type.subtype), &mut resolved_val) {
                    return fail_cannot_convert()
                }        
            },
            _ => panic!("identifier not of const-able type."),
        }

        identifier_constant.resolve_to(resolved_val, r#type);
        true
    }

    fn infer_type(&self, literal: &mut ast::LiteralConstant) -> ast::Type { 
        match &literal.literal {
            ast::Literal::StringValue(string_literal, span) => {
                let inferred_size = todo!(); // TODO: string_literal_length(span.data);
                return ast::Type::String(self.typespace().get_string_type(inferred_size));
            }
            ast::Literal::NumericValue(_, _) => ast::Type::UntypedNumeric(self.typespace().get_untyped_numeric_type()),
            ast::Literal::BoolValue(_, _) => ast::Type::Primitive(self.typespace().get_primitive_type(ast::PrimitiveSubtype::Bool))
            //case RawLiteral::Kind::kDocComment:
            //    return typespace()->GetUnboundedStringType();
        }
    }

    fn resolve_literal_constant(
        &self,
        literal_constant: &mut ast::LiteralConstant,
        opt_type: Option<ast::Type>,
    ) -> bool {
        let inferred_type = self.infer_type(literal_constant);

        let r#type = if opt_type.is_some() {
            opt_type.unwrap()
        } else {
            todo!("inferred_type");
            // inferred_type
        };

        if !self.type_is_convertible_to(&inferred_type, &r#type) {
        //self.ctx.diagnostics.push_error(
        //    ErrTypeCannotBeConvertedToType,
        //    literal_constant.literal.span(),
        //    literal_constant,
        //    inferred_type,
        //    r#type,
        //);
        // return false;
        }

        match literal_constant.literal {
            ast::Literal::StringValue(_, _) => {
                literal_constant.resolve_to(
                    ConstantValue::String(literal_constant.literal.span().data),
                    ast::Type::String(self.typespace().get_unbounded_string_type()),
                );

                true
            }
            ast::Literal::BoolValue(value, _) => {
                literal_constant.resolve_to(
                    ConstantValue::Bool(value),
                    ast::Type::Primitive(self.typespace().get_primitive_type(ast::PrimitiveSubtype::Bool)),
                );

                true
            }
            ast::Literal::NumericValue(_, _) => {
                if let ast::Type::Primitive(ref prim) = r#type {
                    // Even though `untyped numeric` is convertible to any numeric type, we
                    // still need to check for overflows which is done in
                    // ResolveLiteralConstantKindNumericLiteral.
                    match prim.subtype {
                        ast::PrimitiveSubtype::Int8
                        | ast::PrimitiveSubtype::Int16
                        | ast::PrimitiveSubtype::Int32
                        | ast::PrimitiveSubtype::Int64
                        | ast::PrimitiveSubtype::Uint8
                        | ast::PrimitiveSubtype::Uint16
                        | ast::PrimitiveSubtype::Uint32
                        | ast::PrimitiveSubtype::Uint64
                        | ast::PrimitiveSubtype::Float64
                        | ast::PrimitiveSubtype::Float32 => {
                            self.resolve_literal_constant_kind_numeric_literal(literal_constant, prim.subtype, r#type)
                        }
                        _ => panic!("should not have any other primitive type reachable"),
                    }
                } else {
                    panic!("should have a primitive type")
                }
            }
        }
    }

    /*fn infer_type(&self, constant: &mut ast::Constant) -> Rc<ast::Type> {
        match constant {
            Constant::Literal(c) =>{
                let literal = c.literal;
                match literal {
                    ast::Literal::StringValue(l, span) => {
                        let string_literal = l;
                        let inferred_size = string_literal_length(span.data);
                        return typespace().GetStringType(inferred_size);
                    }
                    ast::Literal::NumericValue => self.typespace().get_untyped_numeric_type(),
                    ast::Literal::BoolValue => self.typespace().get_primitiveType(ast::PrimitiveSubtype::Bool),
                    ast::Literal::DocComment => self.typespace().get_unbounded_string_type()
                }
                return nullptr;
            }

            Constant::Identifier(_) => {
                if !self.resolve_constant(constant, None) {
                    return None;
                }

                constant.type
            }
            _ => unimplemented!("type inference not implemented for binops")
        }
    }*/

    fn resolve_literal_constant_kind_numeric_literal(
        &self,
        literal_constant: &mut ast::LiteralConstant,
        subtype: ast::PrimitiveSubtype,
        r#type: ast::Type,
    ) -> bool {
        let span = literal_constant.literal.span();
        let string_data = String::from(span.data);

        let result: Result<()> = try {
            match subtype {
                ast::PrimitiveSubtype::Float64 => {
                    let value = string_data.parse::<f64>()?;
                    literal_constant.resolve_to(ConstantValue::Float64(value), r#type);
                }
                ast::PrimitiveSubtype::Float32 => {
                    let value = string_data.parse::<f32>()?;
                    literal_constant.resolve_to(ConstantValue::Float32(value), r#type);
                }
                ast::PrimitiveSubtype::Int8 => {
                    let value = string_data.parse::<i8>()?;
                    literal_constant.resolve_to(ConstantValue::Int8(value), r#type);
                }
                ast::PrimitiveSubtype::Int16 => {
                    let value = string_data.parse::<i16>()?;
                    literal_constant.resolve_to(ConstantValue::Int16(value), r#type);
                }
                ast::PrimitiveSubtype::Int32 => {
                    let value = string_data.parse::<i32>()?;
                    literal_constant.resolve_to(ConstantValue::Int32(value), r#type);
                }
                ast::PrimitiveSubtype::Int64 => {
                    let value = string_data.parse::<i64>()?;
                    literal_constant.resolve_to(ConstantValue::Int64(value), r#type);
                }
                ast::PrimitiveSubtype::Uint8 => {
                    let value = string_data.parse::<u8>()?;
                    literal_constant.resolve_to(ConstantValue::Uint8(value), r#type);
                }
                ast::PrimitiveSubtype::Uint16 => {
                    let value = string_data.parse::<u16>()?;
                    literal_constant.resolve_to(ConstantValue::Uint16(value), r#type);
                }
                ast::PrimitiveSubtype::Uint32 => {
                    let value = string_data.parse::<u32>()?;
                    literal_constant.resolve_to(ConstantValue::Uint32(value), r#type);
                }
                ast::PrimitiveSubtype::Uint64 => {
                    let value = string_data.parse::<u64>()?;
                    literal_constant.resolve_to(ConstantValue::Uint64(value), r#type);
                }
                _ => panic!("non numeric value"),
            }
        };

        match result {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn type_is_convertible_to(&self, s: &ast::Type, typ: &ast::Type) -> bool {
        true
    }

    fn type_can_be_const(&self, typ: &ast::Type) -> bool {
        match typ {
            ast::Type::String(_) => return !typ.is_nullable(),
            ast::Type::Primitive(_) => true,
            ast::Type::Identifier(identifier_type) => match identifier_type.decl {
                ast::Declaration::Enum { .. } | ast::Declaration::Bits { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn compile_type_constructor(&self, type_ctor: &mut ast::TypeConstructor) {
        if type_ctor.r#type.is_some() {
            return;
        }

        let all_libs = self.ctx.all_libraries.borrow();
        let type_resolver = TypeResolver::new(self);
        let typespace = all_libs.typespace();

        type_ctor.r#type = typespace.create(
            &type_resolver,
            &type_ctor.layout,
            &type_ctor.parameters,
            &type_ctor.constraints,
        );
    }
}
