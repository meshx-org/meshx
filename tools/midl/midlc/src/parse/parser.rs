use pest::Parser;

#[derive(Parser)]
#[grammar = "midl.pest"]
pub struct MIDLParser;
