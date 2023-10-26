use crate::process::data::{InputPorts, OutputPorts, RuntimeType};
use crate::process::flow::FlowControls;
use std::collections::HashMap;

pub type WorkerFn = fn(&Node, &InputPorts, &mut OutputPorts, &mut FlowControls);
pub type Workers = HashMap<String, WorkerFn>;

use crate::flow::Node;
use rand::Rng;

fn test(_node: &Node, _inputs: &InputPorts, outputs: &mut OutputPorts, ctrl: &mut FlowControls) {
    // Read
    /*match inputs.read("test".to_string()) {
        Ok(data) => {}
        Err(err) => {}
    };*/

    let mut rng = rand::thread_rng();
    let r = rng.gen_range(1000..5000);
    std::thread::sleep(std::time::Duration::from_millis(r));

    // Write
    outputs
        .write("out:data".to_string(), RuntimeType::Number(10.0))
        .unwrap();

    // Flow
    ctrl.flow(String::from("$out")).unwrap();
}

fn print(_node: &Node, inputs: &InputPorts, _outputs: &mut OutputPorts, _ctrl: &mut FlowControls) {
    // Read
    match inputs.read("in:data".to_string()) {
        Ok(data) => println!("{:?}", data),
        Err(_err) => {}
    };

    let mut rng = rand::thread_rng();
    let r = rng.gen_range(1000..5000);
    std::thread::sleep(std::time::Duration::from_millis(r));

    // Sleep for random time
    let mut t = 0;
    for _n in 0..100_000 {
        t += 1;
    }

    println!("loop res = {}", t)
}

pub fn init_workers() -> Workers {
    let mut workers: Workers = HashMap::new();
    workers.insert("core:test".to_owned(), test);
    workers.insert("core:print".to_owned(), print);

    workers
}
