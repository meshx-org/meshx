use midlgen::ir::{self, EncodedCompoundIdentifier};
use serde_json::json;

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
        println!("compilation: {:#?}", self.compilation);

        let ir = ir::Root {
            name: ir::EncodedLibraryIdentifier("".to_string()),
            table_declarations: vec![],
            const_declarations: self.generate_const_declarations(&self.compilation.declarations.consts),
            enum_declarations: vec![],
            struct_declarations: vec![],
            protocol_declarations: vec![],
            union_declarations: vec![],
            bits_declarations: vec![],
            experiments: vec![],
            library_dependencies: vec![],
        };

        serde_json::to_value(ir).unwrap()
    }

    fn generate_const_declarations(&self, decls: &Vec<ast::Declaration>) -> Vec<ir::Const> {
        decls
            .into_iter()
            .map(|decl| {
                if let ast::Declaration::Const(const_decl) = decl {
                    self.generate_const(const_decl.borrow().clone())
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

    fn generate_name(&self, name: ast::Name) -> ir::EncodedCompoundIdentifier {
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

    fn generate_nullable(&self, value: ast::Nullability) -> bool {
        match value {
            ast::Nullability::Nullable => true,
            ast::Nullability::Nonnullable => false,
        }
    }

    fn generate_primitive_subtype(&self, value: ast::PrimitiveSubtype) -> ir::PrimitiveSubtype {
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

    fn generate_type(&self, value: ast::Type) -> ir::Type {
        match value {
            ast::Type::Vector(r#type) => ir::Type::VectorType {
                element_type: Box::from(self.generate_type(*r#type.element_type.clone())),
                element_count: Some(r#type.element_size()),
                nullable: self.generate_nullable(r#type.nullability),
            },
            ast::Type::Primitive(r#type) => ir::Type::PrimitiveType {
                primitive_subtype: self.generate_primitive_subtype(r#type.subtype),
            },
            _ => panic!("unsupported type: {:?}", value),
        }
    }

    fn generate_type_and_from_alias(&self, parent_type_kind: TypeKind, value: ast::TypeConstructor) -> ir::Type {
        let r#type = value.r#type;
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

    fn generate_constant_value(&self, value: ast::ConstantValue) -> String {
        value.to_string()
    }

    fn generate_constant(&self, value: ast::Constant) -> ir::Constant {
        match value {
            ast::Constant::Identifier(identifier) => ir::Constant::Identifier {
                value: self.generate_constant_value(identifier.value()),
                expression: identifier.span().data.clone(),
                identifier: EncodedCompoundIdentifier("".to_string()), //identifier.reference.resolved().name(),
            },
            ast::Constant::Literal(literal) => ir::Constant::LiteralConstant {
                value: self.generate_constant_value(literal.value()),
                expression: literal.span().data.clone(),
                literal: self.generate_literal(literal),
            },
        }
    }

    fn generate_const(&self, value: ast::Const) -> ir::Const {
        ir::Const {
            name: self.generate_name(value.name),
            location: self.generate_location(value.span),
            r#type: self.generate_type_and_from_alias(TypeKind::Concrete, value.type_ctor),
            value: self.generate_constant(value.value),
        }
    }
}
