use std::collections::HashMap;
use std::collections::HashSet;

use crate::types;
use crate::types::Payload;
use crate::types::Root;
use convert_case::Casing;
use midlgen::ir;

struct Compiler {
    root: Root,
    decls: ir::DeclInfoMap,
    library: ir::LibraryIdentifier,
    structs: HashMap<ir::EncodedCompoundIdentifier, ir::Struct>,
    // types_root: midlgen::Root,
    // paramable_types: HashMap<midlgen::EncodedCompoundIdentifier, Parameterizer>
}

lazy_static::lazy_static! {
    static ref PRIMITIVE_SUBTYPES: HashMap<ir::PrimitiveSubtype, String> = {
        let mut m = HashMap::new();
        m.insert(ir::PrimitiveSubtype::Bool, "boolean".to_string());
        m.insert(ir::PrimitiveSubtype::Uint8, "number".to_string());
        m.insert(ir::PrimitiveSubtype::Uint16, "number".to_string());
        m.insert(ir::PrimitiveSubtype::Uint32, "number".to_string());
        m.insert(ir::PrimitiveSubtype::Uint64, "bigint".to_string());
        m.insert(ir::PrimitiveSubtype::Int8, "number".to_string());
        m.insert(ir::PrimitiveSubtype::Int16, "number".to_string());
        m.insert(ir::PrimitiveSubtype::Int32, "number".to_string());
        m.insert(ir::PrimitiveSubtype::Int64, "bigint".to_string());
        m.insert(ir::PrimitiveSubtype::Float32, "number".to_string());
        m.insert(ir::PrimitiveSubtype::Float64, "number".to_string());
        m
    };

    static ref VALUE_TYPES: HashMap<ir::PrimitiveSubtype, String> = {
        let mut m = HashMap::new();
        m.insert(ir::PrimitiveSubtype::Bool, "midl.BoolType".to_string());
        m.insert(ir::PrimitiveSubtype::Uint8, "midl.UInt8Type".to_string());
        m.insert(ir::PrimitiveSubtype::Uint16, "midl.UInt16Type".to_string());
        m.insert(ir::PrimitiveSubtype::Uint32, "midl.UInt32Type".to_string());
        m.insert(ir::PrimitiveSubtype::Uint64, "midl.UInt64Type".to_string());
        m.insert(ir::PrimitiveSubtype::Int8, "midl.Int8Type".to_string());
        m.insert(ir::PrimitiveSubtype::Int16, "midl.Int16Type".to_string());
        m.insert(ir::PrimitiveSubtype::Int32, "midl.Int32Type".to_string());
        m.insert(ir::PrimitiveSubtype::Int64, "midl.Int64Type".to_string());
        m.insert(ir::PrimitiveSubtype::Float32, "midl.Float32Type".to_string());
        m.insert(ir::PrimitiveSubtype::Float64, "midl.Float64Type".to_string());
        m
    };
}

lazy_static::lazy_static! {
    static ref RESERVED_WORDS: HashSet<String> = {
        let mut m = HashSet::new();
        m.insert("break".to_string());
        m.insert("const".to_string());
        m.insert("let".to_string());
        m.insert("var".to_string());
        m.insert("throw".to_string());
        m.insert("default".to_string());
        m.insert("case".to_string());
        m.insert("delete".to_string());
        m.insert("await".to_string());
        m.insert("async".to_string());
        m.insert("import".to_string());
        m.insert("export".to_string());
        m.insert("this".to_string());
        m.insert("void".to_string());
        m.insert("yield".to_string());
        m.insert("enum".to_string());
        m.insert("abstract".to_string());
        m.insert("try".to_string());
        m.insert("catch".to_string());
        m.insert("class".to_string());
        m.insert("function".to_string());
        m.insert("new".to_string());
        m.insert("interface".to_string());
        m.insert("type".to_string());
        m.insert("static".to_string());
        m.insert("protected".to_string());
        m.insert("public".to_string());
        m.insert("if".to_string());
        m.insert("else".to_string());
        m.insert("return".to_string());
        m.insert("true".to_string());
        m.insert("false".to_string());
        m.insert("in".to_string());
        m.insert("null".to_string());
        m.insert("undefined".to_string());
        m.insert("super".to_string());
        m.insert("extends".to_string());
        m.insert("implements".to_string());
        m.insert("instanceof".to_string());
        m.insert("keyof".to_string());
        m.insert("typeof".to_string());
        m.insert("with".to_string());
        m.insert("while".to_string());
        m.insert("finally".to_string());
        m.insert("satisfies".to_string());
        m.insert("debugger".to_string());
        m.insert("continue".to_string());

        // Built-ins
        m.insert("name".to_string());
        m.insert("String".to_string());
        m.insert("Number".to_string());
        m.insert("BigInt".to_string());
        m.insert("Infinity".to_string());
        m.insert("Date".to_string());
        m.insert("Object".to_string());
        m.insert("isFinite".to_string());
        m.insert("NaN".to_string());

        // Others
        m.insert("window".to_string());
        m.insert("frame".to_string());
        m.insert("constructor".to_string());
        m.insert("document".to_string());
        m.insert("global".to_string());
        m.insert("arguments".to_string());
        m.insert("eval".to_string());
        m.insert("of".to_string());
        m.insert("set".to_string());
        m.insert("get".to_string());
        m.insert("as".to_string());

        // Keywords reserved for future use (future-proofing...)
        m.insert("long".to_string());
        m.insert("double".to_string());
        m.insert("short".to_string());
        m.insert("native".to_string());
        m.insert("transient".to_string());
        m.insert("volatile".to_string());
        m.insert("throws".to_string());
        m.insert("byte".to_string());
        m.insert("char".to_string());
        m.insert("goto".to_string());
        m.insert("float".to_string());
        m.insert("eval".to_string());
        m.insert("package".to_string());
        m.insert("synchronized".to_string());

        m
    };
}

lazy_static::lazy_static! {
    static ref HANDLE_SUBTYPES: HashMap<midlgen::ir::HandleSubtype, String> = {
        let mut m = HashMap::new();
        m.insert(midlgen::ir::HandleSubtype::None, "fiber.Handle".to_string());
         m.insert(midlgen::ir::HandleSubtype::Channel, "fiber.Channel".to_string());
        m.insert(midlgen::ir::HandleSubtype::Event, "fiber.Event".to_string());
        m.insert(midlgen::ir::HandleSubtype::EventPair, "fiber.EventPair".to_string());
        m.insert(midlgen::ir::HandleSubtype::Exception, "fiber.Exception".to_string());
        m.insert(midlgen::ir::HandleSubtype::Guest, "fiber.Guest".to_string());
        m.insert(midlgen::ir::HandleSubtype::Fifo, "fiber.Fifo".to_string());
        m.insert(midlgen::ir::HandleSubtype::Bti, "fiber.Bti".to_string());
        m.insert(midlgen::ir::HandleSubtype::Clock, "fiber.Clock".to_string());
        m.insert(midlgen::ir::HandleSubtype::Debuglog, "fiber.Debuglog".to_string());
        m.insert(midlgen::ir::HandleSubtype::Interrupt, "fiber.Interrupt".to_string());
        m.insert(midlgen::ir::HandleSubtype::Iommu, "fiber.Iommu".to_string());
        m.insert(midlgen::ir::HandleSubtype::Job, "fiber.Job".to_string());
        m.insert(midlgen::ir::HandleSubtype::Msi, "fiber.Msi".to_string());
        m.insert(midlgen::ir::HandleSubtype::Pager, "fiber.Pager".to_string());
        m.insert(midlgen::ir::HandleSubtype::PciDevice, "fiber.PciDevice".to_string());
        m.insert(midlgen::ir::HandleSubtype::Pmt, "fiber.Pmt".to_string());
        m.insert(midlgen::ir::HandleSubtype::Port, "fiber.Port".to_string());
        m.insert(midlgen::ir::HandleSubtype::Process, "fiber.Process".to_string());
        m.insert(midlgen::ir::HandleSubtype::Profile, "fiber.Profile".to_string());
        m.insert(midlgen::ir::HandleSubtype::Resource, "fiber.Resource".to_string());
        m.insert(midlgen::ir::HandleSubtype::Socket, "fiber.Socket".to_string());
        m.insert(midlgen::ir::HandleSubtype::Stream, "fiber.Stream".to_string());
        m.insert(midlgen::ir::HandleSubtype::SuspendToken, "fiber.SuspendToken".to_string());
        m.insert(midlgen::ir::HandleSubtype::Thread, "fiber.Thread".to_string());
        m.insert(midlgen::ir::HandleSubtype::Timer, "fiber.Timer".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vcpu, "fiber.Vcpu".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vmar, "fiber.Vmar".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vmo, "fiber.Vmo".to_string());

        m
    };

    static ref OBJECT_TYPES: HashMap<midlgen::ir::HandleSubtype, String> = {
        let mut m = HashMap::new();
        m.insert(midlgen::ir::HandleSubtype::None, "FX_OBJECT_TYPE_NONE".to_string());
        m.insert(midlgen::ir::HandleSubtype::Channel, "FX_OBJECT_TYPE_CHANNEL".to_string());
        m.insert(midlgen::ir::HandleSubtype::Event, "FX_OBJECT_TYPE_EVENT".to_string());
        m.insert(midlgen::ir::HandleSubtype::EventPair, "FX_OBJECT_TYPE_EVENTPAIR".to_string());
        m.insert(midlgen::ir::HandleSubtype::Exception, "midl.ObjectType.EXCEPTION".to_string());
        m.insert(midlgen::ir::HandleSubtype::Guest, "midl.ObjectType.GUEST".to_string());
        m.insert(midlgen::ir::HandleSubtype::Fifo, "midl.ObjectType.FIFO".to_string());
        m.insert(midlgen::ir::HandleSubtype::Bti, "midl.ObjectType.BTI".to_string());
        m.insert(midlgen::ir::HandleSubtype::Clock, "midl.ObjectType.CLOCK".to_string());
        m.insert(midlgen::ir::HandleSubtype::Debuglog, "midl.ObjectType.DEBUGLOG".to_string());
        m.insert(midlgen::ir::HandleSubtype::Interrupt, "midl.ObjectType.INTERRUPT".to_string());
        m.insert(midlgen::ir::HandleSubtype::Iommu, "midl.ObjectType.IOMMU".to_string());
        m.insert(midlgen::ir::HandleSubtype::Job, "midl.ObjectType.JOB".to_string());
        m.insert(midlgen::ir::HandleSubtype::Msi, "midl.ObjectType.MSI".to_string());
        m.insert(midlgen::ir::HandleSubtype::Pager, "midl.ObjectType.PAGER".to_string());
        m.insert(midlgen::ir::HandleSubtype::PciDevice, "midl.ObjectType.PCIDEVICE".to_string());
        m.insert(midlgen::ir::HandleSubtype::Pmt, "midl.ObjectType.PMT".to_string());
        m.insert(midlgen::ir::HandleSubtype::Port, "midl.ObjectType.PORT".to_string());
        m.insert(midlgen::ir::HandleSubtype::Process, "midl.ObjectType.PROCESS".to_string());
        m.insert(midlgen::ir::HandleSubtype::Profile, "midl.ObjectType.PROFILE".to_string());
        m.insert(midlgen::ir::HandleSubtype::Resource, "midl.ObjectType.RESOURCE".to_string());
        m.insert(midlgen::ir::HandleSubtype::Socket, "FX_OBJECT_TYPE_SOCKET".to_string());
        m.insert(midlgen::ir::HandleSubtype::Stream, "midl.ObjectType.STREAM".to_string());
        m.insert(midlgen::ir::HandleSubtype::SuspendToken, "midl.ObjectType.SUSPENDTOKEN".to_string());
        m.insert(midlgen::ir::HandleSubtype::Thread, "midl.ObjectType.THREAD".to_string());
        m.insert(midlgen::ir::HandleSubtype::Timer, "midl.ObjectType.TIMER".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vcpu, "midl.ObjectType.VCPU".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vmar, "midl.ObjectType.VMAR".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vmo, "midl.ObjectType.VMO".to_string());

        m
    };
}

lazy_static::lazy_static! {
    static ref RESERVED_SUFFIX: Vec<String> = {
        let mut v = Vec::new();
        v.push("Impl".to_string());
        v.push("Marker".to_string());
        v.push("Proxy".to_string());
        v.push("ProxyProtocol".to_string());
        v.push("ControlHandle".to_string());
        v.push("Responder".to_string());
        v.push("Server".to_string());
        v
    };
}

// hasReservedSuffix checks if a string ends with a suffix commonly used by the
// bindings in generated types
fn has_reserved_suffix(str: &String) -> bool {
    for suffix in RESERVED_SUFFIX.iter() {
        if str.starts_with(suffix) {
            return true;
        }
    }

    false
}

impl Compiler {
    fn lookup_decl_info(&self, val: &midlgen::ir::EncodedCompoundIdentifier) -> &midlgen::ir::DeclInfo {
        let info = self.decls.get(val);
        info.expect(format!("identifier not in decl map: {:?}", val).as_str())
    }

    /// changeIfReserved adds an underscore suffix to differentiate an identifier
    /// from a reserved name
    ///
    /// Reserved names include a variety of rust keywords, commonly used rust types
    /// like Result, Vec, and Future, and any name ending in a suffix used by the
    /// bindings to identify particular generated types like -Impl, -Marker, and
    /// -Proxy.
    fn change_if_reserved(&self, str: String) -> String {
        if has_reserved_suffix(&str) || RESERVED_WORDS.contains(&str) {
            return str + "_";
        }

        str
    }

    fn compile_snake_identifier(&self, val: &ir::Identifier) -> String {
        self.change_if_reserved(val.0.to_case(convert_case::Case::Snake))
    }

    fn compile_camel_identifier(&self, val: &ir::Identifier) -> String {
        self.change_if_reserved(val.0.to_case(convert_case::Case::UpperCamel))
    }

    fn compile_screaming_snake_identifier(&self, val: &ir::Identifier) -> String {
        self.change_if_reserved(val.0.to_case(convert_case::Case::ScreamingSnake))
    }

    fn compile_primitive_subtype(&self, val: &ir::PrimitiveSubtype) -> (String, String) {
        (
            PRIMITIVE_SUBTYPES
                .get(val)
                .expect(format!("unknown primitive type: {:?}", val).as_str())
                .clone(),
            VALUE_TYPES
                .get(val)
                .expect(format!("unknown primitive type: {:?}", val).as_str())
                .clone(),
        )
    }

    fn compile_decl_identifier(&self, val: &ir::EncodedCompoundIdentifier) -> String {
        let ci = val.parse();

        if ci.member.is_some() {
            panic!("unexpected member: {:?}", val);
        }

        let mut name: String;

        if let midlgen::ir::DeclType::ConstDecl = self.lookup_decl_info(val).r#type {
            name = self.compile_screaming_snake_identifier(&ci.name);
        } else {
            name = self.compile_camel_identifier(&ci.name);
        }

        // TODO(https://fxbug.dev/66767): This is incorrect. We're calling changeIfReserved
        // a second time, after compileScreamingSnakeIdentifier or
        // compileCamelIdentifier already did. We should only call it once.
        name = self.change_if_reserved(name);

        // if self.in_external_library(val) {
        //	let r#crate = compile_library_name(ci.library);
        //	extern_crates[crate] = struct{}{}
        //	return fmt.Sprintf("%s::%s", crate, name)
        // }

        name
    }

    // TODO(https://fxbug.dev/7660): Store this on types directly.
    fn lookup_resourceness(&self, t: &ir::Type) -> ir::Resourceness {
        match t {
            midlgen::ir::Type::PrimitiveType { .. }
            | midlgen::ir::Type::StringType { .. }
            | midlgen::ir::Type::InternalType { .. } => ir::VALUE_TYPE,
            midlgen::ir::Type::HandleType { .. }
            | midlgen::ir::Type::ServerEnd { .. }
            | midlgen::ir::Type::ClientEnd { .. } => ir::RESOURCE_TYPE,
            midlgen::ir::Type::ArrayType { element_type, .. }
            | midlgen::ir::Type::StringArray { element_type, .. }
            | midlgen::ir::Type::VectorType { element_type, .. } => self.lookup_resourceness(element_type.as_ref()),
            midlgen::ir::Type::IdentifierType { identifier, .. } => {
                if let Some(info) = self.decls.get(identifier) {
                    match info.r#type {
                        midlgen::ir::DeclType::BitsDecl | midlgen::ir::DeclType::EnumDecl => ir::VALUE_TYPE,
                        midlgen::ir::DeclType::ProtocolDecl => ir::RESOURCE_TYPE,
                        midlgen::ir::DeclType::StructDecl
                        | midlgen::ir::DeclType::TableDecl
                        | midlgen::ir::DeclType::UnionDecl
                        | midlgen::ir::DeclType::OverlayDecl => info.resource.clone().unwrap(),
                        _ => panic!("unexpected decl type: {:?}", info.r#type),
                    }
                } else {
                    panic!("identifier not found: {:?}", identifier)
                }
            }

            _ => panic!("unknown type kind: {:?}", t),
        }
    }

    fn compile_object_type_const(&self, val: &ir::HandleSubtype) -> String {
        OBJECT_TYPES
            .get(val)
            .expect(format!("unknown handle type: {:?}", val).as_str())
            .clone()
    }

    fn compile_handle_subtype(&self, val: &ir::HandleSubtype) -> String {
        HANDLE_SUBTYPES
            .get(val)
            .expect(format!("unknown handle type: {:?}", val).as_str())
            .clone()
    }

    fn compile_type(&self, val: &midlgen::ir::Type) -> types::Type {
        let resourceness = self.lookup_resourceness(val);

        let mut t = types::Type {
            resourceness,
            kind: types::TypeKind::StringType,
            nullable: false,
            primitive_subtype: None,
            identifier: None,
            element_type: None,
            decl_type: None,
            midl: String::new(),
            ctor: String::new(),
            value_type: String::new(),
            param: String::new(),
        };

        match val {
            midlgen::ir::Type::PrimitiveType { primitive_subtype, .. } => {
                let (prim, midl) = self.compile_primitive_subtype(primitive_subtype);
                t.midl = midl.clone();
                t.value_type = prim.clone();
                t.param = prim;
                t.ctor = format!("new {}()", midl);
                t.primitive_subtype = Some(primitive_subtype.clone());
            }
            midlgen::ir::Type::InternalType { internal_subtype, .. } => match internal_subtype {
                midlgen::ir::InternalSubtype::FrameworkErr => {
                    let s = "midl.FrameworkErr".to_string();
                    t.midl = s.clone();
                    t.param = s;
                }
                _ => panic!("unknown internal subtype"),
            },
            midlgen::ir::Type::ArrayType {
                element_type,
                element_count,
                ..
            }
            | midlgen::ir::Type::StringArray {
                element_type,
                element_count,
                ..
            } => {
                let el = self.compile_type(element_type);
                t.element_type = Some(Box::from(el.clone()));
                t.midl = format!("midl.ArrayType<{}, {}>", el.midl, element_count);

                if resourceness.is_resource_type() {
                    t.param = t.value_type.clone();
                } else {
                    t.param = format!("&{}", t.value_type);
                }
            }
            midlgen::ir::Type::VectorType {
                element_type,
                element_count,
                nullable,
                ..
            } => {
                let el = self.compile_type(&element_type);
                t.element_type = Some(Box::from(el.clone()));

                if element_count.is_none() {
                    t.midl = format!("midl.UnboundedVectorType<{}>", el.midl);
                } else {
                    t.midl = format!("midl.VectorType<{}, {}>", el.midl, element_count.expect("to be some"));
                }

                t.param = format!("{}[]", el.value_type);

                if *nullable {
                    t.midl = format!("midl.OptionalType<{}>", t.midl);
                    t.ctor = format!("new {}({})", t.midl, t.ctor);
                    t.param = format!("{} | null", t.param);
                }
            }
            midlgen::ir::Type::StringType {
                element_count,
                nullable,
                ..
            } => {
                if element_count.is_none() {
                    t.midl = "midl.StringType".to_string();
                } else {
                    let element_count = element_count.unwrap();
                    t.midl = format!("midl.StringType<{element_count}>");
                }

                t.param = "string".to_string();
                t.nullable = *nullable;
                t.ctor = format!("new {}()", &t.midl);

                if *nullable {
                    t.midl = format!("midl.OptionalType<{}>", t.midl);
                    t.ctor = format!("new {}({})", t.midl, t.ctor);
                    t.param = format!("{} | null", t.param);
                }
            }
            midlgen::ir::Type::HandleType {
                handle_subtype,
                nullable,
                handle_rights,
                ..
            } => {
                let s = self.compile_handle_subtype(handle_subtype);
                let obj_type = self.compile_object_type_const(handle_subtype);

                t.nullable = *nullable;
                t.midl = format!("midl.HandleType");
                t.ctor = format!("new {}({}, {})", t.midl, obj_type, handle_rights.bits());
                t.param = s;

                if *nullable {
                    t.midl = format!("midl.OptionalType<{}>", t.midl);
                    t.ctor = format!("new {}({})", t.midl, t.ctor);
                    t.param = format!("{} | null", t.param);
                }
            }
            midlgen::ir::Type::ClientEnd {
                identifier, nullable, ..
            } => {
                let s = format!("{}Marker", self.compile_decl_identifier(identifier));
                t.midl = format!("midl.InterfaceHandleType<{}>", s.clone());
                t.ctor = format!("new {}()", t.midl);
                t.param = s.clone();

                if *nullable {
                    t.midl = format!("midl.NullableInterfaceHandleType<{}>", s);
                    t.ctor = format!("new {}()", t.midl);
                    t.param = format!("{} | null", t.param);
                }
            }
            midlgen::ir::Type::ServerEnd { subtype, nullable, .. } => {
                let s = format!("{}Marker", self.compile_decl_identifier(subtype));
                t.nullable = *nullable;
                t.midl = format!("midl.InterfaceRequestType<{}>", s.clone());
                t.ctor = format!("new {}()", t.midl);
                t.param = s.clone();

                if *nullable {
                    t.midl = format!("midl.NullableInterfaceRequestType<{}>", s.clone());
                    t.ctor = format!("new {}()", t.midl);
                    t.param = format!("{} | null", t.param);
                }
            }
            midlgen::ir::Type::IdentifierType {
                identifier, nullable, ..
            } => {
                let name = self.compile_decl_identifier(identifier);
                let decl_info = self.lookup_decl_info(identifier);
                t.decl_type = Some(decl_info.r#type.clone());
                t.identifier = Some(identifier.clone());
                t.nullable = *nullable;

                match decl_info.r#type {
                    midlgen::ir::DeclType::EnumDecl => {
                        t.midl = format!("midl.EnumType<{}>", name.clone());
                        t.ctor = format!("_{}Type", name.clone());
                        t.param = name;
                    }
                    midlgen::ir::DeclType::BitsDecl => {
                        t.midl = format!("midl.BitsType<{}>", name.clone());
                        t.ctor = format!("_{}Type", name.clone());
                        t.param = name;
                    }
                    midlgen::ir::DeclType::StructDecl => {
                        t.midl = format!("midl.StructType<{}>", name.clone());
                        t.ctor = format!("_{}Type", name.clone());
                        t.param = name;
                    }
                    midlgen::ir::DeclType::TableDecl => {
                        t.midl = format!("midl.TableType<{}>", name.clone());
                        t.ctor = format!("_{}Type", name.clone());
                        t.param = name;
                    }
                    midlgen::ir::DeclType::UnionDecl => {
                        t.midl = format!("midl.UnionType<{}>", name.clone());
                        t.ctor = format!("_{}Type", name.clone());
                        t.param = name;
                    }
                    midlgen::ir::DeclType::ProtocolDecl => {
                        let s = format!("midl.ClientEnd<{}Marker>", name);
                        t.midl = format!("midl.Endpoint<{}>", s.clone());
                        t.ctor = format!("new {}()", t.midl);
                        t.param = s;

                        if *nullable {
                            t.midl = format!("midl.OptionalType<{}>", t.midl);
                            t.ctor = format!("new {}({})", t.midl, t.ctor);
                            t.param = format!("Option<{}>", t.param);
                        }
                    }
                    _ => panic!("unexpected type: {:?}", decl_info.r#type),
                };
            }
            _ => panic!("unknown type kind: {:?}", val),
        };

        t
    }

    fn compile_literal(&self, val: &midlgen::ir::Literal, typ: &midlgen::ir::Type) -> String {
        match val {
            midlgen::ir::Literal::StringLiteral { value } => {
                let mut b = String::with_capacity(value.len());

                for c in value.chars() {
                    match c {
                        '\\' => b.push('\\'),
                        '"' => b.push('\"'),
                        '\n' => b.push('\n'),
                        '\r' => b.push('\r'),
                        '\t' => b.push('\t'),
                        _ => {
                            if c.is_ascii_graphic() {
                                b.push(c);
                            } else {
                                b.push_str(format!("\u{c}").as_str());
                            }
                        }
                    }
                }

                b
            }
            midlgen::ir::Literal::NumericLiteral { value } => {
                if let midlgen::ir::Type::PrimitiveType { primitive_subtype, .. } = typ {
                    if *primitive_subtype == midlgen::ir::PrimitiveSubtype::Uint64
                        || *primitive_subtype == midlgen::ir::PrimitiveSubtype::Int64
                    {
                        return format!("{value}n");
                    }

                    if *primitive_subtype == midlgen::ir::PrimitiveSubtype::Float32
                        || *primitive_subtype == midlgen::ir::PrimitiveSubtype::Float64
                    {
                        if !value.contains(".") {
                            return format!("{value}.0");
                        }
                        return value.clone();
                    }
                }
                value.clone()
            }
            midlgen::ir::Literal::BoolLiteral { value } => value.clone().to_string(),
            midlgen::ir::Literal::DefaultLiteral => "::Default::default()".to_string(),
        }
    }

    /// compileMemberIdentifier returns a Rust path expression referring to the given
    /// declaration member. It qualifies the crate name if it is external.
    fn compile_member_identifier(&self, val: &ir::EncodedCompoundIdentifier) -> String {
        let ci = val.parse();
        if ci.member.is_none() {
            panic!("expected a member: {:?}", val);
        }

        let decl = val.decl_name();
        let decl_type = self.lookup_decl_info(&decl).r#type.to_owned();
        let member: String;

        let ci_member = ci.member.unwrap();

        match decl_type {
            midlgen::ir::DeclType::BitsDecl => member = self.compile_screaming_snake_identifier(&ci_member),
            midlgen::ir::DeclType::EnumDecl => {
                if ci.name == String::from("ObjType") {
                    member = self.compile_screaming_snake_identifier(&ci_member);
                } else {
                    member = self.compile_camel_identifier(&ci_member);
                }
            }
            _ => panic!("unexpected decl type: {:?}", decl_type),
        }

        format!("{}.{}", self.compile_decl_identifier(&decl), member)
    }

    fn compile_constant(&self, val: &midlgen::ir::Constant, typ: &midlgen::ir::Type) -> String {
        match val {
            midlgen::ir::Constant::Identifier { identifier, .. } => {
                let decl_type = &self.lookup_decl_info(&identifier.decl_name()).r#type;
                let mut expr: String;

                match decl_type {
                    midlgen::ir::DeclType::ConstDecl => {
                        expr = self.compile_decl_identifier(&identifier);
                    }
                    midlgen::ir::DeclType::BitsDecl => {
                        expr = self.compile_member_identifier(&identifier);
                        if let midlgen::ir::Type::PrimitiveType { .. } = typ {
                            expr += ".bits()";
                        }
                    }
                    midlgen::ir::DeclType::EnumDecl => {
                        expr = self.compile_member_identifier(&identifier);
                        if let midlgen::ir::Type::PrimitiveType { .. } = typ {
                            expr += ".value";
                        }
                    }

                    _ => panic!("unexpected decl type: {:?}", decl_type),
                }

                // TODO(https://fxbug.dev/62520): midlc allows conversions between primitive
                // types. Ideally the JSON IR would model these conversions explicitly.
                // In the meantime, just assume that a cast is always necessary.
                if let midlgen::ir::Type::PrimitiveType { primitive_subtype, .. } = typ {
                    let (subtype, _) = self.compile_primitive_subtype(primitive_subtype);
                    expr = format!("{} as {}", expr, subtype)
                }

                expr
            }
            midlgen::ir::Constant::LiteralConstant { literal, .. } => self.compile_literal(literal, typ),
            midlgen::ir::Constant::BinaryOperator { value, .. } => {
                if let midlgen::ir::Type::PrimitiveType { .. } = typ {
                    return value.clone();
                }

                if let midlgen::ir::Type::IdentifierType { identifier, .. } = typ {
                    let decl = self.compile_decl_identifier(identifier);
                    // TODO(https://github.com/rust-lang/rust/issues/67441):
                    // Use from_bits(%s).unwrap() once the const_option feature is stable.
                    return format!("{}::from_bits_truncate({})", decl, value);
                }

                panic!("unsupported type: {:?}", typ);
            }
            _ => panic!("unknown constant kind: {:?}", val),
        }
    }

    fn compile_const(&self, val: midlgen::ir::Const) -> types::Const {
        let ir = val.clone();
        let r#type;

        if let midlgen::ir::Type::StringType { .. } = val.r#type {
            r#type = String::from("string");
        } else {
            r#type = self.compile_type(&val.r#type).param;
        }

        types::Const {
            ir,
            r#type,
            name: self.compile_decl_identifier(&val.name),
            value: self.compile_constant(&val.value, &val.r#type),
        }
    }

    fn compile_union(&self, val: ir::Union) -> types::Union {
        let ir = val.clone();
        let mut members = vec![];

        for v in val.members {
            let ir = v.clone();

            members.push(types::UnionMember {
                ir,
                r#type: self.compile_type(&v.r#type.unwrap()),
                name: self.compile_camel_identifier(&v.name.expect("not reserved")),
                ordinal: v.ordinal,
            })
        }

        types::Union {
            ir,
            name: self.compile_decl_identifier(&val.name),
            members,
        }
    }

    fn compile_enum(&self, val: ir::Enum) -> types::Enum {
        let ir = val.clone();
        let mut members = vec![];

        for v in val.members {
            let ir = v.clone();
            let (ir::Constant::Identifier { value, .. }
            | ir::Constant::LiteralConstant { value, .. }
            | ir::Constant::BinaryOperator { value, .. }) = v.value;

            members.push(types::EnumMember {
                ir,
                name: self.compile_camel_identifier(&v.name),
                value,
            })
        }

        //let min_member = self.find_min_enum_value(&val.r#type, &members).clone();
        let (_, value_type) = self.compile_primitive_subtype(&val.r#type);

        types::Enum {
            ir,
            name: self.compile_decl_identifier(&val.name),
            underlying_type: value_type,
            members,
            min_member: String::from("test"),
        }
    }

    fn compile_struct_member(&self, val: midlgen::ir::StructMember) -> types::StructMember {
        let ir = val.clone();

        types::StructMember {
            ir,
            r#type: self.compile_type(&val.r#type),
            name: self.compile_snake_identifier(&val.name),
            offset_v2: val.field_shape_v2.offset,
        }
    }

    fn compile_struct(&self, val: midlgen::ir::Struct) -> types::Struct {
        let ir = val.clone();
        let name = self.compile_decl_identifier(&val.name);
        let mut has_padding: bool = false;
        let mut members = vec![];

        for v in val.members.iter() {
            let member = self.compile_struct_member(v.clone());
            members.push(member);
            has_padding = has_padding || (v.field_shape_v2.padding != 0);
        }

        types::Struct {
            ir,
            eci: val.name.clone(),
            name,
            members,
            padding_markers_v2: val.build_padding_markers(ir::PaddingConfig::default()),
            flattened_padding_markers_v2: val.build_padding_markers(ir::PaddingConfig {
                flatten_structs: true,
                flatten_arrays: true,
                // resolve_struct:  c.resolveStruct,
            }),
            size_v2: val.type_shape_v2.inline_size,
            alignment_v2: val.type_shape_v2.alignment,
            has_padding,
        }
    }

    fn compile_parameters(&self) -> Vec<types::Parameter> {
        // Implement the logic to compile parameters from a payload
        Vec::new()
    }

    fn type_expr_for_method(
        &self,
        val: &ir::ProtocolMethod,
        request: &[types::Parameter],
        response: &[types::Parameter],
        name: &str,
    ) -> String {
        let request_size_v2 = val
            .request_payload
            .as_ref()
            .map_or(0, |payload| payload.get_type_shape_v2().unwrap().inline_size);
        let response_size_v2 = val
            .response_payload
            .as_ref()
            .map_or(0, |payload| payload.get_type_shape_v2().unwrap().inline_size);

        format!(
            "midl.MethodType(\n\
            \trequest: {},\n\
            \tresponse: {},\n\
            \tname: r\"{}\",\n\
            \trequestInlineSizeV2: {},\n\
            \tresponseInlineSizeV2: {},\n\
            )",
            "", "", name, request_size_v2, response_size_v2
        )
    }

    fn empty_payload(midl_type: String) -> types::Payload {
        types::Payload {
            midl_type,
            parameters: vec![], //OwnedType:       "()",
                                //TupleType:       "()",
                                //EncodeExpr:      "()",
        }
    }

    fn payload_for_type(&self, payload_type: types::Type) -> types::Payload {
        let id = payload_type.identifier.as_ref().unwrap();
        let type_name = self.compile_decl_identifier(id);
        let st = self.structs.get(id);

        let s = types::Payload {
            midl_type: type_name.clone(),
            parameters: vec![types::Parameter {
                name: "payload".to_owned(),
                param_type: payload_type.param.clone(),
                //    owned_type: payload_type.owned.clone(),
                //    struct_member_type: None,
            }],
            //encode_expr: convert_param_to_encode_expr(&param_name, &payload_type),
            // convert_to_tuple: Box::new(move |owned| owned.to_string()),
            // convert_to_fields: Box::new(move |owned| format!("{}: {},", param_name, owned)),
        };

      
        return s;

        // Not a struct: table or union payload.
        if st.is_none() {
            let param_name = "payload".to_string();

            return types::Payload {
                midl_type: type_name.clone(),
                parameters: vec![types::Parameter {
                    name: param_name.clone(),
                    param_type: payload_type.param.clone(),
                    //    owned_type: payload_type.owned.clone(),
                    //    struct_member_type: None,
                }],
                //encode_expr: convert_param_to_encode_expr(&param_name, &payload_type),
                // convert_to_tuple: Box::new(move |owned| owned.to_string()),
                // convert_to_fields: Box::new(move |owned| format!("{}: {},", param_name, owned)),
            };
        }

        let st = st.unwrap();

        // Empty struct with error syntax.
        if st.members.is_empty() {
            return Compiler::empty_payload(String::from("midl.EmptyStruct"));
        }

        // Struct payload. Flatten it to parameters.
        let mut parameters = Vec::new();
        //let mut owned_types = Vec::new();
        //let mut encode_exprs = Vec::new();

        for v in &st.members {
            let param_name = self.compile_snake_identifier(&v.name);
            let typ = self.compile_type(&v.r#type);
            parameters.push(types::Parameter {
                name: param_name.clone(),
                param_type: typ.param.clone(),
                //owned_type: typ.owned.clone(),
                //struct_member_type: Some(typ.clone()),
            });
            //owned_types.push(typ.owned.clone());
            //encode_exprs.push(convert_param_to_encode_expr(&param_name, &typ));
        }

        types::Payload {
            midl_type: type_name.clone(),
            //owned_type: type_name.clone(),
            //tuple_type: fmt_one_or_tuple(&owned_types),
            parameters,
            //encode_expr: fmt_tuple(&encode_exprs),
            //convert_to_tuple: Box::new(move |owned| {
            //    parameters.iter().map(|param| format!("{}.{}", owned, param.name)).collect::<Vec<_>>().join(", ")
            //}),
            //convert_to_fields: Box::new(move |owned| {
            //    parameters.iter().map(|param| format!("{}: {}.{}{}", param.name, owned, param.name, if param != parameters.last().unwrap() { ",\n" } else { "" })).collect::<String>()
            //}),
        }
    }

    fn compile_response(&self, method: &ir::ProtocolMethod) -> types::Payload {
        if let None = method.response_payload {
            // Empty payload, e.g. the response of `Foo() -> ();` or `-> Foo();`.
            return Compiler::empty_payload(String::from("midl.EmptyPayload"));
        }

        if !method.has_result_union() {
            // Plain payload with no flexible/error result union.
            return self.payload_for_type(self.compile_type(method.response_payload.as_ref().unwrap()));
        }

        let inner_type = self.compile_type(method.success_type.as_ref().unwrap());
        let inner = self.payload_for_type(inner_type);
        let mut err_type = None;

        if method.has_error {
            err_type = Some(self.compile_type(method.error_type.as_ref().unwrap()))
        }

        let mut p = types::Payload::default();

        // Set FidlType and OwnedType, which are different for each of the 3 cases.
        if method.has_framework_error() && method.has_error {
            p.midl_type = format!(
                "midl.FlexibleResultType<{}, {}>",
                &inner.midl_type,
                &err_type.unwrap().midl
            )
        } else if method.has_framework_error() {
            p.midl_type = format!("midl.FlexibleType<{}>", &inner.midl_type)
        } else if method.has_error {
            p.midl_type = format!("midl.ResultType<{}, {}>", &inner.midl_type, &err_type.unwrap().midl)
        } else {
            panic!("should have returned earlier")
        }

        // For FlexibleType and FlexibleResultType, we need to wrap the value before encoding.
        if method.has_framework_error() {
            //name, _, _ := strings.Cut(p.OwnedType, "<")
            //p.EncodeExpr = fmt.Sprintf("%s::new(%s)", name, p.EncodeExpr)
        }

        p
    }

    fn compile_request(&self, method: &ir::ProtocolMethod) -> types::Payload {
        if !method.has_request {
            // Not applicable (because the method is an event).
            return types::Payload::default();
        }

        if let Some(ref ty) = method.request_payload {
            // Struct, table, or union request payload.
            return self.payload_for_type(self.compile_type(ty));
        } else {
            // Empty payload, e.g. the request of `Foo();`.
            return Compiler::empty_payload(String::from("midl.EmptyPayload"));
        }
    }

    fn compile_method(
        &self,
        val: &ir::ProtocolMethod,
        protocol: &types::Protocol,
        midl_protocol: &ir::Protocol,
    ) -> types::Method {
        let protocol_name = self.compile_decl_identifier(&midl_protocol.name);
        let name = self.compile_camel_identifier(&val.name);
        let mut request = Compiler::empty_payload(String::from("midl.EmptyPayload"));
        let mut response = Compiler::empty_payload(String::from("midl.EmptyPayload"));
        let mut async_response_class = String::new();
        let mut async_response_type = String::new();

        /*if val.has_response {
            match response.method_parameters.len() {
                0 => {
                    async_response_type = "void".to_string();
                }
                1 => {
                    async_response_type = response.method_parameters[0].param_type.decl.clone();
                }
                _ => {
                    async_response_type = format!("{}${}$Response", protocol.name, val.name);
                    async_response_class = async_response_type.clone();
                }
            }
        }*/

        let transitional = false; //val.lookup_attribute("transitional").is_some();

        types::Method {
            ir: val.clone(),
            ordinal: val.ordinal,
            ordinal_name: format!("_{}_{}_Ordinal", protocol_name, val.name),
            name,
            has_request: val.has_request,
            has_response: val.has_response,
            request: self.compile_request(val),
            response: self.compile_response(val),
            async_response_class,
            async_response_type,
            callback_type: format!(
                "{}{}Callback",
                protocol_name,
                self.compile_screaming_snake_identifier(&val.name)
            ),
            type_symbol: format!("_{}_{}_Type", protocol_name, val.name),
            type_expr: String::new(), /*self.type_expr_for_method(
                                          val,
                                          &request,
                                          &response.wire_parameters,
                                          &format!("{}.{}", protocol_name, val.name),
                                      )*/
            transitional,
        }
    }

    fn compile_protocol(&self, val: midlgen::ir::Protocol) -> types::Protocol {
        let name = self.compile_decl_identifier(&val.name);
        let mut r = types::Protocol {
            ir: val.clone(),
            eci: val.name.clone(),
            marker: format!("{}Marker", name),
            proxy: format!("{}Proxy", name),
            proxy_interface: format!("{}ProxyInterface", name),
            synchronous_proxy: format!("{}SynchronousProxy", name),
            request: format!("{}Request", name),
            request_stream: format!("{}RequestStream", name),
            event: format!("{}Event", name),
            event_stream: format!("{}EventStream", name),
            control_handle: format!("{}ControlHandle", name),
            discoverable: false,
            debug_name: String::new(),
            methods: Vec::new(),
            has_events: false,
        };

        let binding = val.get_protocol_name();
        let discoverable_name = binding.trim_matches('"');
        if !discoverable_name.is_empty() {
            r.discoverable = true;
            r.debug_name = discoverable_name.to_string();
        } else {
            r.debug_name = format!("(anonymous) {}", name);
        }

        for v in &val.methods {
            let m = self.compile_method(v, &r, &val);
            r.methods.push(m);
            if !v.has_request && v.has_response {
                r.has_events = true;
            }
        }

        r
    }
}

fn format_library_name(library: ir::LibraryIdentifier) -> String {
    library.encode().0.replace(".", "_")
}

// Compile the language independent type definition into the Dart-specific representation.
pub fn compile(ir: ir::Root) -> Root {
    let this_lib_parsed = ir.name.parse();

    let mut compiler = Compiler {
        decls: ir.decl_info(),
        root: Root::default(),
        structs: HashMap::new(),
        // experiments: .experiments,
        // types_root: r,
        library: this_lib_parsed,
        // paramableTypes: map[fidlgen.EncodedCompoundIdentifier]Parameterizer{},
    };

    for s in ir.struct_declarations.iter() {
        compiler.structs.insert(s.name.clone(), s.clone());
    }

    let mut consts = vec![];
    let mut unions = vec![];
    let mut enums = vec![];
    let mut bits = vec![];
    let mut structs = vec![];
    let mut protocols = vec![];
    let mut tables = vec![];

    for const_decl in ir.const_declarations {
        consts.push(compiler.compile_const(const_decl));
    }

    for enum_decl in ir.enum_declarations {
        enums.push(compiler.compile_enum(enum_decl));
    }

    for union_decl in ir.union_declarations {
        unions.push(compiler.compile_union(union_decl));
    }

    for struct_decl in ir.struct_declarations {
        structs.push(compiler.compile_struct(struct_decl));
    }

    for protocol_decl in ir.protocol_declarations {
        protocols.push(compiler.compile_protocol(protocol_decl));
    }

    let library_name = format!("midl_{}", format_library_name(compiler.library));

    types::Root {
        library_name,
        experiments: vec![],
        imports: vec![],
        consts,
        enums,
        bits,
        structs,
        unions,
        tables,
        protocols,
        external_structs: vec![],
    }
}
