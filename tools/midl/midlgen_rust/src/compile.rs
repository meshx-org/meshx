use crate::types::*;
use ir::BaseDeclaration;
use midlgen::ir;
use serde_json::Value;
use std::collections::HashMap;

pub fn compile(ir: midlgen::ir::Root) -> Root {
    // TODO: add ir compilation

    let const1 = Const {
        base: ir::Const {
            base: BaseDeclaration {
                name: String::from("TEST_CONST"),
                attributes: vec![],
                location: ir::Location {
                    filename: String::from(""),
                    line: 0,
                    column: 0,
                    length: 0,
                },
            },
            doc: Some(String::new()),
        },
        name: String::from("TEST_CONST"),
        r#type: String::from("u32"),
        value: String::from("123"),
    };

    let enum_member1 = EnumMember {
        base: ir::EnumMember {
            doc: None,
            value: Value::Null,
        },
        name: String::from("default"),
        value: String::from("1"),
    };

    let enum1 = Enum {
        base: ir::Enum {
            base: BaseDeclaration {
                name: String::from(""),
                attributes: vec![],
                location: ir::Location {
                    filename: String::from(""),
                    line: 0,
                    column: 0,
                    length: 0,
                },
            },
            doc: None,
            member: None,
        },
        name: String::from("TestEnum"),
        r#type: String::from("u16"),
        members: vec![],
        min_member: String::from("default"),
        is_flexible: false,
    };

    let struct_member1 = StructMember {
        base: ir::StructMember {
            doc: None,
            r#type: String::from("uint32"),
            of: None,
        },
        og_type: ir::Type {},
        r#type: String::from("u32"),
        name: String::from("test_member"),
        offset_v1: 0,
        offset_v2: 0,
        has_default: true,
        default_value: String::from("12"),
        has_handle_metadata: false,
        handle_rights: String::from("WRITE"),
        handle_subtype: String::from("Test"),
    };

    let protocol1 = Protocol {
        base: ir::Protocol {
            doc: None,
            methods: vec![],
        },
        eci: ir::EncodedCompoundIdentifier("".to_owned()),
        name: String::from("TestProtocol"),
        methods: vec![],
        protocol_name: String::from("TestProtocol"),
    };

    let struct1 = Struct {
        base: ir::Struct {
            base: BaseDeclaration {
                name: "".to_owned(),
                attributes: vec![],
                location: ir::Location {
                    filename: String::from(""),
                    line: 0,
                    column: 0,
                    length: 0,
                },
            },
            doc: Some(String::new()),
            member: HashMap::new(),
        },
        name: String::from("TestSctruct"),
        eci: ir::EncodedCompoundIdentifier("".to_owned()),
        derives: Derives(0),
        members: vec![struct_member1],
        padding_markers_v1: vec![],
        padding_markers_v2: vec![],
        flattened_padding_markers_v1: vec![],
        flattened_padding_markers_v2: vec![],
        size_v1: 12,
        size_v2: 12,
        alignment_v1: 12,
        alignment_v2: 12,
        has_padding: true,
        use_fidl_struct_copy: true,
    };

    Root {
        consts: vec![const1],
        enums: vec![enum1],
        structs: vec![struct1],
        extern_crates: vec![],
        external_structs: vec![],
        protocols: vec![protocol1],
        handle_metadata_wrappers: vec![],
    }
}
