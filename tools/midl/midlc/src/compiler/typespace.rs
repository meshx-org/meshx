use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{ast, diagnotics::Diagnostics};

use super::type_resolver::TypeResolver;

enum TransportSide {
    Client,
    Server,
}

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

    fn create_alias_type(&self, decl: ast::Declaration) -> Option<ast::Type> {
        todo!()
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

    fn create_handle_type(&self, decl: ast::Declaration) -> Option<ast::Type> {
        todo!()
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
        todo!()
    }

    fn create_array_type(&self) -> Option<ast::Type> {
        todo!()
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
            //self.out_params_.element_type_raw = self.parameters.items[0].as_type_ctor();

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

    fn create_transport_side_type(&self, side: TransportSide) -> Option<ast::Type> {
        todo!()
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
        let target = self.layout.resolved().unwrap().element().as_decl().unwrap();

        match target {
            ast::Declaration::Bits|
            ast::Declaration::Enum{..}|
            ast::Declaration::NewType|
            ast::Declaration::Struct {..}|
            ast::Declaration::Table{..}|
            ast::Declaration::Union{..} |
            ast::Declaration::Overlay => return self.create_identifier_type(target),
            ast::Declaration::Resource{..} => return self.create_handle_type(target),
            ast::Declaration::Alias {..} => return self.create_alias_type(target),
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
                ast::BuiltinIdentity::ClientEnd => self.create_transport_side_type(TransportSide::Client),
                ast::BuiltinIdentity::ServerEnd => self.create_transport_side_type(TransportSide::Server),
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

    unbounded_string_type: Option<Rc<ast::StringType>>,
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
            unbounded_string_type,
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
        self.unbounded_string_type.as_ref().unwrap().clone()
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
