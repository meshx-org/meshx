use crate::flow::FlowModule;
use serde_json::Result;

pub fn parse_flow(flow_str: &str) -> Result<FlowModule> {
    let parsed_flow: FlowModule = serde_json::from_str(flow_str)?;
    Ok(parsed_flow)
}
