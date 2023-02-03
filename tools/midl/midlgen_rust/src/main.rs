mod helpers;
mod types;

use handlebars::Handlebars;

use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use types::*;

use crate::helpers::{PrintfHelper, IfCondHelper};

fn compile(ir: midlgen::Root) -> Root {
    // TODO: add ir compilation

    let const1 = Const {
        base: midlgen::Const {
            doc: Some(String::new()),
        },
        name: String::from("TEST_CONST"),
        r#type: String::from("u32"),
        value: String::from("123"),
    };

    let enum_member1 = EnumMember {
        base: midlgen::EnumMember {
            doc: None,
            value: Value::Null,
        },
        name: String::from("default"),
        value: String::from("1"),
    };

    let enum1 = Enum {
        base: midlgen::Enum {
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
        base: midlgen::StructMember {
            doc: None,
            r#type: String::from("uint32"),
            of: None,
        },
        og_type: midlgen::Type {},
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
        base: midlgen::Protocol {
            doc: None,
            methods: vec![],
        },
        eci: midlgen::EncodedCompoundIdentifier,
        name: String::from("TestProtocol"),
        methods: vec![],
        protocol_name: String::from("TestProtocol"),
    };

    let struct1 = Struct {
        base: midlgen::Struct {
            doc: Some(String::new()),
            member: HashMap::new(),
        },
        name: String::from("TestSctruct"),
        eci: midlgen::EncodedCompoundIdentifier,
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

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("render error")]
    TemplateRender(#[from] handlebars::RenderError),

    #[error("template error")]
    Template,

    #[error("unknown generator error")]
    Unknown,
}

struct Generator {
    registry: Handlebars<'static>,
}

impl Generator {
    fn new() -> Self {
        let mut registry = Handlebars::new();
        let generate_source_tpl = include_str!("./templates/sourcefile.hbs");
        let const_declaration_tpl = include_str!("./templates/const.hbs");
        let enum_declaration_tpl = include_str!("./templates/enum.hbs");
        let struct_declaration_tpl = include_str!("./templates/struct.hbs");
        let protocol_declaration_tpl = include_str!("./templates/protocol.hbs");

        registry.register_helper("if_cond", Box::new(IfCondHelper {}));
        registry.register_helper("printf", Box::new(PrintfHelper {}));

        registry
            .register_template_string("GenerateSourceFile", generate_source_tpl)
            .unwrap();

        registry
            .register_partial("ConstDeclaration", const_declaration_tpl)
            .unwrap();

        registry
            .register_partial("EnumDeclaration", enum_declaration_tpl)
            .unwrap();

        registry
            .register_partial("StructDeclaration", struct_declaration_tpl)
            .unwrap();

        registry
            .register_partial("ProtocolDeclaration", protocol_declaration_tpl)
            .unwrap();

        Generator { registry }
    }

    fn generate_fidl(&self, ir: midlgen::Root, output_filename: String) -> Result<(), GeneratorError> {
        let tree = compile(ir);
        return self.generate_file(output_filename, "GenerateSourceFile", tree);
    }

    fn generate_file(&self, filename: String, tmpl: &str, data: types::Root) -> Result<(), GeneratorError> {
        let path = PathBuf::from(filename.clone());
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir)?;

        let result = self.execute_template(tmpl, data)?;

        println!("{:?}", result);

        let mut file = File::create(filename)?;
        file.write_all(result.as_bytes())?;

        // TODO: formating
        // let formatted, err = gen.formatter.Format(generated)
        // if err != nil {
        //     return fmt.Errorf("Error formatting source: %w", err)
        //0 }

        // return WriteFileIfChanged(filename, formatted)

        Ok(())
    }

    fn execute_template(&self, tmpl: &str, root: types::Root) -> Result<String, GeneratorError> {
        let result = self.registry.render(tmpl, &root)?;
        Ok(result)
    }
}

fn main() -> Result<(), GeneratorError> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();

    let root = serde_json::from_str::<midlgen::Root>(buffer.as_str())?;

    let generator = Generator::new();
    generator.generate_fidl(root, "./OUTPUT.rs".to_owned())?;

    Ok(())
}
