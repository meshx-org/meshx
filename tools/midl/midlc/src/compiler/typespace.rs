use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{
    ast::{self, ConstantValue, Name, UntypedNumericType},
    diagnotics::Diagnostics,
};

use super::type_resolver::TypeResolver;

struct TypeCreator<'a, 'r, 'd> {
    typespace: &'a Typespace,
    resolver: &'a TypeResolver<'r, 'd>,
    layout: &'a ast::Reference,
    parameters: &'a ast::LayoutParameterList,
    constraints: &'a ast::LayoutConstraints,
}

fn builtin_to_internal_subtype(id: ast::BuiltinIdentity) -> Option<ast::InternalSubtype> {
    match id {
        ast::BuiltinIdentity::framework_err => Some(ast::InternalSubtype::FrameworkErr),
        _ => None,
    }
}

fn is_struct(typ: &ast::Type) -> bool {
    if let ast::Type::Identifier(id_type) = typ {
        if let ast::Declaration::Struct { .. } = id_type.decl {
            return true;
        }

        return false;
    } else {
        return false;
    }
}

fn cannot_be_boxed_nor_optional(typ: &ast::Type) -> bool {
    match typ {
        ast::Type::Array(_) | ast::Type::Box(_) | ast::Type::Primitive(_) => true,

        ast::Type::Identifier(id_type) => match id_type.decl {
            ast::Declaration::Enum { .. }
            | ast::Declaration::Builtin { .. }
            | ast::Declaration::Table { .. }
            | ast::Declaration::Bits { .. } => true,
            _ => false,
        },

        _ => false,
    }
}

fn builtin_to_primitive_subtype(id: ast::BuiltinIdentity) -> Option<ast::PrimitiveSubtype> {
    match id {
        ast::BuiltinIdentity::bool => Some(ast::PrimitiveSubtype::Bool),
        ast::BuiltinIdentity::int8 => Some(ast::PrimitiveSubtype::Int8),
        ast::BuiltinIdentity::int16 => Some(ast::PrimitiveSubtype::Int16),
        ast::BuiltinIdentity::int32 => Some(ast::PrimitiveSubtype::Int32),
        ast::BuiltinIdentity::int64 => Some(ast::PrimitiveSubtype::Int64),
        ast::BuiltinIdentity::uint8 => Some(ast::PrimitiveSubtype::Uint8),
        ast::BuiltinIdentity::uint16 => Some(ast::PrimitiveSubtype::Uint16),
        ast::BuiltinIdentity::uint32 => Some(ast::PrimitiveSubtype::Uint32),
        ast::BuiltinIdentity::uint64 => Some(ast::PrimitiveSubtype::Uint64),
        ast::BuiltinIdentity::float32 => Some(ast::PrimitiveSubtype::Float32),
        ast::BuiltinIdentity::float64 => Some(ast::PrimitiveSubtype::Float64),
        _ => None,
    }
}

// TODO(https://fxbug.dev/42134495): Support more transports.
const CHANNEL_TRANSPORT: &str = "Channel";

impl<'a, 'r, 'd> TypeCreator<'a, 'r, 'd> {
    fn ensure_number_of_layout_params(&self, expected_params: u32) -> bool {
        let num_params = self.parameters.items.len();
        if num_params == expected_params as usize {
            return true;
        }

        let span_data = if num_params == 0 {
            self.layout.span.as_ref().unwrap().data.clone()
        } else {
            self.parameters.span.as_ref().unwrap().data.clone()
        };

        // TODO: return reporter()->Fail(ErrWrongNumberOfLayoutParameters, span, layout_.resolved().name(),
        //                 expected_params, num_params);

        todo!();
        return false;
    }

    fn create_alias_type(&self, alias_decl: Rc<RefCell<ast::Alias>>) -> Option<ast::Type> {
        let mut as_decl = ast::Declaration::Alias {
            decl: alias_decl.clone(),
        };

        if let Some(cycle) = self.resolver.get_decl_cycle(&mut as_decl) {
            panic!("ErrIncludeCycle");
            // reporter()->Fail(ErrIncludeCycle, alias.name.span().value(), cycle.value());
            // return None;
        }

        self.resolver.compile_decl(&mut as_decl);

        if !self.ensure_number_of_layout_params(0) {
            return None;
        }

        let alias = alias_decl.borrow();

        // Compilation failed while trying to resolve something farther up the chain;
        // exit early
        if let None = alias.partial_type_ctor.r#type {
            return None;
        }

        let aliased_type = alias.partial_type_ctor.r#type.as_ref().unwrap();
        //out_params_.from_alias = alias;

        let constrained_type = aliased_type
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
                // out_params_,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create_identifier_type(&self, mut decl: ast::Declaration) -> Option<ast::Type> {
        if !decl.compiled() && !matches!(decl, ast::Declaration::Protocol { .. }) {
            if decl.compiling() {
                decl.set_recursive(true);
            } else {
                self.resolver.compile_decl(&mut decl);
            }
        }

        if !self.ensure_number_of_layout_params(0) {
            return None;
        }

        let r#type = ast::IdentifierType::new(decl);

        let constrained_type = r#type
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
                // out_params_,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create_handle_type(&self, resource: Rc<RefCell<ast::Resource>>) -> Option<ast::Type> {
        let mut resource_decl = ast::Declaration::Resource { decl: resource.clone() };

        if !self.ensure_number_of_layout_params(0) {
            return None;
        }

        if let Some(cycl) = self.resolver.get_decl_cycle(&mut resource_decl) {
            panic!("ErrIncludeCycle");
            //reporter()->Fail(ErrIncludeCycle, resource->name.span().value(), cycle.value());
            return None;
        }

        self.resolver.compile_decl(&mut resource_decl);

        let r#type = ast::HandleType::new(self.layout.resolved().unwrap().name(), resource);

        let constrained_type = r#type
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create_primitive_type(&self, subtype: ast::PrimitiveSubtype) -> Option<ast::Type> {
        if !self.ensure_number_of_layout_params(0) {
            return None;
        }

        let constrained_type = self
            .typespace
            .get_primitive_type(subtype)
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
                // out_params_,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create_box_type(&self) -> Option<ast::Type> {
        if !self.ensure_number_of_layout_params(1) {
            return None;
        }

        let boxed_type;

        if let Ok(boxed) = self
            .resolver
            .resolve_param_as_type(self.layout, &self.parameters.items[0])
        {
            boxed_type = boxed;
        } else {
            return None;
        }

        if !is_struct(&boxed_type) {
            if cannot_be_boxed_nor_optional(&boxed_type) {
                panic!("ErrCannotBeBoxedNorOptional");
                // reporter()->Fail(ErrCannotBeBoxedNorOptional, parameters_.items[0]->span, boxed_type->name);
            } else {
                panic!("ErrCannotBeBoxedShouldBeOptional");
                // reporter()->Fail(ErrCannotBeBoxedShouldBeOptional, parameters_.items[0]->span, boxed_type->name);
            }

            return None;
        }

        if let ast::Type::Identifier(inner) = boxed_type.clone() {
            assert!(
                matches!(inner.constraints.nullabilty(), ast::Nullability::Nonnullable),
                "the inner type must be non-nullable because it is a struct"
            );

            // We disallow specifying the boxed type as nullable in FIDL source but
            // then mark the boxed type as nullable, so that internally it shares the
            // same code path as its old syntax equivalent (a nullable struct). This
            // allows us to call `f(type->boxed_type)` wherever we used to call `f(type)`
            // in the old code.
            // As a temporary workaround for piping unconst-ness everywhere or having
            // box types own their own boxed types, we cast away the const to be able
            // to change the boxed type to be mutable.
            inner.constraints.set_nullabilty(ast::Nullability::Nullable);
        }

        // self.out_params.boxed_type_resolved = boxed_type;
        // self.out_params.boxed_type_raw = self.parameters_.items[0].as_type_ctor();

        let r#type = ast::BoxType::new(self.layout.resolved().unwrap().name(), boxed_type);

        let constrained_type = r#type
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create_array_type(&self) -> Option<ast::Type> {
        if !self.ensure_number_of_layout_params(2) {
            return None;
        }

        let element_type;
        if let Ok(param_type) = self
            .resolver
            .resolve_param_as_type(self.layout, &self.parameters.items[0])
        {
            element_type = param_type;
        } else {
            return None;
        }

        // out_params_.element_type_resolved = element_type;
        // out_params_.element_type_raw = parameters_.items[0]->AsTypeCtor();

        let size;
        if let Ok(param_size) = self
            .resolver
            .resolve_param_as_size(self.layout, &self.parameters.items[1])
        {
            size = param_size;
        } else {
            return None;
        }

        // out_params_->size_resolved = size;
        // out_params_->size_raw = parameters_.items[1]->AsConstant();

        let r#type = ast::ArrayType::new(self.layout.resolved().unwrap().name(), element_type, size);
        let constrained_type = r#type
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create_vector_type(&self) -> Option<ast::Type> {
        if !self.ensure_number_of_layout_params(1) {
            return None;
        }

        if let Ok(typ) = self
            .resolver
            .resolve_param_as_type(self.layout, self.parameters.items.get(0).unwrap())
        {
            //self.out_params.element_type_resolved = element_type;
            //self.out_params.element_type_raw = self.parameters.items[0].as_type_ctor();

            let r#type = ast::VectorType::new(self.layout.resolved().unwrap().name(), typ);
            let constrained_type = r#type
                .apply_constraints(
                    self.resolver,
                    self.typespace.diagnostics.clone(),
                    self.constraints,
                    self.layout,
                )
                .unwrap();

            self.typespace.intern(constrained_type)
        } else {
            return None;
        }
    }

    fn create_string_type(&self) -> Option<ast::Type> {
        if !self.ensure_number_of_layout_params(0) {
            return None;
        }

        let r#type = ast::StringType::new(self.layout.resolved().unwrap().name());
        let constrained_type = r#type
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create_transport_side_type(&self, end: ast::TransportSide) -> Option<ast::Type> {
        if !self.ensure_number_of_layout_params(0) {
            return None;
        }

        let r#type = ast::TransportSideType::new(self.layout.resolved().unwrap().name(), end, CHANNEL_TRANSPORT);
        let constrained_type = r#type
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create_internal_type(&self, subtype: ast::InternalSubtype) -> Option<ast::Type> {
        if !self.ensure_number_of_layout_params(0) {
            return None;
        }

        let constrained_type = self
            .typespace
            .get_internal_type(subtype)
            .apply_constraints(
                self.resolver,
                self.typespace.diagnostics.clone(),
                self.constraints,
                self.layout,
            )
            .unwrap();

        self.typespace.intern(constrained_type)
    }

    fn create(&self) -> Option<ast::Type> {
        log::warn!("lay: {:?}", self.layout);
        let target = self.layout.resolved().unwrap().element().as_decl().unwrap();

        match target {
            ast::Declaration::Bits{ .. }|
            ast::Declaration::Enum{..}|
            ast::Declaration::NewType|
            ast::Declaration::Struct {..}|
            ast::Declaration::Table{..}|
            ast::Declaration::Union{..} |
            ast::Declaration::Overlay => return self.create_identifier_type(target),
            ast::Declaration::Resource{ decl: resource } => return self.create_handle_type(resource),
            ast::Declaration::Alias { decl } => return self.create_alias_type(decl),
            ast::Declaration::Builtin {..} => {
                // Handled below.
            },
            ast::Declaration::Const{..} | ast::Declaration::Protocol{..} /* |ast::Declaration::Service */=> {
                //TODO: self.typespace.diagnostics.push_error(ErrExpectedType, layout_.span());
                return None;
            }
        };

        if let ast::Declaration::Builtin { decl } = target {
            let builtin = decl.borrow();

            match builtin.id {
                ast::BuiltinIdentity::bool
                | ast::BuiltinIdentity::int8
                | ast::BuiltinIdentity::int16
                | ast::BuiltinIdentity::int32
                | ast::BuiltinIdentity::int64
                | ast::BuiltinIdentity::uint8
                | ast::BuiltinIdentity::uint16
                | ast::BuiltinIdentity::uint32
                | ast::BuiltinIdentity::uint64
                | ast::BuiltinIdentity::float32
                | ast::BuiltinIdentity::float64 => {
                    let subtype = builtin_to_primitive_subtype(builtin.id.clone()).unwrap();
                    self.create_primitive_type(subtype)
                }
                ast::BuiltinIdentity::StringArray => None,
                ast::BuiltinIdentity::String => self.create_string_type(),
                ast::BuiltinIdentity::Array => self.create_array_type(),
                ast::BuiltinIdentity::Vector => self.create_vector_type(),
                ast::BuiltinIdentity::Box => self.create_box_type(),
                ast::BuiltinIdentity::ClientEnd => self.create_transport_side_type(ast::TransportSide::Client),
                ast::BuiltinIdentity::ServerEnd => self.create_transport_side_type(ast::TransportSide::Server),
                ast::BuiltinIdentity::Byte => self.create_primitive_type(ast::PrimitiveSubtype::Uint8),
                ast::BuiltinIdentity::framework_err => {
                    let subtype = builtin_to_internal_subtype(builtin.id.clone()).unwrap();
                    self.create_internal_type(subtype)
                }
                ast::BuiltinIdentity::Optional | ast::BuiltinIdentity::MAX | ast::BuiltinIdentity::HEAD => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct Typespace {
    root_library: Rc<ast::Library>,
    pub(crate) diagnostics: Rc<Diagnostics>,
    types: RefCell<Vec<ast::Type>>,

    unbounded_string_type: Rc<ast::StringType>,
    untyped_numeric_type: Rc<ast::UntypedNumericType>,
    primitive_types: BTreeMap<ast::PrimitiveSubtype, Rc<ast::PrimitiveType>>,
    internal_types: BTreeMap<ast::InternalSubtype, Rc<ast::InternalType>>,
}

impl Typespace {
    pub fn new(root_library: Rc<ast::Library>, diagnostics: Rc<Diagnostics>) -> Self {
        let mut primitive_types = BTreeMap::new();
        let mut internal_types = BTreeMap::new();
        let mut unbounded_string_type = None;

        for builtin in root_library.declarations.borrow().builtins.iter() {
            if let ast::Declaration::Builtin { decl } = builtin {
                let builtin = decl.borrow();

                if let Some(subtype) = builtin_to_primitive_subtype(builtin.id) {
                    primitive_types.insert(
                        subtype,
                        Rc::from(ast::PrimitiveType {
                            name: builtin.name.clone(),
                            subtype,
                        }),
                    );
                } else if let Some(subtype) = builtin_to_internal_subtype(builtin.id) {
                    internal_types.insert(
                        subtype,
                        Rc::from(ast::InternalType {
                            name: builtin.name.clone(),
                            subtype,
                        }),
                    );
                } else if builtin.id == ast::BuiltinIdentity::String {
                    unbounded_string_type = Some(Rc::from(ast::StringType::new(builtin.name.clone())));
                }
            }

            /* else if (builtin->id == Builtin::Identity::kVector) {
                vector_layout_name_ = builtin->name;
            } else if (builtin->id == Builtin::Identity::kZxExperimentalPointer) {
                pointer_type_name_ = builtin->name;
            }*/
        }

        Self {
            root_library,
            diagnostics,
            types: RefCell::from(vec![]),
            primitive_types,
            internal_types,
            unbounded_string_type: unbounded_string_type.unwrap(),
            untyped_numeric_type: Rc::new(UntypedNumericType::new(Name::create_intrinsic(None, "untyped numeric"))),
        }
    }

    fn intern(&self, typ: ast::Type) -> Option<ast::Type> {
        let mut types = self.types.borrow_mut();
        types.push(typ);
        types.last().cloned()
    }

    pub fn get_primitive_type(&self, subtype: ast::PrimitiveSubtype) -> Rc<ast::PrimitiveType> {
        self.primitive_types
            .get(&subtype)
            .cloned()
            .expect("all primitive subtypes should be inserted")
    }

    pub fn get_internal_type(&self, subtype: ast::InternalSubtype) -> Rc<ast::InternalType> {
        self.internal_types
            .get(&subtype)
            .cloned()
            .expect("all internal subtypes should be inserted")
    }

    pub fn get_unbounded_string_type(&self) -> Rc<ast::StringType> {
        self.unbounded_string_type.clone()
    }

    pub fn get_string_type(&self, max_size: usize) -> Rc<ast::StringType> {
        //self.sizes.push_back(std::make_unique<SizeValue>(max_size));
        //let size = self.sizes_.back().get();

        let r#type = Rc::new(ast::StringType::new_with_constraints(
            self.unbounded_string_type.name.clone(),
            ast::VectorConstraints::new(
                Some(ConstantValue::Uint32(max_size as u32)),
                ast::Nullability::Nonnullable,
            ),
        ));

        self.intern(ast::Type::String(r#type.clone()));
        r#type
    }

    pub fn get_untyped_numeric_type(&self) -> Rc<ast::UntypedNumericType> {
        self.untyped_numeric_type.clone()
    }

    pub fn create(
        &self,
        resolver: &TypeResolver<'_, '_>,
        layout: &ast::Reference,
        parameters: &ast::LayoutParameterList,
        constraints: &ast::LayoutConstraints,
    ) -> Option<ast::Type> {
        // TODO(https://fxbug.dev/76219): lookup whether we've already created the type, and
        // return it rather than create a new one. Lookup must be by name, arg_type,
        // size, and nullability.
        TypeCreator {
            typespace: self,
            resolver,
            layout,
            parameters,
            constraints,
        }
        .create()
    }
}
