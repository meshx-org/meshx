use midlgen::ir::{self, EncodedCompoundIdentifier, Resourceness};

use crate::{
    ast::{self, ConstantTrait},
    compiler, ExperimentalFlags,
};

pub struct JSONGenerator {
    compilation: compiler::Compilation,
}

enum TypeKind {
    Concrete,
    Parameterized,
    RequestPayload,
    ResponsePayload,
}

impl JSONGenerator {
    pub fn new(compilation: compiler::Compilation, flags: ExperimentalFlags) -> Self {
        Self { compilation }
    }

    pub fn produce(&self) -> serde_json::Value {
        // &self.compilation.library_name

        let ir = ir::Root {
            name: ir::EncodedLibraryIdentifier("TODO".to_string()),
            const_declarations: self.generate_const_declarations(&self.compilation.declarations.consts),
            enum_declarations: self.generate_enum_declarations(&self.compilation.declarations.enums),
            struct_declarations: self.generate_struct_declarations(&self.compilation.declarations.structs),
            union_declarations: self.generate_union_declarations(&self.compilation.declarations.unions),
            protocol_declarations: self.generate_protocol_declarations(&self.compilation.declarations.protocols),
            table_declarations: vec![],
            bits_declarations: vec![],
            experiments: vec![],
            library_dependencies: vec![],
        };

        serde_json::to_value(ir).unwrap()
    }

    fn generate_const_declarations(&self, decls: &Vec<ast::Declaration>) -> Vec<ir::Const> {
        let mut ir_decls = vec![];

        for decl in decls {
            if let ast::Declaration::Const { decl } = decl {
                ir_decls.push(self.generate_const(decl.borrow().clone()))
            }
        }

        ir_decls
    }

    fn generate_struct_declarations(&self, decls: &Vec<ast::Declaration>) -> Vec<ir::Struct> {
        decls
            .into_iter()
            .map(|decl| {
                if let ast::Declaration::Struct { decl } = decl {
                    self.generate_struct(decl.borrow().clone())
                } else {
                    panic!("")
                }
            })
            .collect()
    }

    fn generate_enum_declarations(&self, decls: &Vec<ast::Declaration>) -> Vec<ir::Enum> {
        decls
            .into_iter()
            .map(|decl| {
                if let ast::Declaration::Enum { decl } = decl {
                    self.generate_enum(decl.borrow().clone())
                } else {
                    panic!("")
                }
            })
            .collect()
    }

    fn generate_union_declarations(&self, decls: &Vec<ast::Declaration>) -> Vec<ir::Union> {
        decls
            .into_iter()
            .map(|decl| {
                if let ast::Declaration::Union { decl } = decl {
                    self.generate_union(decl.borrow().clone())
                } else {
                    panic!("")
                }
            })
            .collect()
    }

    fn generate_protocol_declarations(&self, decls: &Vec<ast::Declaration>) -> Vec<ir::Protocol> {
        decls
            .into_iter()
            .map(|decl| {
                if let ast::Declaration::Protocol { decl } = decl {
                    self.generate_protocol(decl.borrow().clone())
                } else {
                    panic!("")
                }
            })
            .collect()
    }

    fn generate_location(&self, value: ast::Span) -> ir::Location {
        ir::Location {
            filename: "TODO".to_string(),
            line: 0,
            column: 0,
            length: 0,
        }
    }

    fn generate_name(&self, name: &ast::Name) -> ir::EncodedCompoundIdentifier {
        // TODO(https://fxbug.dev/92422): NameFlatName omits the library name for builtins,
        // since we want error messages to say "uint32" not "fidl/uint32". However,
        // builtins MAX and HEAD can end up in the JSON IR as identifier constants,
        // and to satisfy the schema we must produce a proper compound identifier
        // (with a library name). We should solve this in a cleaner way.
        if name.is_intrinsic() && name.decl_name() == "MAX" {
            EncodedCompoundIdentifier("fidl/MAX".to_string())
        } else if name.is_intrinsic() && name.decl_name() == "HEAD" {
            EncodedCompoundIdentifier("fidl/HEAD".to_string())
        } else {
            EncodedCompoundIdentifier(ast::name_flat_name(name))
        }
    }

    fn generate_naming_context(&self, name: ast::Name) -> ir::NamingContext {
        if let Some(n) = name.as_anonymous() {
            n.context.context()
        } else {
            vec![name.decl_name()]
        }
    }

    fn generate_nullable(&self, value: &ast::Nullability) -> bool {
        match value {
            ast::Nullability::Nullable => true,
            ast::Nullability::Nonnullable => false,
        }
    }

    fn generate_primitive_subtype(&self, value: &ast::PrimitiveSubtype) -> ir::PrimitiveSubtype {
        match value {
            ast::PrimitiveSubtype::Bool => ir::PrimitiveSubtype::Bool,
            ast::PrimitiveSubtype::Int8 => ir::PrimitiveSubtype::Int8,
            ast::PrimitiveSubtype::Int16 => ir::PrimitiveSubtype::Int16,
            ast::PrimitiveSubtype::Int32 => ir::PrimitiveSubtype::Int32,
            ast::PrimitiveSubtype::Int64 => ir::PrimitiveSubtype::Int64,
            ast::PrimitiveSubtype::Uint8 => ir::PrimitiveSubtype::Uint8,
            ast::PrimitiveSubtype::Uint16 => ir::PrimitiveSubtype::Uint16,
            ast::PrimitiveSubtype::Uint32 => ir::PrimitiveSubtype::Uint32,
            ast::PrimitiveSubtype::Uint64 => ir::PrimitiveSubtype::Uint64,
            ast::PrimitiveSubtype::Float32 => ir::PrimitiveSubtype::Float32,
            ast::PrimitiveSubtype::Float64 => ir::PrimitiveSubtype::Float64,
        }
    }

    fn generate_internal_subtype(&self, value: &ast::InternalSubtype) -> ir::InternalSubtype {
        match value {
            ast::InternalSubtype::FrameworkErr => ir::InternalSubtype::FrameworkErr,
        }
    }

    fn generate_type_shape(&self) -> ir::TypeShape {
        ir::TypeShape {
            inline_size: 0,
            alignment: 0,
            depth: 0,
            max_handles: 0,
            max_out_of_line: 0,
            has_padding: false,
            has_flexible_envelope: false,
        }
    }

    fn generate_field_shape(&self) -> ir::FieldShape {
        ir::FieldShape { offset: 0, padding: 0 }
    }

    fn generate_type(&self, value: ast::Type) -> ir::Type {
        match value {
            ast::Type::Vector(r#type) => ir::Type::VectorType {
                element_type: Box::from(self.generate_type(r#type.element_type.clone())),
                element_count: Some(r#type.element_size()),
                nullable: self.generate_nullable(&r#type.constraints.nullabilty()),
            },
            ast::Type::Identifier(ref r#type) => ir::Type::IdentifierType {
                identifier: self.generate_name(&r#type.name),
                nullable: self.generate_nullable(&r#type.constraints.nullabilty()),
                type_shape_v2: self.generate_type_shape(),
            },
            ast::Type::String(ref r#type) => {
                let mut element_count = None;

                if r#type.max_size() < std::u32::MAX {
                    element_count = Some(r#type.max_size());
                }

                ir::Type::StringType {
                    element_count,
                    nullable: self.generate_nullable(&r#type.constraints.nullabilty()),
                    type_shape_v2: self.generate_type_shape(),
                }
            }
            ast::Type::Internal(r#type) => ir::Type::InternalType {
                internal_subtype: self.generate_internal_subtype(&r#type.subtype),
                type_shape_v2: self.generate_type_shape(),
            },
            ast::Type::Primitive(r#type) => ir::Type::PrimitiveType {
                primitive_subtype: self.generate_primitive_subtype(&r#type.subtype),
            },
            _ => panic!("unsupported type: {:?}", value),
        }
    }

    fn generate_type_and_from_alias(&self, parent_type_kind: TypeKind, value: ast::TypeConstructor) -> ir::Type {
        let r#type = value.clone().r#type;
        // let invocation = value.resolved_params;

        // if (ShouldExposeAliasOfParametrizedType(*type)) {
        //        if invocation.from_alias {
        //            GenerateParameterizedType(parent_type_kind, type, invocation.from_alias.partial_type_ctor.get());
        //        } else {
        //            GenerateParameterizedType(parent_type_kind, type, value);
        //        }
        //    GenerateExperimentalMaybeFromAlias(invocation);
        //
        //    return;
        //}

        self.generate_type(r#type.unwrap())
    }

    fn generate_literal(&self, value: ast::LiteralConstant) -> ir::Literal {
        match value.literal {
            ast::Literal::NumericValue(value, _) => ir::Literal::NumericLiteral { value },
            ast::Literal::StringValue(value, _) => ir::Literal::StringLiteral { value },
            ast::Literal::BoolValue(value, _) => ir::Literal::BoolLiteral { value },
        }
    }

    fn generate_ordinal64(&self, value: ast::RawOrdinal64) -> u64 {
        value.value
    }

    fn generate_constant(&self, value: ast::Constant) -> ir::Constant {
        match value {
            ast::Constant::Identifier(identifier) => ir::Constant::Identifier {
                value: identifier.value().to_string(),
                expression: identifier.span().data.clone(),
                identifier: self.generate_name(&identifier.reference.resolved().unwrap().name()),
            },
            ast::Constant::Literal(literal) => ir::Constant::LiteralConstant {
                value: literal.value().to_string(),
                expression: literal.span().data.clone(),
                literal: self.generate_literal(literal),
            },
        }
    }

    fn generate_const(&self, value: ast::Const) -> ir::Const {
        ir::Const {
            name: self.generate_name(&value.name),
            location: self.generate_location(value.span),
            r#type: self.generate_type_and_from_alias(TypeKind::Concrete, value.type_ctor),
            value: self.generate_constant(value.value),
        }
    }

    fn generate_union(&self, value: ast::Union) -> ir::Union {
        let mut members = vec![];

        for member in value.members {
            let member = member.borrow();

            if let Some(ref used) = member.maybe_used {
                members.push(ir::UnionMember {
                    name: Some(self.generate_identifier(used.name.clone())),
                    r#type: Some(self.generate_type(used.type_ctor.r#type.as_ref().unwrap().clone())),
                    reserved: false,
                    ordinal: self.generate_ordinal64(member.ordinal.clone()),
                    max_out_of_line: 0,
                })
            } else {
                members.push(ir::UnionMember {
                    name: None,
                    r#type: None,
                    reserved: true,
                    ordinal: self.generate_ordinal64(member.ordinal.clone()),
                    max_out_of_line: 0,
                })
            }
        }

        ir::Union {
            members,
            name: self.generate_name(&value.name),
            location: self.generate_location(value.span),
            resourceness: Resourceness(false),
        }
    }

    fn generate_struct(&self, value: ast::Struct) -> ir::Struct {
        let mut members = vec![];

        for member in value.members {
            let member = member.borrow();

            members.push(ir::StructMember {
                name: self.generate_identifier(member.name.clone()),
                location: self.generate_location(member.span.clone()),
                r#type: self.generate_type(member.type_ctor.r#type.as_ref().unwrap().clone()),
                field_shape_v2: self.generate_field_shape(),
                // value: self.generate_constant(member.value.clone()),
            })
        }

        ir::Struct {
            name: self.generate_name(&value.name),
            naming_context: self.generate_naming_context(value.name),
            location: self.generate_location(value.span),
            resourceness: ir::Resourceness(false),
            maybe_attributes: None,
            is_empty_success_struct: false,
            members,
            max_handles: None,
            type_shape_v2: self.generate_type_shape(),
        }
    }

    fn generate_identifier(&self, value: ast::Span) -> ir::Identifier {
        ir::Identifier(value.data)
    }

    fn generate_enum(&self, value: ast::Enum) -> ir::Enum {
        let mut members = vec![];

        for member in value.members {
            let member = member.borrow();

            members.push(ir::EnumMember {
                name: self.generate_identifier(member.name.clone()),
                value: self.generate_constant(member.value.clone()),
            })
        }

        ir::Enum {
            name: self.generate_name(&value.name),
            location: self.generate_location(value.span),
            maybe_attributes: vec![],
            r#type: ir::PrimitiveSubtype::Uint32,
            members,
            is_strict: false,
            raw_unknown_value: None,
        }
    }

    fn generate_protocol(&self, value: ast::Protocol) -> ir::Protocol {
        let mut methods = vec![];

        for method in value.methods {
            let method = method.borrow();

            let mut maybe_request_payload = None;
            let mut maybe_response_payload = None;

            if let Some(typ) = &method.maybe_request {
                maybe_request_payload = Some(self.generate_type(typ.r#type.as_ref().unwrap().clone()));
            }

            if let Some(typ) = &method.maybe_response {
                maybe_response_payload = Some(self.generate_type(typ.r#type.as_ref().unwrap().clone()));
            }

            methods.push(ir::ProtocolMethod {
                name: self.generate_identifier(method.name.clone()),
                ordinal: 0,
                has_response: false,
                has_request: false,
                has_error: false,
                maybe_request_payload,
                maybe_response_payload,
            })
        }

        ir::Protocol {
            location: self.generate_location(value.span),
            name: self.generate_name(&value.name),
            methods,
        }
    }
}
