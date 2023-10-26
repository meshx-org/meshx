pub mod data;
pub mod flow;
mod state;

use self::data::{InputPorts, OutputPorts, RuntimeType};
use self::flow::FlowControls;
use self::state::NodeState;

use crate::errors::FlowError;
use crate::flow::{Flow, Node};
use crate::workers::Workers;

use std::collections::HashMap;
use std::rc::Rc;

/// A separeated runtime for each flow evaulation.
pub struct Process {
    workers: Rc<Workers>,
    flow_stack: Vec<String>,
    states: HashMap<String, NodeState>,
    flow: Flow,
}

fn is_flow(port_key: &str) -> bool {
    port_key.starts_with('$')
}

// TODO: we can run this function before eval starts
fn build_outputs(node: &Node, controls: &mut FlowControls, outputs: &mut OutputPorts) {
    for (key, output_conns) in &node.outputs {
        if is_flow(key.as_str()) {
            //TODO: create dummy flow

            for conn in output_conns {
                let next_node_id = conn.nid.clone();
                controls.port_mapping.insert(key.clone(), next_node_id);
            }
        } else {
            // Default value
            outputs.write(key.clone(), RuntimeType::Null).unwrap();
        }
    }
}

fn build_inputs(node: &Node, inputs: &mut InputPorts, states: &HashMap<String, NodeState>) {
    for (key, input_conns) in &node.inputs {
        if is_flow(key) {
            continue;
        }

        let connection = input_conns.get(0);

        match connection {
            Some(output) => {
                // TODO: push data nodes to the stack and process them

                // When building inputs we actualy share a reference to an output data
                let other_node_state = states.get(&output.nid).unwrap();
                let data = other_node_state.outputs.ports.get(&output.key).unwrap().clone();

                inputs.ports.insert(key.clone(), data);
            }

            None => unimplemented!(),
        }
    }
}

impl Process {
    pub fn new(workers: Rc<Workers>, flow: Flow) -> Self {
        let mut states: HashMap<String, NodeState> = HashMap::new();
        states.reserve(flow.len());

        for nid in flow.keys() {
            states.insert(
                nid.clone(),
                NodeState {
                    processed: false,
                    outputs: OutputPorts::new(),
                },
            );
        }

        Process {
            workers,
            flow,
            states,
            flow_stack: Vec::new(),
        }
    }

    /// Start evaulating the flow
    /// * `entry_nid` - The id of the node to start from
    pub fn dispatch(&mut self, entry_nid: String) -> Result<(), FlowError> {
        // Push the entry to the stack
        self.flow_stack.push(entry_nid);

        // Process the stack
        while !self.flow_stack.is_empty() {
            // println!("state: {:?}", self.states);
            let next_nid = self.flow_stack.pop().unwrap();
            log::trace!(target: "flow-process", "Flowing => {}", next_nid);
            self.state_transition(next_nid)?;
        }

        log::debug!(target: "flow-process", "flow completed");

        Ok(())
    }

    /// Evaulate a single node
    /// * `nid` - The id of the node to process
    fn state_transition(&mut self, nid: String) -> Result<(), FlowError> {
        // Get immutable reference to the current state
        let state = self.states.get(&nid).expect("Bug");

        // Every node need to be processed once (fail early)
        if state.processed {
            return Err(FlowError::AlreadyEvaulated);
        }

        // Get reference to the current node
        let node = self.flow.get(&nid);
        let node = node.ok_or(FlowError::MissingNode)?;

        // Get reference to the current worker
        let worker = self.workers.get(&node.component_id);
        let worker = worker.ok_or_else(|| FlowError::MissingWorker(node.component_id.clone()))?;

        log::trace!(target: "flow-process",
            "Evaling node: {} - {}",
            node.node_id, node.component_id
        );

        // Inputs constructed before each node eval unlike outputs which are stored
        // in the node state
        let mut inputs = InputPorts::new();

        // Extract input data from other node outputs (and if not evaulated before, eval them)
        build_inputs(node, &mut inputs, &self.states);

        // Get mutable reference to the current state TODO: find a way to remove double get
        let state = self.states.get_mut(&nid).unwrap();

        let mut controls = FlowControls {
            port_mapping: HashMap::new(),
            flow_stack: &mut self.flow_stack,
        };

        // Construct flow control, and hold a reference to the flow stack in it.
        build_outputs(node, &mut controls, &mut state.outputs);

        // Invoke component worker for the current node
        worker(node, &inputs, &mut state.outputs, &mut controls);

        state.processed = true;

        Ok(())
    }
}
