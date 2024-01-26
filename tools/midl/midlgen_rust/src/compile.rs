use convert_case::Casing;
use std::collections::{HashMap, HashSet};

use crate::types::{self, Derives, Root, Struct, StructMember, Type};
use midlgen::ir::{
    EncodedCompoundIdentifier, HandleSubtype, Identifier, PaddingConfig, PrimitiveSubtype, RESOURCE_TYPE, VALUE_TYPE,
};

struct Compiler {
    decls: midlgen::ir::DeclInfoMap,
    library: midlgen::ir::LibraryIdentifier,
    structs: HashMap<midlgen::ir::EncodedCompoundIdentifier, midlgen::ir::Struct>,
}

lazy_static::lazy_static! {
    static ref PRIMITIVE_SUBTYPES: HashMap<PrimitiveSubtype, String> = {
        let mut m = HashMap::new();
        m.insert(PrimitiveSubtype::Bool, "bool".to_string());
        m.insert(PrimitiveSubtype::Uint8, "u8".to_string());
        m.insert(PrimitiveSubtype::Uint16, "u16".to_string());
        m.insert(PrimitiveSubtype::Uint32, "u32".to_string());
        m.insert(PrimitiveSubtype::Uint64, "u64".to_string());
        m.insert(PrimitiveSubtype::Int8, "i8".to_string());
        m.insert(PrimitiveSubtype::Int16, "i16".to_string());
        m.insert(PrimitiveSubtype::Int32, "i32".to_string());
        m.insert(PrimitiveSubtype::Int64, "i64".to_string());
        m.insert(PrimitiveSubtype::Float32, "f32".to_string());
        m.insert(PrimitiveSubtype::Float64, "f64".to_string());
        m
    };
}

lazy_static::lazy_static! {
    static ref HANDLE_SUBTYPES: HashMap<midlgen::ir::HandleSubtype, String> = {
        let mut m = HashMap::new();
        m.insert(midlgen::ir::HandleSubtype::None, "midl::Handle".to_string());
         m.insert(midlgen::ir::HandleSubtype::Channel, "midl::Channel".to_string());
        m.insert(midlgen::ir::HandleSubtype::Event, "midl::Event".to_string());
        m.insert(midlgen::ir::HandleSubtype::EventPair, "midl::EventPair".to_string());
        m.insert(midlgen::ir::HandleSubtype::Exception, "midl::Exception".to_string());
        m.insert(midlgen::ir::HandleSubtype::Guest, "midl::Guest".to_string());
        m.insert(midlgen::ir::HandleSubtype::Fifo, "midl::Fifo".to_string());
        m.insert(midlgen::ir::HandleSubtype::Bti, "midl::Bti".to_string());
        m.insert(midlgen::ir::HandleSubtype::Clock, "midl::Clock".to_string());
        m.insert(midlgen::ir::HandleSubtype::Debuglog, "midl::Debuglog".to_string());
        m.insert(midlgen::ir::HandleSubtype::Interrupt, "midl::Interrupt".to_string());
        m.insert(midlgen::ir::HandleSubtype::Iommu, "midl::Iommu".to_string());
        m.insert(midlgen::ir::HandleSubtype::Job, "midl::Job".to_string());
        m.insert(midlgen::ir::HandleSubtype::Msi, "midl::Msi".to_string());
        m.insert(midlgen::ir::HandleSubtype::Pager, "midl::Pager".to_string());
        m.insert(midlgen::ir::HandleSubtype::PciDevice, "midl::PciDevice".to_string());
        m.insert(midlgen::ir::HandleSubtype::Pmt, "midl::Pmt".to_string());
        m.insert(midlgen::ir::HandleSubtype::Port, "midl::Port".to_string());
        m.insert(midlgen::ir::HandleSubtype::Process, "midl::Process".to_string());
        m.insert(midlgen::ir::HandleSubtype::Profile, "midl::Profile".to_string());
        m.insert(midlgen::ir::HandleSubtype::Resource, "midl::Resource".to_string());
        m.insert(midlgen::ir::HandleSubtype::Socket, "midl::Socket".to_string());
        m.insert(midlgen::ir::HandleSubtype::Stream, "midl::Stream".to_string());
        m.insert(midlgen::ir::HandleSubtype::SuspendToken, "midl::SuspendToken".to_string());
        m.insert(midlgen::ir::HandleSubtype::Thread, "midl::Thread".to_string());
        m.insert(midlgen::ir::HandleSubtype::Timer, "midl::Timer".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vcpu, "midl::Vcpu".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vmar, "midl::Vmar".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vmo, "midl::Vmo".to_string());

        m
    };
}

lazy_static::lazy_static! {
    static ref OBJECT_TYPES: HashMap<midlgen::ir::HandleSubtype, String> = {
        let mut m = HashMap::new();
        m.insert(midlgen::ir::HandleSubtype::None, "midl::ObjectType::NONE".to_string());
        m.insert(midlgen::ir::HandleSubtype::Channel, "midl::ObjectType::CHANNEL".to_string());
        m.insert(midlgen::ir::HandleSubtype::Event, "midl::ObjectType::EVENT".to_string());
        m.insert(midlgen::ir::HandleSubtype::EventPair, "midl::ObjectType::EVENTPAIR".to_string());
        m.insert(midlgen::ir::HandleSubtype::Exception, "midl::ObjectType::EXCEPTION".to_string());
        m.insert(midlgen::ir::HandleSubtype::Guest, "midl::ObjectType::GUEST".to_string());
        m.insert(midlgen::ir::HandleSubtype::Fifo, "midl::ObjectType::FIFO".to_string());
        m.insert(midlgen::ir::HandleSubtype::Bti, "midl::ObjectType::BTI".to_string());
        m.insert(midlgen::ir::HandleSubtype::Clock, "midl::ObjectType::CLOCK".to_string());
        m.insert(midlgen::ir::HandleSubtype::Debuglog, "midl::ObjectType::DEBUGLOG".to_string());
        m.insert(midlgen::ir::HandleSubtype::Interrupt, "midl::ObjectType::INTERRUPT".to_string());
        m.insert(midlgen::ir::HandleSubtype::Iommu, "midl::ObjectType::IOMMU".to_string());
        m.insert(midlgen::ir::HandleSubtype::Job, "midl::ObjectType::JOB".to_string());
        m.insert(midlgen::ir::HandleSubtype::Msi, "midl::ObjectType::MSI".to_string());
        m.insert(midlgen::ir::HandleSubtype::Pager, "midl::ObjectType::PAGER".to_string());
        m.insert(midlgen::ir::HandleSubtype::PciDevice, "midl::ObjectType::PCIDEVICE".to_string());
        m.insert(midlgen::ir::HandleSubtype::Pmt, "midl::ObjectType::PMT".to_string());
        m.insert(midlgen::ir::HandleSubtype::Port, "midl::ObjectType::PORT".to_string());
        m.insert(midlgen::ir::HandleSubtype::Process, "midl::ObjectType::PROCESS".to_string());
        m.insert(midlgen::ir::HandleSubtype::Profile, "midl::ObjectType::PROFILE".to_string());
        m.insert(midlgen::ir::HandleSubtype::Resource, "midl::ObjectType::RESOURCE".to_string());
        m.insert(midlgen::ir::HandleSubtype::Socket, "midl::ObjectType::SOCKET".to_string());
        m.insert(midlgen::ir::HandleSubtype::Stream, "midl::ObjectType::STREAM".to_string());
        m.insert(midlgen::ir::HandleSubtype::SuspendToken, "midl::ObjectType::SUSPENDTOKEN".to_string());
        m.insert(midlgen::ir::HandleSubtype::Thread, "midl::ObjectType::THREAD".to_string());
        m.insert(midlgen::ir::HandleSubtype::Timer, "midl::ObjectType::TIMER".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vcpu, "midl::ObjectType::VCPU".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vmar, "midl::ObjectType::VMAR".to_string());
        m.insert(midlgen::ir::HandleSubtype::Vmo, "midl::ObjectType::VMO".to_string());

        m
    };
}

lazy_static::lazy_static! {
    static ref RESERVED_WORDS: HashSet<String> = {
        let mut m = HashSet::new();
        m.insert("as".to_string());
        m.insert("box".to_string());
        m.insert("break".to_string());
        m.insert("const".to_string());
        m.insert("continue".to_string());
        m.insert("crate".to_string());
        m.insert("else".to_string());
        m.insert("enum".to_string());
        m.insert("extern".to_string());
        m.insert("false".to_string());
        m.insert("fn".to_string());
        m.insert("for".to_string());
        m.insert("if".to_string());
        m.insert("impl".to_string());
        m.insert("in".to_string());
        m.insert("let".to_string());
        m.insert("loop".to_string());
        m.insert("match".to_string());
        m.insert("mod".to_string());
        m.insert("move".to_string());
        m.insert("mut".to_string());
        m.insert("pub".to_string());
        m.insert("ref".to_string());
        m.insert("return".to_string());
        m.insert("self".to_string());
        m.insert("Self".to_string());
        m.insert("static".to_string());
        m.insert("struct".to_string());
        m.insert("super".to_string());
        m.insert("trait".to_string());
        m.insert("true".to_string());
        m.insert("type".to_string());
        m.insert("unsafe".to_string());
        m.insert("use".to_string());
        m.insert("where".to_string());
        m.insert("while".to_string());

        // Keywords reserved for future use (future-proofing...)
        m.insert("abstract".to_string());
        m.insert("alignof".to_string());
        m.insert("await".to_string());
        m.insert("become".to_string());
        m.insert("do".to_string());
        m.insert("final".to_string());
        m.insert("macro".to_string());
        m.insert("offsetof".to_string());
        m.insert("override".to_string());
        m.insert("priv".to_string());
        m.insert("proc".to_string());
        m.insert("pure".to_string());
        m.insert("sizeof".to_string());
        m.insert("typeof".to_string());
        m.insert("unsized".to_string());
        m.insert("virtual".to_string());
        m.insert("yield".to_string());
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

    fn in_external_library(&self, eci: &midlgen::ir::EncodedCompoundIdentifier) -> bool {
        eci.library_name() != self.library.encode()
    }

    fn compute_use_midl_struct_copy(&self, typ: midlgen::ir::Type) -> bool {
        if let midlgen::ir::Type::StringType { nullable, .. } | midlgen::ir::Type::RequestType { nullable, .. } = typ {
            if nullable {
                return false;
            }
        }

        match typ {
            midlgen::ir::Type::ArrayType { element_type, .. } | midlgen::ir::Type::StringArray { element_type, .. } => {
                return self.compute_use_midl_struct_copy(*element_type.clone())
            }
            midlgen::ir::Type::VectorType { .. }
            | midlgen::ir::Type::StringType { .. }
            | midlgen::ir::Type::HandleType { .. }
            | midlgen::ir::Type::RequestType { .. } => return false,
            midlgen::ir::Type::PrimitiveType { primitive_subtype, .. } => {
                return match primitive_subtype {
                    midlgen::ir::PrimitiveSubtype::Bool
                    | midlgen::ir::PrimitiveSubtype::Float32
                    | midlgen::ir::PrimitiveSubtype::Float64 => true,
                    _ => false,
                }
            }
            midlgen::ir::Type::IdentifierType { .. } => {
                // TODO
                return false;
            }
            _ => panic!("unknown type kind: {:?}", typ),
        }

        /*switch typ.Kind {
        case fidlgen.IdentifierType:
            if c.inExternalLibrary(typ.Identifier) {
                return false
            }
            declType := c.lookupDeclInfo(typ.Identifier).Type
            switch declType {
            case fidlgen.BitsDeclType, fidlgen.EnumDeclType, fidlgen.TableDeclType, fidlgen.UnionDeclType, fidlgen.ProtocolDeclType:
                return false
            case fidlgen.StructDeclType:
                st, ok := c.structs[typ.Identifier]
                if !ok {
                    panic(fmt.Sprintf("struct not found: %v", typ.Identifier))
                }
                return c.computeUseFidlStructCopyForStruct(st)
            default:
                panic(fmt.Sprintf("unknown declaration type: %v", declType))
            }
        }*/
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

    fn compile_decl_identifier(&self, val: &EncodedCompoundIdentifier) -> String {
        let ci = val.parse();

        if ci.member.is_some() {
            panic!("unexpected member: {:?}", val);
        }

        let mut name: String;

        if let midlgen::ir::DeclType::ConstDecl = self.lookup_decl_info(val).r#type {
            name = self.compile_screaming_snake_identifier(ci.name);
        } else {
            name = self.compile_camel_identifier(ci.name);
        }

        // TODO(https://fxbug.dev/66767): This is incorrect. We're calling changeIfReserved
        // a second time, after compileScreamingSnakeIdentifier or
        // compileCamelIdentifier already did. We should only call it once.
        name = self.change_if_reserved(name);

        if self.in_external_library(val) {
            //	let r#crate = compile_library_name(ci.library);
            //	extern_crates[crate] = struct{}{}
            //	return fmt.Sprintf("%s::%s", crate, name)
        }

        name
    }

    /// compileMemberIdentifier returns a Rust path expression referring to the given
    /// declaration member. It qualifies the crate name if it is external.
    fn compile_member_identifier(&self, val: &EncodedCompoundIdentifier) -> String {
        let ci = val.parse();
        if ci.member.is_none() {
            panic!("expected a member: {:?}", val);
        }

        let decl = val.decl_name();
        let decl_type = self.lookup_decl_info(&decl).r#type.to_owned();
        let member: String;

        let ci_member = ci.member.unwrap();

        match decl_type {
            midlgen::ir::DeclType::BitsDecl => member = self.compile_screaming_snake_identifier(ci_member),
            midlgen::ir::DeclType::EnumDecl => {
                if ci.name == String::from("ObjType") {
                    member = self.compile_screaming_snake_identifier(ci_member);
                } else {
                    member = self.compile_camel_identifier(ci_member);
                }
            }
            _ => panic!("unexpected decl type: {:?}", decl_type),
        }

        format!("{}::{}", self.compile_decl_identifier(&decl), member)
    }

    fn compile_snake_identifier(&self, val: Identifier) -> String {
        self.change_if_reserved(val.0.to_case(convert_case::Case::Snake))
    }

    fn compile_camel_identifier(&self, val: midlgen::ir::Identifier) -> String {
        self.change_if_reserved(val.0.to_case(convert_case::Case::UpperCamel))
    }

    fn compile_screaming_snake_identifier(&self, val: midlgen::ir::Identifier) -> String {
        self.change_if_reserved(val.0.to_case(convert_case::Case::ScreamingSnake))
    }

    fn compile_primitive_subtype(&self, val: &PrimitiveSubtype) -> String {
        PRIMITIVE_SUBTYPES
            .get(val)
            .expect(format!("unknown primitive type: {:?}", val).as_str())
            .clone()
    }

    // TODO(https://fxbug.dev/7660): Store this on types directly.
    fn lookup_resourceness(&self, t: &midlgen::ir::Type) -> midlgen::ir::Resourceness {
        match t {
            midlgen::ir::Type::PrimitiveType { .. }
            | midlgen::ir::Type::StringType { .. }
            | midlgen::ir::Type::InternalType { .. } => VALUE_TYPE,
            midlgen::ir::Type::HandleType { .. } | midlgen::ir::Type::RequestType { .. } => RESOURCE_TYPE,
            midlgen::ir::Type::ArrayType { element_type, .. }
            | midlgen::ir::Type::StringArray { element_type, .. }
            | midlgen::ir::Type::VectorType { element_type, .. } => self.lookup_resourceness(element_type.as_ref()),
            midlgen::ir::Type::IdentifierType { identifier, .. } => {
                if let Some(info) = self.decls.get(identifier) {
                    match info.r#type {
                        midlgen::ir::DeclType::BitsDecl | midlgen::ir::DeclType::EnumDecl => VALUE_TYPE,
                        midlgen::ir::DeclType::ProtocolDecl => RESOURCE_TYPE,
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

    fn compile_handle_subtype(&self, val: &HandleSubtype) -> String {
        HANDLE_SUBTYPES
            .get(val)
            .expect(format!("unknown handle type: {:?}", val).as_str())
            .clone()
    }

    fn compile_object_type_const(&self, val: &HandleSubtype) -> String {
        OBJECT_TYPES
            .get(val)
            .expect(format!("unknown handle type: {:?}", val).as_str())
            .clone()
    }

    fn compile_type(&self, val: &midlgen::ir::Type) -> Type {
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
            owned: String::new(),
            param: String::new(),
        };

        match val {
            midlgen::ir::Type::PrimitiveType { primitive_subtype, .. } => {
                let s = self.compile_primitive_subtype(primitive_subtype);
                t.midl = s.clone();
                t.owned = s.clone();
                t.param = s;
                t.primitive_subtype = Some(primitive_subtype.clone());
            }
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
                t.midl = format!("midl::encoding::Array<{}, {}>", el.midl, element_count);
                t.owned = format!("[{}; {}]", el.owned, element_count);

                if resourceness.is_resource_type() {
                    t.param = t.owned.clone();
                } else {
                    t.param = format!("&{}", t.owned);
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
                    t.midl = format!("midl::encoding::UnboundedVector<{}>", el.midl);
                } else {
                    t.midl = format!(
                        "midl::encoding::Vector<{}, {}>",
                        el.midl,
                        element_count.expect("to be some")
                    );
                }

                t.owned = format!("Vec<{}>", el.owned);

                if resourceness.is_resource_type() {
                    t.param = format!("Vec<{}>", el.owned)
                } else {
                    t.param = format!("&[{}]", el.owned)
                }
                if *nullable {
                    t.midl = format!("midl::encoding::Optional<{}>", t.midl);
                    t.owned = format!("Option<{}>", t.owned);
                    t.param = format!("Option<{}>", t.param);
                }
            }
            midlgen::ir::Type::StringType {
                element_count,
                nullable,
                ..
            } => {
                if element_count.is_none() {
                    t.midl = "midl::encoding::UnboundedString".to_string();
                } else {
                    let element_count = element_count.unwrap();
                    t.midl = format!("midl::encoding::BoundedString<{element_count}>");
                }

                t.owned = "String".to_string();
                t.param = "&str".to_string();
                t.nullable = *nullable;

                if *nullable {
                    t.midl = format!("midl::encoding::Optional<{}>", t.midl);
                    t.owned = format!("Option<{}>", t.owned);
                    t.param = format!("Option<{}>", t.param);
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
                t.midl = format!(
                    "midl::encoding::HandleType<{}, {{ {}.into_raw() }}, {}>",
                    s,
                    obj_type,
                    handle_rights.bits()
                );
                t.owned = s.clone();
                t.param = s;

                if *nullable {
                    t.midl = format!("midl::encoding::Optional<{}>", t.midl);
                    t.owned = format!("Option<{}>", t.owned);
                    t.param = format!("Option<%{}>", t.param);
                }
            }
            midlgen::ir::Type::RequestType {
                request_subtype,
                nullable,
                ..
            } => {
                let s = format!(
                    "midl::endpoints::ServerEnd<{}Marker>",
                    self.compile_decl_identifier(request_subtype)
                );
                t.nullable = *nullable;
                t.midl = format!("midl::encoding::Endpoint<{}>", s.clone());
                t.owned = s.clone();
                t.param = s;

                if *nullable {
                    t.midl = format!("midl::encoding::Optional<{}>", t.midl);
                    t.owned = format!("Option<{}>", t.owned);
                    t.param = format!("Option<{}>", t.param);
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
                    midlgen::ir::DeclType::BitsDecl | midlgen::ir::DeclType::EnumDecl => {
                        t.midl = name.clone();
                        t.owned = name.clone();
                        t.param = name;
                    }
                    midlgen::ir::DeclType::StructDecl
                    | midlgen::ir::DeclType::TableDecl
                    | midlgen::ir::DeclType::UnionDecl => {
                        t.midl = name.clone();
                        t.owned = name.clone();

                        if resourceness.is_resource_type() {
                            t.param = name;
                        } else {
                            t.param = format!("&{name}");
                        }
                    }
                    midlgen::ir::DeclType::ProtocolDecl => {
                        let s = format!("midl::endpoints::ClientEnd<{}Marker>", name);
                        t.midl = format!("midl::encoding::Endpoint<{}>", s.clone());
                        t.owned = s.clone();
                        t.param = s;

                        if *nullable {
                            t.midl = format!("midl::encoding::Optional<{}>", t.midl);
                            t.owned = format!("Option<{}>", t.owned);
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

    fn compute_use_midl_struct_copy_for_struct(&self, st: midlgen::ir::Struct) -> bool {
        if st.members.len() == 0 {
            // In Rust, structs containing empty structs do not match the C++ struct layout
            // since empty structs have size 0 in Rust -- even in repr(C).
            return false;
        }

        for member in st.members {
            if !self.compute_use_midl_struct_copy(member.r#type) {
                return false;
            }
        }

        return true;
    }

    fn compile_struct_member(&self, val: midlgen::ir::StructMember) -> StructMember {
        let ir = val.clone();

        StructMember {
            ir,
            r#type: self.compile_type(&val.r#type),
            name: self.compile_snake_identifier(val.name),
            offset_v2: val.field_shape_v2.offset,
        }
    }

    fn compile_literal(&self, val: &midlgen::ir::Literal, typ: &midlgen::ir::Type) -> String {
        match val {
            midlgen::ir::Literal::StringLiteral { value } => {
                let mut b = String::with_capacity(value.len());
                b.push('"');

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

                b.push('"');
                b
            }
            midlgen::ir::Literal::NumericLiteral { value } => {
                if let midlgen::ir::Type::PrimitiveType { primitive_subtype, .. } = typ {
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
                            expr += ".into_primitive()";
                        }
                    }

                    _ => panic!("unexpected decl type: {:?}", decl_type),
                }

                // TODO(https://fxbug.dev/62520): midlc allows conversions between primitive
                // types. Ideally the JSON IR would model these conversions explicitly.
                // In the meantime, just assume that a cast is always necessary.
                if let midlgen::ir::Type::PrimitiveType { primitive_subtype, .. } = typ {
                    expr = format!("{} as {}", expr, self.compile_primitive_subtype(primitive_subtype))
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
            r#type = String::from("&str");
        } else {
            r#type = self.compile_type(&val.r#type).owned;
        }

        types::Const {
            ir,
            r#type,
            name: self.compile_decl_identifier(&val.name),
            value: self.compile_constant(&val.value, &val.r#type),
        }
    }

    fn compile_union(&self, val: midlgen::ir::Union) -> types::Union {
        let mut members = vec![];

        for v in val.members {
            let ir = v.clone();

            if v.reserved {
                continue;
            }

            members.push(types::UnionMember {
                ir,
                r#type: self.compile_type(&v.r#type.unwrap()),
                name: self.compile_camel_identifier(v.name.expect("not reserved")),
                ordinal: v.ordinal,
            })
        }

        types::Union {
            name: self.compile_decl_identifier(&val.name),
            members,
        }
    }

    fn compile_table(&self, val: midlgen::ir::Table) -> types::Table {
        let ir = val.clone();
        let mut members = vec![];

        for v in val.members {
            let ir = v.clone();

            if v.reserved {
                continue;
            }

            members.push(types::TableMember {
                ir,
                r#type: self.compile_type(&v.r#type.unwrap()),
                name: self.compile_snake_identifier(v.name.expect("not reserved")),
                ordinal: v.ordinal,
            })
        }

        types::Table {
            ir,
            name: self.compile_decl_identifier(&val.name),
            eci: val.name,
            members,
        }
    }

    fn find_min_enum_value<'t, 'm>(&self, typ: &'t PrimitiveSubtype, members: &'m Vec<types::EnumMember>) -> String {
        let mut res = String::from("");

        if typ.is_signed() {
            let mut min = i64::MAX;

            for m in members {
                let v = m.value.parse::<i64>();

                match v {
                    Ok(v) => {
                        if v < min {
                            min = v;
                            res = m.name.clone();
                        }
                    }
                    Err(err) => panic!("invalid enum member value: {}", err),
                }
            }
        } else {
            let mut min = u64::MAX;

            for m in members {
                let v = m.value.parse::<u64>();

                match v {
                    Ok(v) => {
                        if v < min {
                            min = v;
                            res = m.name.clone();
                        }
                    }
                    Err(err) => panic!("invalid enum member value: {}", err),
                }
            }
        }

        res
    }

    fn compile_enum(&self, val: midlgen::ir::Enum) -> types::Enum {
        let ir = val.clone();
        let mut members = vec![];

        for v in val.members {
            let ir = v.clone();
            let (midlgen::ir::Constant::Identifier { value, .. }
            | midlgen::ir::Constant::LiteralConstant { value, .. }
            | midlgen::ir::Constant::BinaryOperator { value, .. }) = v.value;

            members.push(types::EnumMember {
                ir,
                name: self.compile_camel_identifier(v.name),
                value,
            })
        }

        let min_member = self.find_min_enum_value(&val.r#type, &members).clone();

        types::Enum {
            ir,
            name: self.compile_decl_identifier(&val.name),
            underlying_type: self.compile_primitive_subtype(&val.r#type),
            members,
            min_member,
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

        let use_midl_struct_copy = self.compute_use_midl_struct_copy_for_struct(val.clone());

        types::Struct {
            ir,
            eci: val.name.clone(),
            name,
            members,
            padding_markers_v2: val.build_padding_markers(PaddingConfig::default()),
            flattened_padding_markers_v2: val.build_padding_markers(PaddingConfig {
                flatten_structs: true,
                flatten_arrays: true,
                // resolve_struct:  c.resolveStruct,
            }),
            size_v2: val.type_shape_v2.inline_size,
            alignment_v2: val.type_shape_v2.alignment,
            has_padding,
            use_midl_struct_copy,
        }
    }
}

pub fn compile(ir: midlgen::ir::Root) -> Root {
    // TODO: add ir compilation

    let this_lib_parsed = ir.name.parse();

    let compiler = Compiler {
        decls: ir.decl_info(),
        library: this_lib_parsed,
        structs: HashMap::new(),
    };

    let mut consts = vec![];
    let mut unions = vec![];
    let mut enums = vec![];
    let mut structs = vec![];
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

    for table_decl in ir.table_declarations {
        tables.push(compiler.compile_table(table_decl));
    }

    // println!("{:#?}", consts);
    // println!("{:#?}", enums);
    // println!("{:#?}", structs);

    Root {
        consts,
        enums,
        structs,
        unions,
        tables,
        extern_crates: vec![],
        external_structs: vec![],
        protocols: vec![],
    }
}
