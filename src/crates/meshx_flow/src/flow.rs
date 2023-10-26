use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NodeInput {
    pub nid: String,

    #[serde(rename(deserialize = "in"))]
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NodeOutput {
    pub nid: String,

    #[serde(rename(deserialize = "out"))]
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Node {
    #[serde(rename(deserialize = "nid"))]
    pub node_id: String,

    #[serde(rename(deserialize = "cid"))]
    pub component_id: String,

    //#[serde(rename(deserialize = "type"))]
    //ntype: String,
    #[serde(rename(deserialize = "in"))]
    pub inputs: HashMap<String, Vec<NodeOutput>>,

    #[serde(rename(deserialize = "out"))]
    pub outputs: HashMap<String, Vec<NodeInput>>,
}

pub type Flow = HashMap<String, Node>;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Component {}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Struct {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Declarations {
    structs: Vec<Struct>,
    components: Vec<Component>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlowModule {
    pub module: String,
    pub uses: Vec<String>,
    pub graph: Flow,
    pub declarations: Declarations,
}
