use nexus_mcp_adapter::mcp_server::JsonRpcRequest;

#[test]
fn test_parse_valid_request() {
    let raw = r#"{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "test"}}"#;
    let req: JsonRpcRequest = serde_json::from_str(raw).unwrap();
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.id.unwrap().as_u64().unwrap(), 1);
    assert_eq!(req.method, "tools/call");
}

#[test]
fn test_parse_notification() {
    let raw = r#"{"jsonrpc": "2.0", "method": "initialized"}"#;
    let req: JsonRpcRequest = serde_json::from_str(raw).unwrap();
    assert!(req.id.is_none());
    assert_eq!(req.method, "initialized");
}

#[test]
fn test_parse_invalid_json() {
    let raw = r#"{"jsonrpc": "2.0", "method": "invalid"#; // missing closing bracket
    let res = serde_json::from_str::<JsonRpcRequest>(raw);
    assert!(res.is_err());
}
