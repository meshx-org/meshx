use anyhow::Result;
use std::{cell::RefCell, rc::Rc, str::FromStr};

use crate::ast::{self, Constant, ConstantTrait, ConstantValue, PrimitiveSubtype, WithSpan};

use super::{type_resolver::TypeResolver, typespace::Typespace, Context};

pub(crate) struct CompileStep<'ctx, 'd> {
    ctx: &'ctx mut Context<'d>,
}

enum ParseNumericResult {
    kOutOfBounds,
    kMalformed,
    kSuccess,
}

impl<'ctx, 'd> CompileStep<'ctx, 'd> {
    pub fn new(ctx: &'ctx mut Context<'d>) -> Self {
        Self { ctx }
    }

    fn typespace(&self) -> Rc<Typespace> {
        self.ctx.all_libraries.borrow().typespace()
    }

    pub(crate) fn run(&self) -> bool {
        log::debug!("running compile step");

        // CompileAttributeList(library()->attributes.get());
        for (_, decl) in self.ctx.library.declarations.borrow().all.flat_iter() {
            self.compile_decl(decl);
        }

        true
    }

    fn compile_decl(&self, decl: &ast::Declaration) {
        match decl {
            ast::Declaration::Const { decl } => self.compile_const(decl.clone()),
            ast::Declaration::Resource { decl } => {}
            ast::Declaration::Alias { decl } => {}
            ast::Declaration::Struct { decl } => {}
            ast::Declaration::Protocol { decl } => {}
            _ => {}
        }
    }

    fn compile_const(&self, const_declaration: Rc<RefCell<ast::Const>>) {
        let mut const_declaration = const_declaration.borrow_mut();
        // self.compile_attribute_list(const_declaration->attributes.get());
        self.compile_type_constructor(&mut const_declaration.type_ctor);

        let const_type = const_declaration.type_ctor.r#type.clone();

        if const_declaration.type_ctor.r#type.is_none() {
            return;
        }

        if !self.type_can_be_const(const_type.clone().unwrap()) {
            //    self.ctx.diagnostics.fail(ErrInvalidConstantType, const_declaration.name.span().value(), const_type);
        } else if !self.resolve_constant(&mut const_declaration.value, const_type) {
            //    self.ctx.diagnostics.fail(ErrCannotResolveConstantValue, const_declaration.name.span().value());
        }
    }

    fn resolve_constant(&self, constant: &mut ast::Constant, opt_type: Option<ast::Type>) -> bool {
        // assert!(constant != nullptr);

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
                self.type_can_be_const(opt_type.unwrap()),
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
            ast::Element::Builtin { inner } => {
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
            ast::Element::Enum => todo!(),
            ast::Element::Resource => todo!(),
            ast::Element::Alias { inner } => todo!(),
            ast::Element::Struct { inner } => todo!(),
            ast::Element::Protocol { inner } => todo!(),
            ast::Element::NewType => todo!(),
            ast::Element::Table => todo!(),
            ast::Element::Union => todo!(),
            ast::Element::Overlay => todo!(),
            ast::Element::StructMember { inner } => todo!(),
            ast::Element::ProtocolMethod { inner } => todo!(),
            ast::Element::TableMember => todo!(),
            ast::Element::UnionMember => todo!(),
            ast::Element::EnumMember => todo!(),
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

    fn type_can_be_const(&self, typ: ast::Type) -> bool {
        match typ {
            ast::Type::String(_) => return !typ.is_nullable(),
            ast::Type::Primitive(_) => true,
            ast::Type::Identifier(identifier_type) => match identifier_type.type_decl {
                ast::Declaration::Enum | ast::Declaration::Bits => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn compile_type_constructor(&self, type_ctor: &mut ast::TypeConstructor) {
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
