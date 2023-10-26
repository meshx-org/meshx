pub static TEST_FLOW: &str = r#"
{
    "module": "test.flow",
    "uses": [],
    "declarations": {
        "structs": [],
        "enums": [],
        "components": []
    }, 
    "graph": {
        "1": { 
            "nid": "1", 
            "type": "action", 
            "cid": "core:dummy1", 
            "in": {}, 
            "out": { 
                "$out": [
                    { "nid": "2", "in": "$in" }
                ]
            }
        },
        "2": { 
            "nid": "2", 
            "type": "action", 
            "cid": "core:dummy2", 
            "in": {
                "$in": [
                    { "nid": "1", "out": "$out" }
                ]
            }, 
            "out": { 
                "$out": []
            }
        }
    }
}
"#;