// Copyright 2023 MeshX Contributors. All rights reserved.

mod errors;
mod flow;
mod parser;
mod process;
mod test_flow;
mod vm;
mod workers;

use std::collections::HashMap;

use flow::Node;
use process::{
    data::{InputPorts, OutputPorts},
    flow::FlowControls,
};

use crate::{
    flow::FlowModule,
    parser::parse_flow,
    test_flow::TEST_FLOW,
    vm::{FlowVM, Module},
    workers::Workers,
};

fn dummy1(_node: &Node, _i: &InputPorts, _o: &mut OutputPorts, f: &mut FlowControls) {
    log::debug!("run dummy1");
    f.flow("$out".to_string()).unwrap();
}

fn dummy2(_node: &Node, _i: &InputPorts, _o: &mut OutputPorts, _f: &mut FlowControls) {
    log::debug!("run dummy2");
}

fn main() {
    env_logger::init();

    let mut workers: Workers = HashMap::new();
    workers.insert("core:dummy1".to_string(), dummy1);
    workers.insert("core:dummy2".to_string(), dummy2);

    let import_hook = &|module| {
        let flow: FlowModule = parse_flow(TEST_FLOW).unwrap();

        log::debug!("{:#?}", flow);

        Module(flow)
    };

    let mut vm = FlowVM::new(workers, import_hook);

    vm.import("meshx.flow");

    let result = vm.invoke("meshx/Component::OnStarted");
    //vm.spawn_sync(flow, "1".to_string(), vec![]);

    println!("finish");
}
