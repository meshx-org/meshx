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

type MemberValidator<T> = dyn FnMut(T, Vec<ast::Attribute>, ast::Identifier) -> Result<(), DiagnosticsError>;

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
        println!("compile_decl start");

        {
            let mut decl_stack = self.decl_stack.borrow_mut();

            decl.set_compiling(true);
            decl_stack.push(decl.clone());
        }

        match decl {
            ast::Declaration::Const { decl } => {
                println!("compile_decl const");
                self.compile_const(decl.clone())
            }
            ast::Declaration::Enum { decl } => self.compile_enum(decl.clone()),
            ast::Declaration::Struct { decl } => {
                println!("compile_decl struct {:#?}", decl);
                self.compile_struct(decl.clone())
            }
            ast::Declaration::Union { decl } => self.compile_union(decl.clone()),
            ast::Declaration::Resource { .. } => {}
            ast::Declaration::Alias { .. } => {}
            ast::Declaration::Protocol { .. } => {}
            _ => todo!(),
        }

        {
            let mut decl_stack = self.decl_stack.borrow_mut();

            decl.set_compiling(false);
            decl.set_compiled(true);
            decl_stack.pop();
        }

        println!("compile_decl end");
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
                    // reporter().Fail(ErrCouldNotResolveMember, member.name, decl.kind);
                    continue;
                }

                let value: MemberType = member.value.value().into();
                let value_result = value_scope.insert(value, member.name.span.clone());
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
            //    self.ctx.diagnostics.fail(ErrInvalidConstantType, const_declaration.name.span().value(), const_type);
        } else if !self.resolve_constant(&mut const_declaration.value, const_type) {
            //    self.ctx.diagnostics.fail(ErrCannotResolveConstantValue, const_declaration.name.span().value());
        }
    }

    fn resolve_constant(&self, constant: &mut ast::Constant, opt_type: Option<ast::Type>) -> bool {
        if constant.compiled() {
            return constant.is_resolved();
        }
        constant.set_compiled(true);

        match constant {
            ast::Constant::Identifier(idenifier) => self.resolve_identifier_constant(idenifier, opt_type),
            ast::Constant::Literal(literal) => self.resolve_literal_constant(literal, opt_type),
        }
    }

    fn resolve_identifier_constant(
        &self,
        identifier_constant: &mut ast::IdentifierConstant,
        opt_type: Option<ast::Type>,
    ) -> bool {
        if opt_type.is_some() {
            assert!(
                self.type_can_be_const(&opt_type.unwrap()),
                "resolving identifier constant to non-const-able type"
            );
        }

        let reference = &identifier_constant.reference;
        let resolved = reference.resolved().unwrap();

        let parent = resolved.element_or_parent_decl();
        let target = resolved.element();
        // TODO: self.compile_decl(parent);

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
            ast::Element::Bits => todo!(),
            ast::Element::Enum { .. } => todo!(),
            ast::Element::Resource => todo!(),
            ast::Element::Alias { .. } => todo!(),
            ast::Element::Struct { .. } => todo!(),
            ast::Element::Protocol { .. } => todo!(),
            ast::Element::NewType => todo!(),
            ast::Element::Table { .. } => todo!(),
            ast::Element::Union { .. } => todo!(),
            ast::Element::Overlay => todo!(),
            ast::Element::StructMember { .. } => todo!(),
            ast::Element::ProtocolMethod { .. } => todo!(),
            ast::Element::TableMember { .. } => todo!(),
            ast::Element::UnionMember { .. } => todo!(),
            ast::Element::EnumMember { .. } => todo!(),
            _ => todo!(),
        }

        false
    }

    fn resolve_literal_constant(
        &self,
        literal_constant: &mut ast::LiteralConstant,
        opt_type: Option<ast::Type>,
    ) -> bool {
        // let inferred_type = self.infer_type(literal_constant);

        let r#type = if opt_type.is_some() {
            opt_type.unwrap()
        } else {
            todo!("inferred_type");
            // inferred_type
        };

        //if !self.type_is_convertible_to(inferred_type, r#type) {
        //self.ctx.diagnostics.push_error(
        //    ErrTypeCannotBeConvertedToType,
        //    literal_constant.literal.span(),
        //    literal_constant,
        //    inferred_type,
        //    r#type,
        //);
        // return false;
        //}

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

    fn type_is_convertible_to(&self, s: ast::Type, typ: Rc<ast::Type>) -> bool {
        true
    }

    fn type_can_be_const(&self, typ: &ast::Type) -> bool {
        match typ {
            ast::Type::String(_) => return !typ.is_nullable(),
            ast::Type::Primitive(_) => true,
            ast::Type::Identifier(identifier_type) => match identifier_type.decl {
                ast::Declaration::Enum { .. } | ast::Declaration::Bits => true,
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
